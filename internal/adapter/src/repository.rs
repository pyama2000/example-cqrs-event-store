use kernel::aggregate::WidgetAggregate;
use kernel::error::AggregateError;
use kernel::event::WidgetEvent;
use kernel::processor::CommandProcessor;
use lib::Error;

use crate::model::{WidgetAggregateModel, WidgetEventMapper, WidgetEventModel};
use crate::persistence::ConnectionPool;

const QUERY_INSERT_AGGREGATE: &str =
    "INSERT INTO aggregate (widget_id, last_events, aggregate_version) VALUES (?, ?, ?)";
const QUERY_INSERT_EVENT: &str =
    "INSERT INTO event (event_id, widget_id, event_name, payload) VALUES (?, ?, ?, ?)";

pub struct WidgetRepository {
    pool: ConnectionPool,
}

impl WidgetRepository {
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }

    /// 時系列順にイベントを取得する
    async fn list_events(
        &self,
        widget_id: &kernel::Id<WidgetAggregate>,
    ) -> Result<Vec<WidgetEventModel>, AggregateError> {
        sqlx::query_as("SELECT * FROM event WHERE widget_id = ? ORDER BY event_id ASC")
            .bind(widget_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AggregateError::Unknow(e.into()))
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
        sqlx::query(QUERY_INSERT_AGGREGATE)
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
        let aggregate_version = model.aggregate_version();
        let models: Vec<WidgetEventModel> = model.try_into()?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AggregateError::Unknow(e.into()))?;
        for model in models {
            let result = sqlx::query(QUERY_INSERT_EVENT)
                .bind(model.event_id())
                .bind(&widget_id.to_string())
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
        let models: Vec<WidgetEventModel> = self.list_events(&widget_id).await?;
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
        WidgetAggregate::new(widget_id)
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
    use kernel::error::{AggregateError, ApplyCommandError, LoadEventError};
    use kernel::processor::CommandProcessor;
    use kernel::Id;
    use lib::Error;
    use testcontainers::clients::Cli;
    use testcontainers_modules::mysql::Mysql;

    use crate::model::{WidgetAggregateModel, WidgetEventModel};
    use crate::persistence::{connect, ConnectionPool};
    use crate::repository::WidgetRepository;

    use super::{QUERY_INSERT_AGGREGATE, QUERY_INSERT_EVENT};

    type AsyncAssertFn<'a, T> = fn(
        name: &'a str,
        actual: T,
        pool: ConnectionPool,
    ) -> Box<dyn Future<Output = Result<(), Error>> + Send>;

    const WIDGET_NAME: &str = "部品名";
    const WIDGET_DESCRIPTION: &str = "部品の説明";

    #[allow(dead_code)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum DateTime {
        /// 2023-01-01T00:00:00Z
        DT2023_01_01_00_00_00_00,
        /// 2023-01-01T00:00:01Z
        DT2023_01_01_00_00_00_01,
        /// 2023-01-01T00:00:02Z
        DT2023_01_01_00_00_00_02,
        /// 2023-01-01T00:01:00Z
        DT2023_01_01_00_00_01_00,
        /// 2023-01-01T01:00:00Z
        DT2023_01_01_00_01_00_00,
        /// 2023-01-01T01:00:00Z
        DT2023_01_01_01_00_00_00,
        /// 2023-01-02T00:00:00Z
        DT2023_01_02_00_00_00_00,
        /// 2023-02-01T00:00:00Z
        DT2023_02_01_00_00_00_00,
        /// 2024-01-01T00:00:00Z
        DT2024_01_01_00_00_00_00,
    }

    impl DateTime {
        fn id(self) -> String {
            match self {
                DateTime::DT2023_01_01_00_00_00_00 => "01GNNA1J00PQ9J874NBWERBM3Z",
                DateTime::DT2023_01_01_00_00_00_01 => "01GNNA1J015CFH0CA590B4K9K6",
                DateTime::DT2023_01_01_00_00_00_02 => "01GNNA1J02N9H1YCMRA2R9Q562",
                DateTime::DT2023_01_01_00_00_01_00 => "01GNNA1JZ86A6F1G8HV7NYHDCN",
                DateTime::DT2023_01_01_00_01_00_00 => "01GNNA3CK0B63HH8HBYQVRJ5Y8",
                DateTime::DT2023_01_01_01_00_00_00 => "01GNNDFDM0WV3PR6RM8TEA7MZ5",
                DateTime::DT2023_01_02_00_00_00_00 => "01GNQWE9003DQHKPAAHCDCVTJZ",
                DateTime::DT2023_02_01_00_00_00_00 => "01GR57SPM0XBGEG4A13ZBW02G2",
                DateTime::DT2024_01_01_00_00_00_00 => "01HK153X00D14NM09FKYEJ7MPY",
            }
            .to_string()
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Fixture {
        Aggregate {
            widget_id: String,
            last_events: serde_json::Value,
            aggregate_version: u64,
        },
        Event {
            event_id: String,
            widget_id: String,
            event_name: String,
            payload: serde_json::Value,
        },
    }

    impl Fixture {
        fn aggregate(
            widget_id: String,
            last_events: serde_json::Value,
            aggregate_version: u64,
        ) -> Self {
            Self::Aggregate {
                widget_id,
                last_events,
                aggregate_version,
            }
        }

        fn event(
            event_id: String,
            widget_id: String,
            event_name: &str,
            payload: serde_json::Value,
        ) -> Self {
            Self::Event {
                event_id,
                widget_id,
                event_name: event_name.to_string(),
                payload,
            }
        }

        async fn execute(self, pool: &ConnectionPool) -> Result<(), Error> {
            let query = match self {
                Fixture::Aggregate {
                    widget_id,
                    last_events,
                    aggregate_version,
                } => sqlx::query(QUERY_INSERT_AGGREGATE)
                    .bind(widget_id)
                    .bind(last_events)
                    .bind(aggregate_version),
                Fixture::Event {
                    event_id,
                    widget_id,
                    event_name,
                    payload,
                } => sqlx::query(QUERY_INSERT_EVENT)
                    .bind(event_id)
                    .bind(widget_id)
                    .bind(event_name)
                    .bind(payload),
            };
            query.execute(pool).await?;
            Ok(())
        }
    }

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
            assert: (move |name, result, pool| {
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

    /// 集約を復元するテスト
    #[tokio::test]
    async fn test_get_aggregate() -> Result<(), Error> {
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
            widget_id: Id<WidgetAggregate>,
            fixtures: Vec<Fixture>,
            assert: AsyncAssertFn<'a, Result<WidgetAggregate, AggregateError>>,
        }
        let tests = vec![
            TestCase {
                name: "Aggregate テーブルに直近のイベントが集約作成イベントかつイベントが永続化前の場合、Aggregate テーブルから集約を復元できる",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse()?,
                fixtures: vec![Fixture::aggregate(
                    DateTime::DT2023_01_01_00_00_00_00.id(),
                    serde_json::json!([{
                        "event_id": DateTime::DT2023_01_01_00_00_00_00.id(),
                        "event_name": "WidgetCreated",
                        "payload": {
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION,
                        },
                    }]),
                    0,
                )],
                assert: (move |name, result, _| {
                    Box::new(async move {
                        assert!(result.is_ok(), "{name}");
                        let aggregate = result.unwrap();
                        assert_eq!(aggregate.name(), WIDGET_NAME, "{name}");
                        assert_eq!(aggregate.description(), WIDGET_DESCRIPTION, "{name}");
                        assert_eq!(aggregate.version(), 0, "{name}");
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "Aggregate テーブルに直近のイベントが集約作成イベントかつイベントが永続化前の場合、Aggregate テーブルは変更されない",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse()?,
                fixtures: vec![Fixture::aggregate(
                    DateTime::DT2023_01_01_00_00_00_00.id(),
                    serde_json::json!([{
                        "event_id": DateTime::DT2023_01_01_00_00_00_00.id(),
                        "event_name": "WidgetCreated",
                        "payload": {
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION,
                        },
                    }]),
                    0,
                )],
                assert: (move |name, _, pool| {
                    Box::new(async move {
                        let models: Vec<WidgetAggregateModel> =
                            sqlx::query_as("SELECT * FROM aggregate")
                                .fetch_all(&pool)
                                .await?;
                        assert_eq!(models.len(), 1, "{name}");
                        let model = models.first().unwrap();
                        assert_eq!(model.aggregate_version(), 0, "{name}");
                        assert_eq!(
                            model.last_events(),
                            &serde_json::json!([{
                                "event_id": DateTime::DT2023_01_01_00_00_00_00.id(),
                                "event_name": "WidgetCreated",
                                "payload": {
                                    "version": "V1",
                                    "widget_name": WIDGET_NAME,
                                    "widget_description": WIDGET_DESCRIPTION,
                                },
                            }]),
                            "{name}"
                        );
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "Aggregate テーブルに直近のイベントが集約作成イベントかつイベントが永続化前の場合、Event テーブルに集約作成イベントが永続化される",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse()?,
                fixtures: vec![Fixture::aggregate(
                    DateTime::DT2023_01_01_00_00_00_00.id(),
                    serde_json::json!([{
                        "event_id": DateTime::DT2023_01_01_00_00_00_00.id(),
                        "event_name": "WidgetCreated",
                        "payload": {
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION,
                        },
                    }]),
                    0,
                )],
                assert: (move |name, _, pool| {
                    Box::new(async move {
                        let models: Vec<WidgetEventModel> = sqlx::query_as("SELECT * FROM event")
                            .fetch_all(&pool)
                            .await?;
                        assert_eq!(models.len(), 1, "{name}");
                        let model = models.first().unwrap();
                        assert_eq!(
                            model.event_id(),
                            DateTime::DT2023_01_01_00_00_00_00.id(),
                            "{name}"
                        );
                        assert_eq!(model.event_name(), "WidgetCreated", "{name}");
                        assert_eq!(
                            model.payload(),
                            &serde_json::json!({
                                "version": "V1",
                                "widget_name": WIDGET_NAME,
                                "widget_description": WIDGET_DESCRIPTION
                            }),
                            "{name}"
                        );
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "Aggregate テーブルの直近のイベントが永続化後の場合、エラーなく集約を取得でき、Event テーブルも変更されない",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse()?,
                fixtures: vec![
                    Fixture::aggregate(
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        serde_json::json!([{
                            "event_id": DateTime::DT2023_01_01_00_00_00_00.id(),
                            "event_name": "WidgetCreated",
                            "payload": {
                                "version": "V1",
                                "widget_name": WIDGET_NAME,
                                "widget_description": WIDGET_DESCRIPTION,
                            },
                        }]),
                        0,
                    ),
                    Fixture::event(
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        "WidgetCreated",
                        serde_json::json!({
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION,
                        }),
                    ),
                ],
                assert: (move |name, result, pool| {
                    Box::new(async move {
                        assert!(result.is_ok(), "{name}");
                        let aggregate = result.unwrap();
                        assert_eq!(aggregate.name(), WIDGET_NAME, "{name}");
                        assert_eq!(aggregate.description(), WIDGET_DESCRIPTION, "{name}");
                        assert_eq!(aggregate.version(), 0, "{name}");

                        let models: Vec<WidgetEventModel> = sqlx::query_as("SELECT * FROM event")
                            .fetch_all(&pool)
                            .await?;
                        assert_eq!(models.len(), 1, "{name}");
                        let model = models.first().unwrap();
                        assert_eq!(
                            model.event_id(),
                            DateTime::DT2023_01_01_00_00_00_00.id(),
                            "{name}"
                        );
                        assert_eq!(model.event_name(), "WidgetCreated", "{name}");
                        assert_eq!(
                            model.payload(),
                            &serde_json::json!({
                                "version": "V1",
                                "widget_name": WIDGET_NAME,
                                "widget_description": WIDGET_DESCRIPTION
                            }),
                            "{name}"
                        );
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "複数のイベントがある場合、全てのイベントから集約を復元する",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse()?,
                fixtures: vec![
                    Fixture::aggregate(
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        serde_json::json!([{
                            "event_id": DateTime::DT2023_01_01_00_00_00_02.id(),
                            "event_name": "WidgetDescriptionChanged",
                            "payload": {
                                "version": "V1",
                                "widget_description": "部品の説明v2",
                            },
                        }]),
                        2,
                    ),
                    Fixture::event(
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        "WidgetCreated",
                        serde_json::json!({
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION,
                        }),
                    ),
                    Fixture::event(
                        DateTime::DT2023_01_01_00_00_00_01.id(),
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        "WidgetNameChanged",
                        serde_json::json!({
                            "version": "V1",
                            "widget_name": "部品名v2",
                        }),
                    ),
                ],
                assert: (move |name, result, _| {
                    Box::new(async move {
                        assert!(result.is_ok(), "{name}");
                        let aggregate = result.unwrap();
                        assert_eq!(aggregate.name(), "部品名v2", "{name}");
                        assert_eq!(aggregate.description(), "部品の説明v2", "{name}");
                        assert_eq!(aggregate.version(), 2, "{name}");
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "同一のイベントが複数ある場合、イベントの発生順に集約を復元する",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse()?,
                fixtures: vec![
                    Fixture::aggregate(
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        serde_json::json!([{
                            "event_id": DateTime::DT2023_01_01_00_00_00_02.id(),
                            "event_name": "WidgetNameChanged",
                            "payload": {
                                "version": "V1",
                                "widget_name": "部品名v3"
                            },
                        }]),
                        2,
                    ),
                    Fixture::event(
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        "WidgetCreated",
                        serde_json::json!({
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION,
                        }),
                    ),
                    Fixture::event(
                        DateTime::DT2023_01_01_00_00_00_01.id(),
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        "WidgetNameChanged",
                        serde_json::json!({
                            "version": "V1",
                            "widget_name": "部品名v2",
                        }),
                    ),
                ],
                assert: (move |name, result, _| {
                    Box::new(async move {
                        assert!(result.is_ok(), "{name}");
                        let aggregate = result.unwrap();
                        assert_eq!(aggregate.name(), "部品名v3", "{name}");
                        assert_eq!(aggregate.description(), WIDGET_DESCRIPTION, "{name}");
                        assert_eq!(aggregate.version(), 2, "{name}");
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "集約が存在しない場合は、AggregateError::NotFoud が返る",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse()?,
                fixtures: Vec::new(),
                assert: (move |name, result, _| {
                    Box::new(async move {
                        assert!(matches!(result, Err(AggregateError::NotFound)), "{name}");
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "Aggregate テーブルの last_events が不正な場合、エラーが起きる",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse()?,
                fixtures: vec![Fixture::aggregate(
                    DateTime::DT2023_01_01_00_00_00_00.id(),
                    serde_json::json!([]),
                    0,
                )],
                assert: (move |name, result, _| {
                    Box::new(async move {
                        assert!(result.is_err(), "{name}");
                        let e = result.err().unwrap();
                        assert_eq!(e.to_string(), LoadEventError::EventsIsEmpty.to_string(), "{name}");
                        Ok(())
                    })
                }),
            },
        ];
        let repository = WidgetRepository::new(pool.clone());
        for test in tests {
            sqlx::query("TRUNCATE TABLE aggregate")
                .execute(&pool)
                .await?;
            sqlx::query("TRUNCATE TABLE event").execute(&pool).await?;
            for fixture in test.fixtures {
                fixture.execute(&pool).await?;
            }
            let result = repository.get_widget_aggregate(test.widget_id).await;
            Pin::from((test.assert)(test.name, result, pool.clone())).await?;
        }
        Ok(())
    }

    /// Event テーブルからイベントを取得するテスト
    #[tokio::test]
    async fn test_list_events() -> Result<(), Error> {
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
            widget_id: Id<WidgetAggregate>,
            fixtures: Vec<Fixture>,
            assert: AsyncAssertFn<'a, Result<Vec<WidgetEventModel>, AggregateError>>,
        }
        let tests = vec![
            TestCase {
                name: "イベントが存在しない場合",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse()?,
                fixtures: Vec::new(),
                assert: (move |name, result, _| {
                    Box::new(async move {
                        assert!(result.is_ok(), "{name}");
                        let models = result.unwrap();
                        assert!(models.is_empty(), "{name}");
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "イベント ID をバラバラの状態で存在した場合、時系列順にイベントを取得する",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse()?,
                fixtures: vec![
                    Fixture::event(
                        DateTime::DT2023_01_01_00_00_00_01.id(),
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        "WidgetCreated",
                        serde_json::json!({
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION
                        }),
                    ),
                    Fixture::event(
                        DateTime::DT2024_01_01_00_00_00_00.id(),
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        "WidgetNameChanged",
                        serde_json::json!({
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                        }),
                    ),
                    Fixture::event(
                        DateTime::DT2023_01_01_00_00_01_00.id(),
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        "WidgetNameChanged",
                        serde_json::json!({
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                        }),
                    ),
                    Fixture::event(
                        DateTime::DT2023_01_01_00_00_00_02.id(),
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        "WidgetDescriptionChanged",
                        serde_json::json!({
                            "version": "V1",
                            "widget_description": WIDGET_DESCRIPTION
                        }),
                    ),
                ],
                assert: (move |name, result, _| {
                    Box::new(async move {
                        assert!(result.is_ok(), "{name}");
                        let ids: Vec<_> = result
                            .unwrap()
                            .into_iter()
                            .map(|x| x.event_id().to_string())
                            .collect();
                        assert_eq!(
                            ids,
                            vec![
                                DateTime::DT2023_01_01_00_00_00_01.id(),
                                DateTime::DT2023_01_01_00_00_00_02.id(),
                                DateTime::DT2023_01_01_00_00_01_00.id(),
                                DateTime::DT2024_01_01_00_00_00_00.id(),
                            ],
                            "{name}"
                        );
                        Ok(())
                    })
                }),
            },
        ];
        let repository = WidgetRepository::new(pool.clone());
        for test in tests {
            sqlx::query("TRUNCATE TABLE event").execute(&pool).await?;
            for fixture in test.fixtures {
                fixture.execute(&pool).await?;
            }
            let result = repository.list_events(&test.widget_id).await;
            Pin::from((test.assert)(test.name, result, pool.clone())).await?;
        }
        Ok(())
    }

    /// 集約を更新するテスト
    #[tokio::test]
    async fn test_update_widget_aggregate() -> Result<(), Error> {
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
            widget_id: Id<WidgetAggregate>,
            fixtures: Vec<Fixture>,
            command_state_builder:
                fn(aggregate: WidgetAggregate) -> Result<WidgetCommandState, ApplyCommandError>,
            assert: AsyncAssertFn<'a, Result<(), AggregateError>>,
        }
        let tests = vec![
            TestCase {
                name: "集約作成後に更新した場合、エラーなく Aggregate テーブルが更新される",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse()?,
                fixtures: vec![Fixture::aggregate(
                    DateTime::DT2023_01_01_00_00_00_00.id(),
                    serde_json::json!([{
                        "event_id": DateTime::DT2023_01_01_00_00_00_00.id(),
                        "event_name": "WidgetCreated",
                        "payload": {
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION,
                        },
                    }]),
                    0,
                )],
                command_state_builder: |aggregate| {
                    aggregate.apply_command(WidgetCommand::ChangeWidgetName {
                        widget_name: "部品名v2".to_string(),
                    })
                },
                assert: (move |name, result, pool| {
                    Box::new(async move {
                        assert!(result.is_ok(), "{name}");

                        let models: Vec<WidgetAggregateModel> =
                            sqlx::query_as("SELECT * FROM aggregate")
                                .fetch_all(&pool)
                                .await?;
                        assert_eq!(models.len(), 1, "{name}");
                        let model = models.first().unwrap();
                        assert_eq!(model.aggregate_version(), 1, "{name}");
                        let last_events = model.last_events().as_array();
                        assert!(last_events.is_some(), "{name}");
                        let last_events = last_events.unwrap();
                        assert_eq!(last_events.len(), 1, "{name}");
                        let event = last_events.first().unwrap();
                        assert_eq!(
                            event.get("event_name"),
                            Some(&serde_json::json!("WidgetNameChanged")),
                            "{name}"
                        );
                        assert_eq!(
                            event.get("payload"),
                            Some(&serde_json::json!({
                                "version": "V1",
                                "widget_name": "部品名v2",
                            })),
                            "{name}"
                        );
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "更新済みの集約を更新した場合、エラーなく Aggregate テーブルが更新される",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse()?,
                fixtures: vec![
                    Fixture::event(
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        "WidgetCreated",
                        serde_json::json!({
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION,
                        }),
                    ),
                    Fixture::aggregate(
                        DateTime::DT2023_01_01_00_00_00_00.id(),
                        serde_json::json!([{
                            "event_id": DateTime::DT2023_01_01_00_00_00_01.id(),
                            "event_name": "WidgetNameChanged",
                            "payload": {
                                "version": "V1",
                                "widget_name": "部品名v2",
                            },
                        }]),
                        1,
                    ),
                ],
                command_state_builder: |aggregate| {
                    aggregate.apply_command(WidgetCommand::ChangeWidgetName {
                        widget_name: "部品名v3".to_string(),
                    })
                },
                assert: (move |name, result, pool| {
                    Box::new(async move {
                        assert!(result.is_ok(), "{name}");

                        let models: Vec<WidgetAggregateModel> =
                            sqlx::query_as("SELECT * FROM aggregate")
                                .fetch_all(&pool)
                                .await?;
                        assert_eq!(models.len(), 1, "{name}");
                        let model = models.first().unwrap();
                        assert_eq!(model.aggregate_version(), 2, "{name}");
                        let last_events = model.last_events().as_array();
                        assert!(last_events.is_some(), "{name}");
                        let last_events = last_events.unwrap();
                        assert_eq!(last_events.len(), 1, "{name}");
                        let event = last_events.first().unwrap();
                        assert_eq!(
                            event.get("event_name"),
                            Some(&serde_json::json!("WidgetNameChanged")),
                            "{name}"
                        );
                        assert_eq!(
                            event.get("payload"),
                            Some(&serde_json::json!({
                                "version": "V1",
                                "widget_name": "部品名v3",
                            })),
                            "{name}"
                        );
                        Ok(())
                    })
                }),
            },
        ];
        let repository = WidgetRepository::new(pool.clone());
        for test in tests {
            sqlx::query("TRUNCATE TABLE aggregate")
                .execute(&pool)
                .await?;
            sqlx::query("TRUNCATE TABLE event").execute(&pool).await?;
            for fixture in test.fixtures {
                fixture.execute(&pool).await?;
            }
            let aggregate = repository.get_widget_aggregate(test.widget_id).await?;
            let result = repository
                .update_widget_aggregate((test.command_state_builder)(aggregate)?)
                .await;
            Pin::from((test.assert)(test.name, result, pool.clone())).await?;
        }
        Ok(())
    }
}
