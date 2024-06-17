use std::collections::HashMap;

use aws_sdk_dynamodb::operation::get_item::GetItemError;
use aws_sdk_dynamodb::types::AttributeValue;
use kernel::aggregate::WidgetAggregate;
use kernel::error::AggregateError;
use kernel::event::WidgetEvent;
use kernel::processor::CommandProcessor;
use lib::Error;

use crate::model::{WidgetAggregateModel, WidgetEventMapper, WidgetEventModel};
use crate::persistence::{ConnectionPool, DbClient};

#[derive(Debug)]
pub struct WidgetRepository {
    pool: ConnectionPool,
    client: DbClient,
}

impl WidgetRepository {
    pub fn new(pool: ConnectionPool, client: DbClient) -> Self {
        Self { pool, client }
    }

    /// 時系列順にイベントを取得する
    async fn list_events(
        &self,
        widget_id: &kernel::Id<WidgetAggregate>,
    ) -> Result<Vec<WidgetEventModel>, AggregateError> {
        let results: Vec<_> = self
            .client
            .query()
            .table_name("EventStore")
            .consistent_read(true)
            .key_condition_expression("AggregateID = :widget_id")
            .expression_attribute_values("widget_id", AttributeValue::S(widget_id.to_string()))
            .into_paginator()
            .send()
            .collect()
            .await;
        if results.iter().any(Result::is_err) {
            let errs: Vec<_> = results
                .into_iter()
                .filter(|x| x.is_err())
                .map(|x| x.unwrap_err().to_string())
                .collect();
            return Err(AggregateError::Unknow(errs.join(",\n").into()));
        }
        let items: Vec<_> = results
            .into_iter()
            .map(Result::unwrap)
            .flat_map(|x| x.items().to_vec())
            .collect();
        serde_dynamo::from_items(items).map_err(|e| AggregateError::Unknow(e.into()))
    }
}

impl CommandProcessor for WidgetRepository {
    #[tracing::instrument(ret, err)]
    async fn create_widget_aggregate(
        &self,
        command_state: kernel::aggregate::WidgetCommandState,
    ) -> Result<(), AggregateError> {
        let model: WidgetAggregateModel = command_state.try_into()?;
        self.client
            .put_item()
            .table_name("Aggregate")
            .set_item(Some(
                serde_dynamo::to_item(model).map_err(|e| AggregateError::Unknow(e.into()))?,
            ))
            .send()
            .await
            .map_err(|e| AggregateError::Unknow(e.into()))?;
        Ok(())
    }

    #[tracing::instrument(ret, err)]
    async fn get_widget_aggregate(
        &self,
        widget_id: kernel::Id<kernel::aggregate::WidgetAggregate>,
    ) -> Result<kernel::aggregate::WidgetAggregate, AggregateError> {
        // Aggregate テーブルから関連する集約項目を取得する
        let model: WidgetAggregateModel = match self
            .client
            .get_item()
            .table_name("Aggregate")
            .key("ID", AttributeValue::S(widget_id.to_string()))
            .send()
            .await
        {
            Ok(x) => {
                let item = x
                    .item()
                    .ok_or_else(|| AggregateError::Unknow("item is None".into()))?;
                serde_dynamo::from_item(item.clone())
                    .map_err(|e| AggregateError::Unknow(e.into()))?
            }
            Err(err) => match err.into_service_error() {
                GetItemError::ResourceNotFoundException(_) => return Err(AggregateError::NotFound),
                e => return Err(AggregateError::Unknow(e.into())),
            },
        };
        // ビジネスロジックの適用の前にイベントと集約のデータが正しい状態にあることを保証するために
        // 前回集約が保存された際に作成されたイベントを Event テーブルに個々の項目として永続化する
        let aggregate_version = model.aggregate_version();
        let models: Vec<WidgetEventModel> = model.try_into()?;
        for model in models {
            self.client
                .put_item()
                .table_name("EventStore")
                .set_item(Some(
                    serde_dynamo::to_item(model).map_err(|e| AggregateError::Unknow(e.into()))?,
                ))
                .condition_expression("attribute_not_exists(ID)")
                .send()
                .await
                .map_err(|e| AggregateError::Unknow(e.into()))?;
        }
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

    #[tracing::instrument(ret, err)]
    async fn update_widget_aggregate(
        &self,
        command_state: kernel::aggregate::WidgetCommandState,
    ) -> Result<(), AggregateError> {
        let model: WidgetAggregateModel = command_state.try_into()?;
        self.client
            .update_item()
            .table_name("Aggregate")
            .key(
                "ID",
                serde_dynamo::to_attribute_value(model.widget_id())
                    .map_err(|e| AggregateError::Unknow(e.into()))?,
            )
            .set_expression_attribute_values(Some(HashMap::from([
                (
                    "NewLastEvents".to_string(),
                    serde_dynamo::to_attribute_value(model.last_events())
                        .map_err(|e| AggregateError::Unknow(e.into()))?,
                ),
                (
                    "NewAggregateVersion".to_string(),
                    serde_dynamo::to_attribute_value(model.aggregate_version())
                        .map_err(|e| AggregateError::Unknow(e.into()))?,
                ),
                (
                    "CurrentAggregateVersion".to_string(),
                    serde_dynamo::to_attribute_value(model.aggregate_version().saturating_sub(1))
                        .map_err(|e| AggregateError::Unknow(e.into()))?,
                ),
            ])))
            .update_expression(
                "SET LastEvents = :NewLastEvents,AggregateVersion = :NewAggregateVersion",
            )
            .condition_expression("AggregateVersion = :CurrentAggregateVersion")
            .send()
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
    use lib::{test_client, DateTime, Error};
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
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

    const WIDGET_NAME: &str = "部品名";
    const WIDGET_DESCRIPTION: &str = "部品の説明";

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
                Box::pin(async move {
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
        let repository = WidgetRepository::new(pool.clone(), test_client().await);
        for test in tests {
            let result = repository.create_widget_aggregate(test.command_state).await;
            (test.assert)(test.name, result, pool.clone()).await?;
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
                    Box::pin(async move {
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
                    Box::pin(async move {
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
                    Box::pin(async move {
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
                    Box::pin(async move {
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
                    Box::pin(async move {
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
                    Box::pin(async move {
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
                    Box::pin(async move {
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
                    Box::pin(async move {
                        assert!(result.is_err(), "{name}");
                        let e = result.err().unwrap();
                        assert_eq!(e.to_string(), LoadEventError::EventsIsEmpty.to_string(), "{name}");
                        Ok(())
                    })
                }),
            },
        ];
        let repository = WidgetRepository::new(pool.clone(), test_client().await);
        for test in tests {
            sqlx::query("TRUNCATE TABLE aggregate")
                .execute(&pool)
                .await?;
            sqlx::query("TRUNCATE TABLE event").execute(&pool).await?;
            for fixture in test.fixtures {
                fixture.execute(&pool).await?;
            }
            let result = repository.get_widget_aggregate(test.widget_id).await;
            (test.assert)(test.name, result, pool.clone()).await?;
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
                    Box::pin(async move {
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
                    Box::pin(async move {
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
        let repository = WidgetRepository::new(pool.clone(), test_client().await);
        for test in tests {
            sqlx::query("TRUNCATE TABLE event").execute(&pool).await?;
            for fixture in test.fixtures {
                fixture.execute(&pool).await?;
            }
            let result = repository.list_events(&test.widget_id).await;
            (test.assert)(test.name, result, pool.clone()).await?;
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
                    Box::pin(async move {
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
                    Box::pin(async move {
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
        let repository = WidgetRepository::new(pool.clone(), test_client().await);
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
            (test.assert)(test.name, result, pool.clone()).await?;
        }
        Ok(())
    }

    /// 同時に集約を更新した時に更新済みのエラーが返ることをテストする
    #[tokio::test]
    async fn test_update_widget_aggregate_return_conflict_error() -> Result<(), Error> {
        const NAME: &str = "同時に集約を更新した場合, 更新済みのエラーが返る";

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

        let repository = WidgetRepository::new(pool.clone(), test_client().await);
        let widget_id: Id<WidgetAggregate> = DateTime::DT2023_01_01_00_00_00_00.id().parse()?;
        let fixture = Fixture::aggregate(
            widget_id.to_string(),
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
        );
        fixture.execute(&pool).await?;
        let aggregate = repository.get_widget_aggregate(widget_id).await?;
        let should_success_command_state =
            aggregate
                .clone()
                .apply_command(WidgetCommand::ChangeWidgetName {
                    widget_name: WIDGET_NAME.to_string(),
                })?;
        let should_conflict_command_state =
            aggregate.apply_command(WidgetCommand::ChangeWidgetDescription {
                widget_description: WIDGET_DESCRIPTION.to_string(),
            })?;
        let result = repository
            .update_widget_aggregate(should_success_command_state)
            .await;
        assert!(result.is_ok(), "{NAME}");
        let result = repository
            .update_widget_aggregate(should_conflict_command_state)
            .await;
        assert!(
            result.is_err_and(|e| matches!(e, AggregateError::Conflict)),
            "{NAME}"
        );
        Ok(())
    }
}
