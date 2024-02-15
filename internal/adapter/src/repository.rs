use kernel::aggregate::WidgetAggregate;
use kernel::error::AggregateError;
use kernel::event::WidgetEvent;
use kernel::processor::CommandProcessor;

use crate::model::{WidgetAggregateModel, WidgetEventMapper, WidgetEventModel, WidgetEventModels};
use crate::persistence::ConnectionPool;

pub struct WidgetRepository {
    pool: ConnectionPool,
}

impl WidgetRepository {
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }
}

impl CommandProcessor for WidgetRepository {
    async fn create_widget_aggregate(
        &self,
        command_state: kernel::aggregate::WidgetCommandState,
    ) -> Result<(), AggregateError> {
        let model: WidgetAggregateModel = command_state.try_into()?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AggregateError::Unknow(e.into()))?;
        sqlx::query(
            "INSERT INTO aggregate (widget_id, last_events, aggregate_version) VALUES (?, ?, ?)",
        )
        .bind(model.widget_id())
        .bind(model.last_events())
        .bind(model.aggregate_version())
        .execute(&mut *tx)
        .await
        .map_err(|e| AggregateError::Unknow(e.into()))?;
        tx.commit()
            .await
            .map_err(|e| AggregateError::Unknow(e.into()))?;
        Ok(())
    }

    async fn get_widget_aggregate(
        &self,
        widget_id: kernel::Id<kernel::aggregate::WidgetAggregate>,
    ) -> Result<Option<kernel::aggregate::WidgetAggregate>, AggregateError> {
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
                    _ => return Err(AggregateError::Unknow(e.into())),
                },
            };
        // ビジネスロジックの適用の前にイベントと集約のデータが正しい状態にあることを保証するために
        // 前回集約が保存された際に作成されたイベントを Event テーブルに個々の項目として永続化する
        let widget_id = widget_id.to_string();
        let aggregate_version = model.aggregate_version();
        let WidgetEventModels(models) = model.try_into()?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AggregateError::Unknow(e.into()))?;
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
                    _ => return Err(AggregateError::Unknow(e.into())),
                }
            }
        }
        tx.commit()
            .await
            .map_err(|e| AggregateError::Unknow(e.into()))?;
        // 関連するすべてのイベントを読み込んで集約の状態を復元する
        let models: Vec<WidgetEventModel> =
            // NOTE: 時系列にしたがってイベントから集約を復元するために event_id の昇順でソートする
            sqlx::query_as("SELECT * FROM event WHERE widget_id = ? ORDER BY event_id ASC")
                .bind(&widget_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| AggregateError::Unknow(e.into()))?;
        let mappers: Vec<Result<WidgetEventMapper, lib::Error>> =
            models.into_iter().map(|x| x.try_into()).collect();
        if mappers.iter().any(|x| x.is_err()) {
            return Err(AggregateError::Unknow("Parse mapper from model".into()));
        }
        let events: Vec<Result<WidgetEvent, lib::Error>> =
            mappers.into_iter().map(|x| x.unwrap().try_into()).collect();
        if events.iter().any(|x| x.is_err()) {
            return Err(AggregateError::Unknow("Parse event from mapper".into()));
        }
        let events: Vec<_> = events.into_iter().map(|x| x.unwrap()).collect();
        Ok(Some(
            WidgetAggregate::new(widget_id.parse()?)
                .load_events(events, aggregate_version)
                .map_err(|e| AggregateError::Unknow(e.into()))?,
        ))
    }

    async fn update_widget_aggregate(
        &self,
        command_state: kernel::aggregate::WidgetCommandState,
    ) -> Result<(), AggregateError> {
        let model: WidgetAggregateModel = command_state.try_into()?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AggregateError::Unknow(e.into()))?;
        let result = sqlx::query(
            "
            UPDATE
                aggregate
            SET
                last_events = ?, aggregate_version = ?
            WHERE
                widget_id = ? AND aggregate_version = ?
            ",
        )
        .bind(model.last_events())
        .bind(model.aggregate_version())
        .bind(model.widget_id())
        .bind(model.aggregate_version().saturating_sub(1))
        .execute(&mut *tx)
        .await
        .map_err(|e| AggregateError::Unknow(e.into()))?;
        // NOTE: 同時接続で既に Aggregate が更新されていた場合はエラーを返す
        if result.rows_affected() == 0 {
            return Err(AggregateError::Conflict);
        }
        tx.commit()
            .await
            .map_err(|e| AggregateError::Unknow(e.into()))?;
        Ok(())
    }
}
