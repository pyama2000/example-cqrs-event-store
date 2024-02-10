use kernel::aggregate::{WidgetAggregate, WidgetAggregateState};
use kernel::event::WidgetEvent;
use kernel::processor::CommandProcessor;
use lib::Result;

use crate::model::{WidgetAggregateModel, WidgetEventMapper, WidgetEventModel, WidgetEventModels};
use crate::persistence::ConnectionPool;

pub struct WidgetRepository {
    pool: ConnectionPool,
}

impl CommandProcessor for WidgetRepository {
    async fn create_widget_aggregate(
        &self,
        command_state: kernel::aggregate::WidgetCommandState,
    ) -> Result<()> {
        let model: WidgetAggregateModel = command_state.try_into()?;
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO aggregate (widget_id, last_events, aggregate_version) VALUES (?, ?, ?)",
        )
        .bind(model.widget_id())
        .bind(model.last_events())
        .bind(model.aggregate_version())
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn get_widget_aggregate(
        &self,
        widget_id: kernel::Id<kernel::aggregate::WidgetAggregate>,
    ) -> Result<Option<kernel::aggregate::WidgetAggregate>> {
        // Aggregate テーブルから関連する集約項目を取得する
        let model: WidgetAggregateModel =
            match sqlx::query_as("SELECT * FROM aggregate WHERE widget_id = ?")
                .bind(widget_id.to_string())
                .fetch_one(&self.pool)
                .await
            {
                Ok(x) => x,
                Err(e) => match e {
                    sqlx::Error::RowNotFound => return Ok(None),
                    _ => return Err(Box::new(e)),
                },
            };
        // ビジネスロジックの適用の前にイベントと集約のデータが正しい状態にあることを保証するために
        // 前回集約が保存された際に作成されたイベントを Event テーブルに個々の項目として永続化する
        let widget_id = model.widget_id().to_string();
        let WidgetEventModels(models) = model.try_into()?;
        let mut tx = self.pool.begin().await?;
        for model in models {
            let result = sqlx::query(
                "INSERT INTO event (event_id, widget_id, event_name, payload) VALUES (?, ?, ?, ?)",
            )
            .bind(model.event_id())
            .bind(&widget_id)
            .bind(model.event_name())
            .bind(model.payload())
            .execute(&mut *tx)
            .await;
            if let Err(e) = result {
                match e.as_database_error() {
                    // NOTE: イベントが既に存在してもイベントは変更不可能なのでエラーを無視する
                    Some(e) if e.is_unique_violation() => continue,
                    _ => return Err(Box::new(e)),
                }
            }
        }
        tx.commit().await?;
        // 関連するすべてのイベントを読み込んで集約の状態を復元する
        let models: Vec<WidgetEventModel> =
            // NOTE: 時系列にしたがってイベントから集約を復元するために event_id の昇順でソートする
            sqlx::query_as("SELECT * FROM event WHERE widget_id = ? ORDER BY event_id ASC")
                .bind(widget_id)
                .fetch_all(&self.pool)
                .await?;
        let mappers: Vec<Result<WidgetEventMapper>> =
            models.into_iter().map(|x| x.try_into()).collect();
        if mappers.iter().any(|x| x.is_err()) {
            return Err("Parse mapper from model".into());
        }
        let events: Vec<Result<WidgetEvent>> =
            mappers.into_iter().map(|x| x.unwrap().try_into()).collect();
        if events.iter().any(|x| x.is_err()) {
            return Err("Parse event from mapper".into());
        }
        let events: Vec<_> = events.into_iter().map(|x| x.unwrap()).collect();
        Ok(Some(
            WidgetAggregateState::new(WidgetAggregate::default(), events).restore(),
        ))
    }

    async fn update_widget_aggregate(
        &self,
        command_state: kernel::aggregate::WidgetCommandState,
    ) -> Result<()> {
        todo!()
    }
}
