use aws_sdk_dynamodb::types::{AttributeValue, Put, TransactWriteItem, Update};
use kernel::command::error::CommandProcessorError;
use kernel::{CommandKernelError, CommandProcessor, Event, Id};
use tracing::instrument;

use crate::{AGGREGATE_TABLE_NAME, EVENT_SEQUENCE_TABLE_NAME, EVENT_STORE_TABLE_NAME};

use super::{AggregateModel, EventSequenceModel, EventStoreModel};

/// コマンド操作を行うリポジトリ
#[derive(Debug, Clone)]
pub struct CommandRepository {
    dynamodb: aws_sdk_dynamodb::Client,
}

impl CommandRepository {
    /// Creates a new [`CommandRepository`].
    #[must_use]
    pub fn new(dynamodb: aws_sdk_dynamodb::Client) -> Self {
        Self { dynamodb }
    }
}

impl CommandProcessor for CommandRepository {
    #[instrument(skip(self), err, ret)]
    async fn create(
        &self,
        aggregate: kernel::Aggregate,
        event: kernel::Event,
    ) -> Result<(), kernel::CommandKernelError> {
        if !matches!(event, kernel::Event::Created { .. }) {
            return Err(CommandProcessorError::InvalidEvent.into());
        }
        let mut transact_items = Vec::new();
        let sequence = EventSequenceModel::new(aggregate.id().to_string(), 0);
        let event_id = sequence.version();
        transact_items.push(
            TransactWriteItem::builder()
                .put(
                    Put::builder()
                        .table_name(EVENT_STORE_TABLE_NAME)
                        .set_item(Some(
                            EventStoreModel::new(
                                event_id,
                                aggregate.id().to_string(),
                                event.into(),
                            )
                            .try_into()?,
                        ))
                        .condition_expression(
                            "attribute_not_exists(id) AND attribute_not_exists(aggregate_id)",
                        )
                        .build()
                        .map_err(|e| CommandKernelError::Unknown(e.into()))?,
                )
                .build(),
        );
        transact_items.push(
            TransactWriteItem::builder()
                .put(
                    Put::builder()
                        .table_name(EVENT_SEQUENCE_TABLE_NAME)
                        .set_item(Some(sequence.try_into()?))
                        .condition_expression("attribute_not_exists(aggregate_id)")
                        .build()
                        .map_err(|e| CommandKernelError::Unknown(e.into()))?,
                )
                .build(),
        );
        transact_items.push(
            TransactWriteItem::builder()
                .put(
                    Put::builder()
                        .table_name(AGGREGATE_TABLE_NAME)
                        .set_item(Some(AggregateModel::from(aggregate).try_into()?))
                        .condition_expression("attribute_not_exists(id)")
                        .build()
                        .map_err(|e| CommandKernelError::Unknown(e.into()))?,
                )
                .build(),
        );
        self.dynamodb
            .transact_write_items()
            .set_transact_items(Some(transact_items))
            .send()
            .await
            .map_err(|e| CommandKernelError::Unknown(e.into()))?;
        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    async fn get(
        &self,
        id: Id<kernel::Aggregate>,
    ) -> Result<Option<kernel::Aggregate>, kernel::CommandKernelError> {
        use aws_sdk_dynamodb::operation::get_item::GetItemError::ResourceNotFoundException;

        let output = match self
            .dynamodb
            .get_item()
            .table_name(AGGREGATE_TABLE_NAME)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
        {
            Ok(output) => output,
            Err(e) => match e.into_service_error() {
                ResourceNotFoundException(_) => return Ok(None),
                e => return Err(CommandKernelError::Unknown(e.into())),
            },
        };
        let Some(item) = output.item() else {
            return Ok(None);
        };
        let model: AggregateModel = serde_dynamo::from_item(item.clone())
            .map_err(|e| CommandKernelError::Unknown(e.into()))?;
        Ok(Some(model.try_into()?))
    }

    #[allow(clippy::too_many_lines)]
    #[instrument(skip(self), err, ret)]
    async fn update(
        &self,
        aggregate: kernel::Aggregate,
        events: Vec<kernel::Event>,
    ) -> Result<(), kernel::CommandKernelError> {
        if events.is_empty() {
            return Err(CommandProcessorError::EmptyEvent.into());
        }
        if events
            .iter()
            .any(|event| matches!(event, Event::Created { .. }))
        {
            return Err(CommandProcessorError::InvalidEvent.into());
        }

        let aggregate_id = AttributeValue::S(aggregate.id().to_string());
        let sequence: EventSequenceModel = serde_dynamo::from_item(
            self.dynamodb
                .get_item()
                .table_name(EVENT_SEQUENCE_TABLE_NAME)
                .key("aggregate_id", aggregate_id.clone())
                .send()
                .await
                .map_err(|e| CommandKernelError::Unknown(e.into()))?
                .item()
                .ok_or_else(|| CommandKernelError::Unknown("event sequence not found".into()))?
                .clone(),
        )
        .map_err(|e| CommandKernelError::Unknown(e.into()))?;
        let current_event_id = sequence.version();
        let mut new_event_id = current_event_id;
        let mut transact_items = Vec::new();
        for event in events {
            new_event_id += 1;
            transact_items.push(
                TransactWriteItem::builder()
                    .put(
                        Put::builder()
                            .table_name(EVENT_STORE_TABLE_NAME)
                            .set_item(Some(
                                EventStoreModel::new(
                                    new_event_id,
                                    aggregate.id().to_string(),
                                    event.into(),
                                )
                                .try_into()?,
                            ))
                            .condition_expression(
                                "attribute_not_exists(id) AND attribute_not_exists(aggregate_id)",
                            )
                            .build()
                            .map_err(|e| CommandKernelError::Unknown(e.into()))?,
                    )
                    .build(),
            );
        }
        transact_items.push(
            TransactWriteItem::builder()
                .update(
                    Update::builder()
                        .table_name(EVENT_SEQUENCE_TABLE_NAME)
                        .key("aggregate_id", aggregate_id.clone())
                        .expression_attribute_values(
                            ":current_version",
                            sequence.version_attribute_value()?,
                        )
                        .expression_attribute_values(
                            ":new_version",
                            AttributeValue::N(new_event_id.to_string()),
                        )
                        .update_expression("SET version = :new_version")
                        .condition_expression(
                            "attribute_exists(aggregate_id) AND version = :current_version",
                        )
                        .build()
                        .map_err(|e| CommandKernelError::Unknown(e.into()))?,
                )
                .build(),
        );
        let aggregate_model: AggregateModel = aggregate.into();
        transact_items.push(
            TransactWriteItem::builder()
                .update(
                    Update::builder()
                        .table_name(AGGREGATE_TABLE_NAME)
                        .key("id", aggregate_id)
                        .expression_attribute_values(
                            ":current_version",
                            serde_dynamo::to_attribute_value(
                                // NOTE: この時点のAggregateはバージョンが更新されているので1つ前のバージョンを指定する
                                aggregate_model.version().checked_sub(1).ok_or_else(|| {
                                    CommandKernelError::Unknown("invalid aggregate version".into())
                                })?,
                            )
                            .map_err(|e| CommandKernelError::Unknown(e.into()))?,
                        )
                        .expression_attribute_values(
                            ":new_version",
                            aggregate_model.version_attribute_value()?,
                        )
                        .expression_attribute_values(
                            ":new_payload",
                            aggregate_model.payload_attribute_value()?,
                        )
                        .update_expression("SET version = :new_version, payload = :new_payload")
                        .condition_expression("attribute_exists(id) AND version = :current_version")
                        .build()
                        .map_err(|e| CommandKernelError::Unknown(e.into()))?,
                )
                .build(),
        );
        self.dynamodb
            .transact_write_items()
            .set_transact_items(Some(transact_items))
            .send()
            .await
            .map_err(|e| CommandKernelError::Unknown(e.into()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use kernel::command::error::CommandProcessorError;
    use kernel::{Aggregate, CommandKernelError, CommandProcessor, Event, Id};
    use testcontainers::ContainerAsync;
    use testcontainers_modules::dynamodb_local::DynamoDb;

    use crate::command::repository::{
        AGGREGATE_TABLE_NAME, EVENT_SEQUENCE_TABLE_NAME, EVENT_STORE_TABLE_NAME,
    };
    use crate::command::{
        AggregateModel, AggregatePayload, EventPayload, EventSequenceModel, EventStoreModel, Item,
    };

    use super::CommandRepository;

    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    const TENANT_NAME: &str = "テストテナント";
    const ITEM_NAME: &str = "テスト商品";

    #[tokio::test]
    async fn test_with_container_create_ok() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            event: Event,
            expected_aggregate_models: Vec<AggregateModel>,
            expected_event_store_models: Vec<EventStoreModel>,
            expected_event_sequence_models: Vec<EventSequenceModel>,
        }

        let Context {
            container: _container,
            repository,
            dynamodb,
        } = Context::with_container().await.unwrap();

        let id = Id::generate();

        let tests = [TestCase {
            name: "集約作成イベントを永続化する場合、それぞれのテーブルにレコードが作成される",
            aggregate: Aggregate::new(id.clone(), TENANT_NAME.to_string(), Vec::new(), 1),
            event: Event::Created {
                name: TENANT_NAME.to_string(),
            },
            expected_aggregate_models: vec![AggregateModel::new(
                id.to_string(),
                1,
                AggregatePayload::V1 {
                    name: TENANT_NAME.to_string(),
                    items: Vec::new(),
                },
            )],
            expected_event_store_models: vec![EventStoreModel::new(
                0,
                id.to_string(),
                EventPayload::TenantCreatedV1 {
                    name: TENANT_NAME.to_string(),
                },
            )],
            expected_event_sequence_models: vec![EventSequenceModel::new(id.to_string(), 0)],
        }];
        for TestCase {
            name,
            aggregate,
            event,
            expected_aggregate_models,
            expected_event_store_models,
            expected_event_sequence_models,
        } in tests
        {
            let result = repository.create(aggregate, event).await;
            assert!(result.is_ok(), "{name}: result should be ok");
            let actual_aggregate_models: Vec<AggregateModel> = serde_dynamo::from_items(
                dynamodb
                    .scan()
                    .table_name(AGGREGATE_TABLE_NAME)
                    .send()
                    .await?
                    .items()
                    .to_vec(),
            )?;
            assert_eq!(actual_aggregate_models, expected_aggregate_models, "{name}");
            let actual_event_store_models: Vec<EventStoreModel> = serde_dynamo::from_items(
                dynamodb
                    .scan()
                    .table_name(EVENT_STORE_TABLE_NAME)
                    .send()
                    .await?
                    .items()
                    .to_vec(),
            )?;
            assert_eq!(
                actual_event_store_models, expected_event_store_models,
                "{name}"
            );
            let actual_event_sequence_models: Vec<EventSequenceModel> = serde_dynamo::from_items(
                dynamodb
                    .scan()
                    .table_name(EVENT_SEQUENCE_TABLE_NAME)
                    .send()
                    .await?
                    .items()
                    .to_vec(),
            )?;
            assert_eq!(
                actual_event_sequence_models, expected_event_sequence_models,
                "{name}"
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_create_err() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            event: Event,
            assert: fn(name: &str, actual: CommandKernelError),
        }

        let Context { repository, .. } = Context::without_container().await?;

        let tests = [TestCase {
            name: "作成イベント以外のイベントが渡された場合はInvalidEventが返る",
            aggregate: Aggregate::default(),
            event: Event::ItemsAdded { items: Vec::new() },
            assert: |name, actual| {
                assert!(
                    matches!(
                        actual,
                        CommandKernelError::ProcessorError(CommandProcessorError::InvalidEvent)
                    ),
                    "{name}"
                );
            },
        }];
        for TestCase {
            name,
            aggregate,
            event,
            assert,
        } in tests
        {
            let result = repository.create(aggregate, event).await;
            assert!(result.is_err(), "{name}: result mut be error");
            assert(name, result.err().unwrap());
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_with_container_create_err() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            fixture: Fixture,
            aggregate: Aggregate,
            event: Event,
            assert: fn(name: &str, actual: CommandKernelError),
        }

        let Context {
            container: _container,
            repository,
            dynamodb,
        } = Context::with_container().await?;

        let aggregate_id: Id<Aggregate> = Id::generate();

        let tests = [
            TestCase {
                name: "集約IDが存在する場合はUnknownが返る",
                fixture: Fixture {
                    aggregates: vec![AggregateModel::new(
                        aggregate_id.to_string(),
                        0,
                        AggregatePayload::V1 {
                            name: TENANT_NAME.to_string(),
                            items: Vec::new(),
                        },
                    )],
                    ..Default::default()
                },
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    TENANT_NAME.to_string(),
                    Vec::new(),
                    0,
                ),
                event: Event::Created {
                    name: TENANT_NAME.to_string(),
                },
                assert: |name, actual| {
                    assert!(matches!(actual, CommandKernelError::Unknown(_)), "{name}");
                },
            },
            TestCase {
                name: "イベントIDが存在する場合はUnknownが返る",
                fixture: Fixture {
                    event_stores: vec![EventStoreModel::new(
                        0,
                        aggregate_id.to_string(),
                        EventPayload::TenantCreatedV1 {
                            name: TENANT_NAME.to_string(),
                        },
                    )],
                    ..Default::default()
                },
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    TENANT_NAME.to_string(),
                    Vec::new(),
                    0,
                ),
                event: Event::Created {
                    name: TENANT_NAME.to_string(),
                },
                assert: |name, actual| {
                    assert!(matches!(actual, CommandKernelError::Unknown(_)), "{name}");
                },
            },
            TestCase {
                name: "EventSequenceに既にアイテムがある場合はUnknownが返る",
                fixture: Fixture {
                    event_sequences: vec![EventSequenceModel::new(aggregate_id.to_string(), 0)],
                    ..Default::default()
                },
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    TENANT_NAME.to_string(),
                    Vec::new(),
                    0,
                ),
                event: Event::Created {
                    name: TENANT_NAME.to_string(),
                },
                assert: |name, actual| {
                    assert!(matches!(actual, CommandKernelError::Unknown(_)), "{name}");
                },
            },
        ];
        for TestCase {
            name,
            fixture,
            aggregate,
            event,
            assert,
        } in tests
        {
            fixture.run(&dynamodb).await?;

            let result = repository.create(aggregate, event).await;
            assert!(result.is_err(), "{name}: result must be error");
            assert(name, result.err().unwrap());

            fixture.rollback(&dynamodb).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_with_container_get_ok() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            fixture: Fixture,
            id: Id<Aggregate>,
            expected: Option<Aggregate>,
        }

        let Context {
            container: _container,
            repository,
            dynamodb,
        } = Context::with_container().await?;

        let aggregate_id: Id<Aggregate> = Id::generate();
        let item_id: Id<kernel::Item> = Id::generate();

        let tests = [
            TestCase {
                name: "AggregateTableに指定したIDを持つ集約がある場合はその集約を返す",
                fixture: Fixture {
                    aggregates: vec![AggregateModel::new(
                        aggregate_id.to_string(),
                        2,
                        AggregatePayload::V1 {
                            name: TENANT_NAME.to_string(),
                            items: vec![Item::V1 {
                                id: item_id.to_string(),
                                name: ITEM_NAME.to_string(),
                                price: 1000,
                            }],
                        },
                    )],
                    ..Default::default()
                },
                id: aggregate_id.clone(),
                expected: Some(Aggregate::new(
                    aggregate_id.clone(),
                    TENANT_NAME.to_string(),
                    vec![kernel::Item::new(
                        item_id.clone(),
                        ITEM_NAME.to_string(),
                        1000,
                    )],
                    2,
                )),
            },
            TestCase {
                name: "AggregateTableに指定したIDを持つ集約がない場合はNoneを返す",
                fixture: Fixture {
                    ..Default::default()
                },
                id: Id::generate(),
                expected: None,
            },
        ];
        for TestCase {
            name,
            fixture,
            id,
            expected,
        } in tests
        {
            fixture.run(&dynamodb).await?;

            let result = repository.get(id).await;
            assert!(result.is_ok(), "{name}: result mut be ok");
            assert_eq!(result.unwrap(), expected);

            fixture.rollback(&dynamodb).await?;
        }
        Ok(())
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn test_with_container_update_ok() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            fixture: Fixture,
            aggregate: Aggregate,
            events: Vec<Event>,
            expected_aggregate_models: Vec<AggregateModel>,
            expected_event_store_models: Vec<EventStoreModel>,
            expected_event_sequence_models: Vec<EventSequenceModel>,
        }

        let Context {
            container: _container,
            repository,
            dynamodb,
        } = Context::with_container().await?;

        let aggregate_id: Id<Aggregate> = Id::generate();
        let item_id: Id<kernel::Item> = Id::generate();
        let item_id_2: Id<kernel::Item> = Id::generate();
        let tests = [
            TestCase {
                name: "商品追加イベントが発生した場合はそれぞれのテーブルに値が保存される",
                fixture: Fixture {
                    aggregates: vec![AggregateModel::new(
                        aggregate_id.to_string(),
                        1,
                        AggregatePayload::V1 {
                            name: TENANT_NAME.to_string(),
                            items: Vec::new(),
                        },
                    )],
                    event_stores: vec![EventStoreModel::new(
                        0,
                        aggregate_id.to_string(),
                        EventPayload::TenantCreatedV1 {
                            name: TENANT_NAME.to_string(),
                        },
                    )],
                    event_sequences: vec![EventSequenceModel::new(aggregate_id.to_string(), 0)],
                },
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    TENANT_NAME.to_string(),
                    vec![kernel::Item::new(
                        item_id.clone(),
                        ITEM_NAME.to_string(),
                        1000,
                    )],
                    2,
                ),
                events: vec![Event::ItemsAdded {
                    items: vec![kernel::Item::new(
                        item_id.clone(),
                        ITEM_NAME.to_string(),
                        1000,
                    )],
                }],
                expected_aggregate_models: vec![AggregateModel::new(
                    aggregate_id.to_string(),
                    2,
                    AggregatePayload::V1 {
                        name: TENANT_NAME.to_string(),
                        items: vec![Item::V1 {
                            id: item_id.to_string(),
                            name: ITEM_NAME.to_string(),
                            price: 1000,
                        }],
                    },
                )],
                expected_event_store_models: vec![
                    EventStoreModel::new(
                        0,
                        aggregate_id.to_string(),
                        EventPayload::TenantCreatedV1 {
                            name: TENANT_NAME.to_string(),
                        },
                    ),
                    EventStoreModel::new(
                        1,
                        aggregate_id.to_string(),
                        EventPayload::ItemsAddedV1 {
                            items: vec![Item::V1 {
                                id: item_id.to_string(),
                                name: ITEM_NAME.to_string(),
                                price: 1000,
                            }],
                        },
                    ),
                ],
                expected_event_sequence_models: vec![EventSequenceModel::new(
                    aggregate_id.to_string(),
                    1,
                )],
            },
            TestCase {
                name: "商品削除イベントが発生した場合はそれぞれのテーブルに値が保存される",
                fixture: Fixture {
                    aggregates: vec![AggregateModel::new(
                        aggregate_id.to_string(),
                        2,
                        AggregatePayload::V1 {
                            name: TENANT_NAME.to_string(),
                            items: vec![
                                Item::V1 {
                                    id: item_id.to_string(),
                                    name: ITEM_NAME.to_string(),
                                    price: 1000,
                                },
                                Item::V1 {
                                    id: item_id_2.to_string(),
                                    name: ITEM_NAME.to_string(),
                                    price: 2000,
                                },
                            ],
                        },
                    )],
                    event_stores: vec![
                        EventStoreModel::new(
                            0,
                            aggregate_id.to_string(),
                            EventPayload::TenantCreatedV1 {
                                name: TENANT_NAME.to_string(),
                            },
                        ),
                        EventStoreModel::new(
                            1,
                            aggregate_id.to_string(),
                            EventPayload::ItemsAddedV1 {
                                items: vec![
                                    Item::V1 {
                                        id: item_id.to_string(),
                                        name: ITEM_NAME.to_string(),
                                        price: 1000,
                                    },
                                    Item::V1 {
                                        id: item_id_2.to_string(),
                                        name: ITEM_NAME.to_string(),
                                        price: 2000,
                                    },
                                ],
                            },
                        ),
                    ],
                    event_sequences: vec![EventSequenceModel::new(aggregate_id.to_string(), 1)],
                },
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    TENANT_NAME.to_string(),
                    vec![kernel::Item::new(
                        item_id_2.clone(),
                        ITEM_NAME.to_string(),
                        2000,
                    )],
                    3,
                ),
                events: vec![Event::ItemsRemoved {
                    item_ids: vec![item_id.clone()],
                }],
                expected_aggregate_models: vec![AggregateModel::new(
                    aggregate_id.to_string(),
                    3,
                    AggregatePayload::V1 {
                        name: TENANT_NAME.to_string(),
                        items: vec![Item::V1 {
                            id: item_id_2.to_string(),
                            name: ITEM_NAME.to_string(),
                            price: 2000,
                        }],
                    },
                )],
                expected_event_store_models: vec![
                    EventStoreModel::new(
                        0,
                        aggregate_id.to_string(),
                        EventPayload::TenantCreatedV1 {
                            name: TENANT_NAME.to_string(),
                        },
                    ),
                    EventStoreModel::new(
                        1,
                        aggregate_id.to_string(),
                        EventPayload::ItemsAddedV1 {
                            items: vec![
                                Item::V1 {
                                    id: item_id.to_string(),
                                    name: ITEM_NAME.to_string(),
                                    price: 1000,
                                },
                                Item::V1 {
                                    id: item_id_2.to_string(),
                                    name: ITEM_NAME.to_string(),
                                    price: 2000,
                                },
                            ],
                        },
                    ),
                    EventStoreModel::new(
                        2,
                        aggregate_id.to_string(),
                        EventPayload::ItemsRemoved {
                            item_ids: vec![item_id.to_string()],
                        },
                    ),
                ],
                expected_event_sequence_models: vec![EventSequenceModel::new(
                    aggregate_id.to_string(),
                    2,
                )],
            },
        ];
        for TestCase {
            name,
            fixture,
            aggregate,
            events,
            expected_aggregate_models,
            expected_event_store_models,
            expected_event_sequence_models,
        } in tests
        {
            fixture.run(&dynamodb).await?;

            let result = repository.update(aggregate, events).await;
            assert!(result.is_ok(), "{name}: result must be ok: {result:?}");
            let actual_aggregate_models: Vec<AggregateModel> = serde_dynamo::from_items(
                dynamodb
                    .scan()
                    .table_name(AGGREGATE_TABLE_NAME)
                    .send()
                    .await?
                    .items()
                    .to_vec(),
            )?;
            assert_eq!(actual_aggregate_models, expected_aggregate_models, "{name}");
            let actual_event_store_models: Vec<EventStoreModel> = serde_dynamo::from_items(
                dynamodb
                    .scan()
                    .table_name(EVENT_STORE_TABLE_NAME)
                    .send()
                    .await?
                    .items()
                    .to_vec(),
            )?;
            assert_eq!(
                actual_event_store_models, expected_event_store_models,
                "{name}"
            );
            let actual_event_sequence_models: Vec<EventSequenceModel> = serde_dynamo::from_items(
                dynamodb
                    .scan()
                    .table_name(EVENT_SEQUENCE_TABLE_NAME)
                    .send()
                    .await?
                    .items()
                    .to_vec(),
            )?;
            assert_eq!(
                actual_event_sequence_models, expected_event_sequence_models,
                "{name}"
            );

            fixture.rollback(&dynamodb).await?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_with_container_update_err() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            fixture: Fixture,
            aggregate: Aggregate,
            events: Vec<Event>,
            assert: fn(name: &str, actual: CommandKernelError),
        }

        let Context {
            container: _container,
            repository,
            dynamodb,
        } = Context::with_container().await?;

        let aggregate_id: Id<Aggregate> = Id::generate();
        let tests = [
            TestCase {
                name: "集約バージョンがコンフリクトした場合はUnknownが返る",
                fixture: Fixture {
                    aggregates: vec![AggregateModel::new(
                        aggregate_id.to_string(),
                        2,
                        AggregatePayload::V1 {
                            name: TENANT_NAME.to_string(),
                            items: Vec::new(),
                        },
                    )],
                    event_sequences: vec![EventSequenceModel::new(aggregate_id.to_string(), 0)],
                    ..Default::default()
                },
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    TENANT_NAME.to_string(),
                    Vec::new(),
                    2,
                ),
                events: vec![Event::ItemsAdded { items: Vec::new() }],
                assert: |name, actual| {
                    assert!(matches!(actual, CommandKernelError::Unknown(_)), "{name}");
                },
            },
            TestCase {
                name: "指定された集約IDが存在しない場合はUnknownが返る",
                fixture: Fixture {
                    ..Default::default()
                },
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    TENANT_NAME.to_string(),
                    Vec::new(),
                    2,
                ),
                events: vec![Event::ItemsAdded { items: Vec::new() }],
                assert: |name, actual| {
                    assert!(matches!(actual, CommandKernelError::Unknown(_)), "{name}");
                },
            },
        ];
        for TestCase {
            name,
            fixture,
            aggregate,
            events,
            assert,
        } in tests
        {
            fixture.run(&dynamodb).await?;

            let result = repository.update(aggregate, events).await;
            assert!(result.is_err(), "{name}: result must be err");
            assert(name, result.err().unwrap());

            fixture.rollback(&dynamodb).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_update_err() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            events: Vec<Event>,
            assert: fn(name: &str, actual: CommandKernelError),
        }

        let Context { repository, .. } = Context::without_container().await?;

        let aggregate_id: Id<Aggregate> = Id::generate();
        let tests = [
            TestCase {
                name: "イベントが空の場合はEmptyEventが返る",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    TENANT_NAME.to_string(),
                    Vec::new(),
                    2,
                ),
                events: Vec::new(),
                assert: |name, actual| {
                    assert!(
                        matches!(
                            actual,
                            CommandKernelError::ProcessorError(CommandProcessorError::EmptyEvent)
                        ),
                        "{name}: actual: {actual:?}"
                    );
                },
            },
            TestCase {
                name: "イベントにCreatedが含まれる場合はInvalidEventが返る",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    TENANT_NAME.to_string(),
                    Vec::new(),
                    2,
                ),
                events: vec![
                    Event::ItemsAdded { items: Vec::new() },
                    Event::Created {
                        name: String::new(),
                    },
                    Event::ItemsRemoved {
                        item_ids: Vec::new(),
                    },
                ],
                assert: |name, actual| {
                    assert!(
                        matches!(
                            actual,
                            CommandKernelError::ProcessorError(CommandProcessorError::InvalidEvent)
                        ),
                        "{name}: actual: {actual:?}"
                    );
                },
            },
        ];
        for TestCase {
            name,
            aggregate,
            events,
            assert,
        } in tests
        {
            let result = repository.update(aggregate, events).await;
            assert!(result.is_err(), "{name}: result must be err");
            assert(name, result.err().unwrap());
        }
        Ok(())
    }

    struct Context {
        container: Option<ContainerAsync<DynamoDb>>,
        repository: CommandRepository,
        dynamodb: aws_sdk_dynamodb::Client,
    }

    impl Context {
        async fn with_container() -> Result<Self, Error> {
            use aws_config::BehaviorVersion;
            use aws_sdk_dynamodb::types::{
                AttributeDefinition, BillingMode, KeySchemaElement, KeyType, ScalarAttributeType,
            };
            use testcontainers::runners::AsyncRunner;

            // NOTE: デフォルトのDockerコンテキスト以外を使っている場合にtestcontainersが正しく動作しないため、
            // 環境変数の `DOCKER_HOST` にホストを設定する必要がある
            // read mores: https://github.com/testcontainers/testcontainers-rs/issues/627
            option_env!("DOCKER_HOST")
                .unwrap_or_else(|| panic!("DOCKER_HOST must be set (e.g. DOCKER_HOST=(docker context inspect | jq -r '.[0].Endpoints.docker.Host'))"));

            let container = DynamoDb::default().start().await?;
            let endpoint = format!(
                "http://{}:{}",
                container.get_host().await?,
                container.get_host_port_ipv4(8000).await?,
            );
            let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
                .endpoint_url(endpoint)
                .test_credentials()
                .load()
                .await;
            let dynamodb = aws_sdk_dynamodb::Client::new(&config);
            let repository = CommandRepository::new(dynamodb.clone());

            dynamodb
                .create_table()
                .table_name(EVENT_STORE_TABLE_NAME)
                .attribute_definitions(
                    AttributeDefinition::builder()
                        .attribute_name("aggregate_id")
                        .attribute_type(ScalarAttributeType::S)
                        .build()?,
                )
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("aggregate_id")
                        .key_type(KeyType::Hash)
                        .build()?,
                )
                .attribute_definitions(
                    AttributeDefinition::builder()
                        .attribute_name("id")
                        .attribute_type(ScalarAttributeType::N)
                        .build()?,
                )
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("id")
                        .key_type(KeyType::Range)
                        .build()?,
                )
                .billing_mode(BillingMode::PayPerRequest)
                .send()
                .await?;
            dynamodb
                .create_table()
                .table_name(EVENT_SEQUENCE_TABLE_NAME)
                .attribute_definitions(
                    AttributeDefinition::builder()
                        .attribute_name("aggregate_id")
                        .attribute_type(ScalarAttributeType::S)
                        .build()?,
                )
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("aggregate_id")
                        .key_type(KeyType::Hash)
                        .build()?,
                )
                .billing_mode(BillingMode::PayPerRequest)
                .send()
                .await?;
            dynamodb
                .create_table()
                .table_name(AGGREGATE_TABLE_NAME)
                .attribute_definitions(
                    AttributeDefinition::builder()
                        .attribute_name("id")
                        .attribute_type(ScalarAttributeType::S)
                        .build()?,
                )
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("id")
                        .key_type(KeyType::Hash)
                        .build()?,
                )
                .billing_mode(BillingMode::PayPerRequest)
                .send()
                .await?;

            Ok(Self {
                container: Some(container),
                repository,
                dynamodb,
            })
        }

        async fn without_container() -> Result<Self, Error> {
            use aws_config::BehaviorVersion;

            let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
                .test_credentials()
                .load()
                .await;
            let dynamodb = aws_sdk_dynamodb::Client::new(&config);
            let repository = CommandRepository::new(dynamodb.clone());
            Ok(Self {
                container: None,
                repository,
                dynamodb,
            })
        }
    }

    #[derive(Default)]
    struct Fixture {
        aggregates: Vec<AggregateModel>,
        event_stores: Vec<EventStoreModel>,
        event_sequences: Vec<EventSequenceModel>,
    }

    impl Fixture {
        async fn run(&self, dynamodb: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
            for item in self.aggregates.clone() {
                dynamodb
                    .put_item()
                    .table_name(AGGREGATE_TABLE_NAME)
                    .set_item(Some(item.try_into()?))
                    .send()
                    .await?;
            }
            for item in self.event_stores.clone() {
                dynamodb
                    .put_item()
                    .table_name(EVENT_STORE_TABLE_NAME)
                    .set_item(Some(item.try_into()?))
                    .send()
                    .await?;
            }
            for item in self.event_sequences.clone() {
                dynamodb
                    .put_item()
                    .table_name(EVENT_SEQUENCE_TABLE_NAME)
                    .set_item(Some(item.try_into()?))
                    .send()
                    .await?;
            }
            Ok(())
        }

        async fn rollback(&self, dynamodb: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
            use aws_sdk_dynamodb::types::AttributeValue;

            let aggregates: Vec<AggregateModel> = serde_dynamo::from_items(
                dynamodb
                    .scan()
                    .table_name(AGGREGATE_TABLE_NAME)
                    .send()
                    .await?
                    .items()
                    .to_vec(),
            )?;
            for aggregate in aggregates {
                dynamodb
                    .delete_item()
                    .table_name(AGGREGATE_TABLE_NAME)
                    .key("id", AttributeValue::S(aggregate.id().to_string()))
                    .send()
                    .await?;
            }
            let event_stores: Vec<EventStoreModel> = serde_dynamo::from_items(
                dynamodb
                    .scan()
                    .table_name(EVENT_STORE_TABLE_NAME)
                    .send()
                    .await?
                    .items()
                    .to_vec(),
            )?;
            for event_store in event_stores {
                dynamodb
                    .delete_item()
                    .table_name(EVENT_STORE_TABLE_NAME)
                    .key(
                        "aggregate_id",
                        AttributeValue::S(event_store.aggregate_id().to_string()),
                    )
                    .key("id", AttributeValue::N(event_store.id().to_string()))
                    .send()
                    .await?;
            }
            let event_sequences: Vec<EventSequenceModel> = serde_dynamo::from_items(
                dynamodb
                    .scan()
                    .table_name(EVENT_SEQUENCE_TABLE_NAME)
                    .send()
                    .await?
                    .items()
                    .to_vec(),
            )?;
            for event_sequence in event_sequences {
                dynamodb
                    .delete_item()
                    .table_name(EVENT_SEQUENCE_TABLE_NAME)
                    .key(
                        "aggregate_id",
                        AttributeValue::S(event_sequence.aggregate_id().to_string()),
                    )
                    .send()
                    .await?;
            }
            Ok(())
        }
    }
}
