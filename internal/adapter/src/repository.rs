use kernel::aggregate::WidgetAggregate;
use kernel::error::AggregateError;
use kernel::event::WidgetEvent;
use kernel::processor::CommandProcessor;
use lib::Error;

use crate::model::{WidgetAggregateModel, WidgetEventMapper, WidgetEventModel};
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
    ) -> Result<kernel::aggregate::WidgetAggregate, AggregateError> {
        // Aggregate テーブルから関連する集約項目を取得する
        let model: WidgetAggregateModel =
            match sqlx::query_as("SELECT * FROM aggregate WHERE widget_id = ?")
                .bind(widget_id.to_string())
                .fetch_one(&self.pool)
                .await
            {
                Ok(x) => x,
                Err(e) => match e {
                    sqlx::Error::RowNotFound => return Err(AggregateError::NotFound),
                    _ => return Err(AggregateError::Unknow(e.into())),
                },
            };
        // ビジネスロジックの適用の前にイベントと集約のデータが正しい状態にあることを保証するために
        // 前回集約が保存された際に作成されたイベントを Event テーブルに個々の項目として永続化する
        let widget_id = widget_id.to_string();
        let aggregate_version = model.aggregate_version();
        let models: Vec<WidgetEventModel> = model.try_into()?;
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
        let mappers: Vec<Result<WidgetEventMapper, Error>> =
            models.into_iter().map(|x| x.try_into()).collect();
        if mappers.iter().any(|x| x.is_err()) {
            return Err(AggregateError::Unknow("Parse mapper from model".into()));
        }
        let events: Vec<Result<WidgetEvent, Error>> =
            mappers.into_iter().map(|x| x.unwrap().try_into()).collect();
        if events.iter().any(|x| x.is_err()) {
            return Err(AggregateError::Unknow("Parse event from mapper".into()));
        }
        let events: Vec<_> = events.into_iter().map(|x| x.unwrap()).collect();
        WidgetAggregate::new(widget_id.parse()?)
            .load_events(events, aggregate_version)
            .map_err(|e| AggregateError::Unknow(e.into()))
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

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;

    use kernel::aggregate::{WidgetAggregate, WidgetCommandState};
    use kernel::command::WidgetCommand;
    use kernel::error::AggregateError;
    use kernel::processor::CommandProcessor;
    use kernel::Id;
    use lib::Error;
    use testcontainers::clients::Cli;
    use testcontainers_modules::mysql::Mysql;

    use crate::model::{WidgetAggregateModel, WidgetEventModel};
    use crate::persistence::{connect, ConnectionPool};
    use crate::repository::WidgetRepository;

    type AsyncAssertFn<'a, T> = Box<
        fn(
            name: &'a str,
            actual: T,
            pool: ConnectionPool,
        ) -> Box<dyn Future<Output = Result<(), Error>> + Send>,
    >;

    const WIDGET_NAME: &str = "部品名";
    const WIDGET_DESCRIPTION: &str = "部品の説明";

    /// 集約の作成を永続化するテスト
    #[tokio::test]
    async fn test_crate_aggregate() -> Result<(), Error> {
        let docker = Cli::default();
        let container = docker.run(Mysql::default());
        let pool = connect(&format!(
            "mysql://root@127.0.0.1:{}/mysql",
            container.get_host_port_ipv4(3306)
        ))
        .await?;
        sqlx::query(include_str!(
            "../../../migrations/20240210132634_create_aggregate.sql"
        ))
        .execute(&pool)
        .await?;
        sqlx::query(include_str!(
            "../../../migrations/20240210132646_create_event.sql"
        ))
        .execute(&pool)
        .await?;

        struct TestCase<'a> {
            name: &'a str,
            command_state: WidgetCommandState,
            assert: AsyncAssertFn<'a, Result<(), AggregateError>>,
        }
        let tests = vec![TestCase {
            name: "部品作成コマンドの場合、Aggregate テーブルにのみ永続化される",
            command_state: WidgetAggregate::default().apply_command(
                WidgetCommand::CreateWidget {
                    widget_name: WIDGET_NAME.to_string(),
                    widget_description: WIDGET_DESCRIPTION.to_string(),
                },
            )?,
            assert: Box::new(move |name, result, pool| {
                Box::new(async move {
                    assert!(result.is_ok(), "{name}");
                    let models: Vec<WidgetAggregateModel> =
                        sqlx::query_as("SELECT * FROM aggregate")
                            .fetch_all(&pool)
                            .await?;
                    assert_eq!(models.len(), 1, "{name}");
                    let model = models.first().unwrap();
                    assert!(
                        model.widget_id().parse::<Id<WidgetAggregate>>().is_ok(),
                        "{name}"
                    );
                    assert_eq!(model.aggregate_version(), 0, "{name}");
                    let last_events = model.last_events().as_array();
                    assert!(last_events.is_some(), "{name}");
                    let last_events = last_events.unwrap();
                    assert_eq!(last_events.len(), 1, "{name}");
                    let event = last_events.first().unwrap();
                    assert_eq!(
                        event.get("event_name"),
                        Some(&serde_json::json!("WidgetCreated")),
                        "{name}"
                    );
                    assert_eq!(
                        event.get("payload"),
                        Some(&serde_json::json!({
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION,
                        })),
                        "{name}"
                    );
                    let models: Vec<WidgetEventModel> = sqlx::query_as("SELECT * FROM event")
                        .fetch_all(&pool)
                        .await?;
                    assert_eq!(models.len(), 0, "{name}");
                    Ok(())
                })
            }),
        }];
        let repository = WidgetRepository::new(pool.clone());
        for test in tests {
            let result = repository.create_widget_aggregate(test.command_state).await;
            Pin::from((test.assert)(test.name, result, pool.clone())).await?;
        }
        Ok(())
    }

        Ok(())
    }
}
