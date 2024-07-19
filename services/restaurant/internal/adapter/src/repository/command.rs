use aws_sdk_dynamodb::types::{AttributeValue, Put, TransactWriteItem, Update};
use kernel::{Aggregate, CommandProcessor, Event, EventPayload, Id, KernelError};

use crate::{AggregateModel, EventModel};

const AGGREGATE_TABLE_NAME: &str = "restaurant-aggregate";
const EVENT_TABLE_NAME: &str = "restaurant-event";

#[derive(Debug, Clone)]
pub struct CommandRepository {
    dynamodb: aws_sdk_dynamodb::Client,
}

impl CommandRepository {
    #[must_use]
    pub fn new(dynamodb: aws_sdk_dynamodb::Client) -> Self {
        Self { dynamodb }
    }
}

impl CommandProcessor for CommandRepository {
    async fn create(&self, aggregate: Aggregate, events: Vec<Event>) -> Result<(), KernelError> {
        if events.is_empty() {
            return Err(KernelError::EmptyEvent);
        }
        if let Some(event) = events.first() {
            if !matches!(event.payload(), EventPayload::AggregateCreated(..)) {
                return Err(KernelError::InvalidEvents);
            }
        }

        let mut transact_items = Vec::new();
        for event in events {
            transact_items.push(
                TransactWriteItem::builder()
                    .put(
                        Put::builder()
                            .table_name(EVENT_TABLE_NAME)
                            .set_item(Some(
                                EventModel::new(
                                    event.id(),
                                    aggregate.id(),
                                    event.payload().clone(),
                                )
                                .to_item()?,
                            ))
                            .condition_expression(
                                "attribute_not_exists(id) AND attribute_not_exists(aggregate_id)",
                            )
                            .build()
                            .map_err(|e| KernelError::Unknown(e.into()))?,
                    )
                    .build(),
            );
        }
        transact_items.push(
            TransactWriteItem::builder()
                .put(
                    Put::builder()
                        .table_name(AGGREGATE_TABLE_NAME)
                        .set_item(Some(AggregateModel::new(aggregate).to_item()?))
                        .condition_expression("attribute_not_exists(id)")
                        .build()
                        .map_err(|e| KernelError::Unknown(e.into()))?,
                )
                .build(),
        );
        self.dynamodb
            .transact_write_items()
            .set_transact_items(Some(transact_items))
            .send()
            .await
            .map_err(|e| KernelError::Unknown(e.into()))?;

        Ok(())
    }

    async fn get(&self, id: Id<Aggregate>) -> Result<Aggregate, KernelError> {
        let result = self
            .dynamodb
            .get_item()
            .table_name(AGGREGATE_TABLE_NAME)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await;
        if let Err(e) = result {
            match e.into_service_error() {
                aws_sdk_dynamodb::operation::get_item::GetItemError::ResourceNotFoundException(
                    _,
                ) => return Err(KernelError::AggregateNotFound),
                e => return Err(KernelError::Unknown(e.into())),
            }
        }
        let model: AggregateModel = serde_dynamo::from_item(
            result
                .unwrap()
                .item()
                .ok_or_else(|| KernelError::AggregateNotFound)?
                .clone(),
        )
        .map_err(|e| KernelError::Unknown(e.into()))?;

        Ok(model.try_into()?)
    }

    async fn update(&self, aggregate: Aggregate, events: Vec<Event>) -> Result<(), KernelError> {
        if events.is_empty() {
            return Err(KernelError::EmptyEvent);
        }
        if events
            .iter()
            .any(|e| matches!(e.payload(), EventPayload::AggregateCreated(..)))
        {
            return Err(KernelError::InvalidEvents);
        }

        let mut transact_items = Vec::new();
        for event in events {
            transact_items.push(
                TransactWriteItem::builder()
                    .put(
                        Put::builder()
                            .table_name(EVENT_TABLE_NAME)
                            .set_item(Some(
                                EventModel::new(
                                    event.id(),
                                    aggregate.id(),
                                    event.payload().clone(),
                                )
                                .to_item()?,
                            ))
                            .condition_expression(
                                "attribute_not_exists(id) AND attribute_not_exists(aggregate_id)",
                            )
                            .build()
                            .map_err(|e| KernelError::Unknown(e.into()))?,
                    )
                    .build(),
            );
        }
        let aggregate_model = AggregateModel::new(aggregate);
        transact_items.push(
            TransactWriteItem::builder()
                .update(
                    Update::builder()
                        .table_name(AGGREGATE_TABLE_NAME)
                        .key("id", aggregate_model.id_attribute_value()?)
                        .expression_attribute_values(
                            ":new_payload",
                            aggregate_model.payload_attribute_value()?,
                        )
                        .expression_attribute_values(
                            ":new_version",
                            aggregate_model.version_attribute_value()?,
                        )
                        .update_expression("SET payload = :new_payload, version = :new_version")
                        .expression_attribute_values(
                            ":current_version",
                            serde_dynamo::to_attribute_value(
                                aggregate_model.version().saturating_sub(1),
                            )
                            .map_err(|e| KernelError::Unknown(e.into()))?,
                        )
                        .condition_expression("attribute_exists(id) AND version = :current_version")
                        .build()
                        .map_err(|e| KernelError::Unknown(e.into()))?,
                )
                .build(),
        );
        self.dynamodb
            .transact_write_items()
            .set_transact_items(Some(transact_items))
            .send()
            .await
            .map_err(|e| KernelError::Unknown(e.into()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::too_many_lines)]

    use aws_config::BehaviorVersion;
    use aws_sdk_dynamodb::types::{
        AttributeDefinition, BillingMode, KeySchemaElement, KeyType, ProvisionedThroughput,
    };
    use kernel::{
        Aggregate, CommandProcessor, Event, EventPayload, Id, Item, ItemCategory, KernelError,
        Price, Restaurant,
    };
    use testcontainers::runners::AsyncRunner;
    use testcontainers::ContainerAsync;
    use testcontainers_modules::dynamodb_local::DynamoDb;

    use crate::{AggregateModel, CommandRepository, EventModel};

    use super::{AGGREGATE_TABLE_NAME, EVENT_TABLE_NAME};

    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    const RESTAURANT_NAME: &str = "テスト店舗";
    const ITEM_NAME: &str = "テスト商品";

    #[tokio::test]
    async fn test_create_ok() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            events: Vec<Event>,
            expected_aggregate_models: Vec<AggregateModel>,
            expected_event_models: Vec<EventModel>,
        }

        let Prepare {
            container: _container,
            repository,
            dynamodb,
        } = Prepare::with_container().await?;

        let aggregate_id: Id<Aggregate> = Id::generate();
        let restaurant_id: Id<Restaurant> = Id::generate();
        let event_id: Id<Event> = Id::generate();

        let tests = [TestCase {
            name:
                "引数に集約と集約作成イベントが渡された場合、それぞれが対応するテーブルに保存される",
            aggregate: Aggregate::new(
                aggregate_id.clone(),
                Restaurant::new(restaurant_id.clone(), RESTAURANT_NAME.to_string()),
                vec![],
                1,
            ),
            events: vec![Event::new(
                event_id.clone(),
                EventPayload::AggregateCreated(Restaurant::new(
                    restaurant_id.clone(),
                    RESTAURANT_NAME.to_string(),
                )),
            )],
            expected_aggregate_models: vec![AggregateModel::new(Aggregate::new(
                aggregate_id.clone(),
                Restaurant::new(restaurant_id.clone(), RESTAURANT_NAME.to_string()),
                vec![],
                1,
            ))],
            expected_event_models: vec![EventModel::new(
                &event_id,
                &aggregate_id,
                EventPayload::AggregateCreated(Restaurant::new(
                    restaurant_id.clone(),
                    RESTAURANT_NAME.to_string(),
                )),
            )],
        }];
        for TestCase {
            name,
            aggregate,
            events,
            expected_aggregate_models,
            expected_event_models,
        } in tests
        {
            let result = repository.create(aggregate, events).await;
            assert!(result.is_ok(), "{name}: result.is_ok()");
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
            let actual_event_models: Vec<EventModel> = serde_dynamo::from_items(
                dynamodb
                    .scan()
                    .table_name(EVENT_TABLE_NAME)
                    .send()
                    .await?
                    .items()
                    .to_vec(),
            )?;
            assert_eq!(actual_event_models, expected_event_models, "{name}");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_create_err() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            fixture: Fixture,
            aggregate: Aggregate,
            events: Vec<Event>,
            assert: fn(name: &str, actual: Result<(), KernelError>),
        }

        let Prepare {
            container: _container,
            repository,
            dynamodb,
        } = Prepare::with_container().await?;

        let aggregate_id: Id<Aggregate> = Id::generate();
        let event_id: Id<Event> = Id::generate();

        let tests = [
            TestCase {
                name: "集約Idが既に存在する場合はUnknownエラーが返る",
                fixture: Fixture::new(
                    vec![AggregateModel::new(Aggregate::new(
                        aggregate_id.clone(),
                        Restaurant::new(Id::generate(), String::new()),
                        vec![],
                        1,
                    ))],
                    vec![],
                ),
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Restaurant::new(Id::generate(), String::new()),
                    vec![],
                    1,
                ),
                events: vec![Event::new(
                    Id::generate(),
                    EventPayload::AggregateCreated(Restaurant::new(Id::generate(), String::new())),
                )],
                assert: |name, actual| {
                    assert!(matches!(actual, Err(KernelError::Unknown(_))), "{name}");
                },
            },
            TestCase {
                name: "イベントIdが既に存在する場合はUnknownエラーが返る",
                fixture: Fixture::new(
                    vec![],
                    vec![EventModel::new(
                        &event_id,
                        &aggregate_id,
                        EventPayload::AggregateCreated(Restaurant::new(
                            Id::generate(),
                            String::new(),
                        )),
                    )],
                ),
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Restaurant::new(Id::generate(), String::new()),
                    vec![],
                    1,
                ),
                events: vec![Event::new(
                    event_id.clone(),
                    EventPayload::AggregateCreated(Restaurant::new(Id::generate(), String::new())),
                )],
                assert: |name, actual| {
                    assert!(matches!(actual, Err(KernelError::Unknown(_))), "{name}");
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

            let actual = repository.create(aggregate, events).await;
            assert(name, actual);

            fixture.rollback(&dynamodb).await?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_validation_err() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            events: Vec<Event>,
            assert: fn(name: &str, actual: Result<(), KernelError>),
        }

        let Prepare { repository, .. } = Prepare::without_container().await;

        let tests = [
            TestCase {
                name: "eventsが空配列の場合はEmptyEventエラーが返る",
                aggregate: Aggregate::default(),
                events: vec![],
                assert: |name, actual| {
                    assert!(matches!(actual, Err(KernelError::EmptyEvent)), "{name}");
                },
            },
            TestCase {
                name: "eventsの最初の要素が集約作成イベントではない場合はInvalidEventsエラーが返る",
                aggregate: Aggregate::default(),
                events: vec![Event::new(
                    Id::generate(),
                    EventPayload::ItemsRemoved(vec![Id::generate()]),
                )],
                assert: |name, actual| {
                    assert!(matches!(actual, Err(KernelError::InvalidEvents)), "{name}");
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
            let actual = repository.create(aggregate, events).await;
            assert(name, actual);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_get_ok() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            fixture: Fixture,
            id: Id<Aggregate>,
            expected: Aggregate,
        }

        let Prepare {
            container: _container,
            repository,
            dynamodb,
        } = Prepare::with_container().await?;

        let aggregate_id: Id<Aggregate> = Id::generate();
        let restaurant_id: Id<Restaurant> = Id::generate();
        let item_id: Id<Item> = Id::generate();

        let tests = [TestCase {
            name: "Aggregateテーブルに指定したIdを持つ集約があれば、その集約を返す",
            fixture: Fixture::new(
                vec![AggregateModel::new(Aggregate::new(
                    aggregate_id.clone(),
                    Restaurant::new(restaurant_id.clone(), RESTAURANT_NAME.to_string()),
                    vec![
                        Item::new(
                            item_id.clone(),
                            ITEM_NAME.to_string(),
                            Price::Yen(1000),
                            ItemCategory::Food,
                        ),
                        Item::new(
                            item_id.clone(),
                            ITEM_NAME.to_string(),
                            Price::Yen(1000),
                            ItemCategory::Drink,
                        ),
                    ],
                    2,
                ))],
                vec![],
            ),
            id: aggregate_id.clone(),
            expected: Aggregate::new(
                aggregate_id.clone(),
                Restaurant::new(restaurant_id.clone(), RESTAURANT_NAME.to_string()),
                vec![
                    Item::new(
                        item_id.clone(),
                        ITEM_NAME.to_string(),
                        Price::Yen(1000),
                        ItemCategory::Food,
                    ),
                    Item::new(
                        item_id.clone(),
                        ITEM_NAME.to_string(),
                        Price::Yen(1000),
                        ItemCategory::Drink,
                    ),
                ],
                2,
            ),
        }];
        for TestCase {
            name,
            fixture,
            id,
            expected,
        } in tests
        {
            fixture.run(&dynamodb).await?;

            let result = repository.get(id).await;
            assert!(result.is_ok(), "{name}: result.is_ok()");
            assert_eq!(result.unwrap(), expected, "{name}");

            fixture.rollback(&dynamodb).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_get_err() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            fixture: Fixture,
            id: Id<Aggregate>,
            assert: fn(name: &str, actual: Result<Aggregate, KernelError>),
        }

        let Prepare {
            container: _container,
            repository,
            dynamodb,
        } = Prepare::with_container().await?;

        let tests = [TestCase {
            name: "Aggregateテーブルに指定したIdを持つ集約がない場合,AggregateNotFoundエラーが返る",
            fixture: Fixture::new(vec![], vec![]),
            id: Id::generate(),
            assert: |name, actual| {
                assert!(
                    matches!(actual, Err(KernelError::AggregateNotFound)),
                    "{name}"
                );
            },
        }];
        for TestCase {
            name,
            fixture,
            id,
            assert,
        } in tests
        {
            fixture.run(&dynamodb).await?;

            let actual = repository.get(id).await;
            assert(name, actual);

            fixture.rollback(&dynamodb).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_update_ok() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            fixture: Fixture,
            aggregate: Aggregate,
            events: Vec<Event>,
            expected_aggregate_models: Vec<AggregateModel>,
            expected_event_models: Vec<EventModel>,
        }

        let Prepare {
            container: _container,
            repository,
            dynamodb,
        } = Prepare::with_container().await?;

        let aggregate_id: Id<Aggregate> = Id::generate();
        let restaurant_id: Id<Restaurant> = Id::generate();
        let item_id: Id<Item> = Id::generate();
        let event_id: Id<Event> = Id::generate();

        let tests = [TestCase {
            name: "集約バージョンがコンフリクトしない場合は、正しく集約とイベントが保存される",
            fixture: Fixture::new(
                vec![AggregateModel::new(Aggregate::new(
                    aggregate_id.clone(),
                    Restaurant::new(restaurant_id.clone(), RESTAURANT_NAME.to_string()),
                    vec![],
                    1,
                ))],
                vec![],
            ),
            aggregate: Aggregate::new(
                aggregate_id.clone(),
                Restaurant::new(restaurant_id.clone(), RESTAURANT_NAME.to_string()),
                vec![Item::new(
                    item_id.clone(),
                    ITEM_NAME.to_string(),
                    Price::Yen(1000),
                    ItemCategory::Food,
                )],
                2,
            ),
            events: vec![Event::new(
                event_id.clone(),
                EventPayload::ItemsAdded(vec![Item::new(
                    item_id.clone(),
                    ITEM_NAME.to_string(),
                    Price::Yen(1000),
                    ItemCategory::Food,
                )]),
            )],
            expected_aggregate_models: vec![AggregateModel::new(Aggregate::new(
                aggregate_id.clone(),
                Restaurant::new(restaurant_id.clone(), RESTAURANT_NAME.to_string()),
                vec![Item::new(
                    item_id.clone(),
                    ITEM_NAME.to_string(),
                    Price::Yen(1000),
                    ItemCategory::Food,
                )],
                2,
            ))],
            expected_event_models: vec![EventModel::new(
                &event_id,
                &aggregate_id,
                EventPayload::ItemsAdded(vec![Item::new(
                    item_id.clone(),
                    ITEM_NAME.to_string(),
                    Price::Yen(1000),
                    ItemCategory::Food,
                )]),
            )],
        }];

        for TestCase {
            name,
            fixture,
            aggregate,
            events,
            expected_aggregate_models,
            expected_event_models,
        } in tests
        {
            fixture.run(&dynamodb).await?;

            repository.update(aggregate, events).await?;
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
            let actual_event_models: Vec<EventModel> = serde_dynamo::from_items(
                dynamodb
                    .scan()
                    .table_name(EVENT_TABLE_NAME)
                    .send()
                    .await?
                    .items()
                    .to_vec(),
            )?;
            assert_eq!(actual_event_models, expected_event_models, "{name}");

            fixture.rollback(&dynamodb).await?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_err() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            fixture: Fixture,
            aggregate: Aggregate,
            events: Vec<Event>,
            assert: fn(name: &str, actual: Result<(), KernelError>),
        }

        let Prepare {
            container: _container,
            repository,
            dynamodb,
        } = Prepare::with_container().await?;

        let aggregate_id: Id<Aggregate> = Id::generate();
        let restaurant_id: Id<Restaurant> = Id::generate();
        let item_id: Id<Item> = Id::generate();
        let event_id: Id<Event> = Id::generate();

        let tests = [
            TestCase {
                name: "集約バージョンがコンフリクトした場合、Unknownエラーが返る",
                fixture: Fixture::new(
                    vec![AggregateModel::new(Aggregate::new(
                        aggregate_id.clone(),
                        Restaurant::new(restaurant_id.clone(), RESTAURANT_NAME.to_string()),
                        vec![],
                        2,
                    ))],
                    vec![],
                ),
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Restaurant::new(restaurant_id.clone(), RESTAURANT_NAME.to_string()),
                    vec![],
                    2,
                ),
                events: vec![Event::new(
                    event_id.clone(),
                    EventPayload::ItemsAdded(vec![Item::new(
                        item_id.clone(),
                        ITEM_NAME.to_string(),
                        Price::Yen(1000),
                        ItemCategory::Food,
                    )]),
                )],
                assert: |name, actual| {
                    assert!(matches!(actual, Err(KernelError::Unknown(_))), "{name}");
                },
            },
            TestCase {
                name: "指定された集約のIdがない場合は、Unknownエラーが返る",
                fixture: Fixture::new(vec![], vec![]),
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Restaurant::new(restaurant_id.clone(), RESTAURANT_NAME.to_string()),
                    vec![],
                    2,
                ),
                events: vec![Event::new(
                    event_id.clone(),
                    EventPayload::ItemsAdded(vec![Item::new(
                        item_id.clone(),
                        ITEM_NAME.to_string(),
                        Price::Yen(1000),
                        ItemCategory::Food,
                    )]),
                )],
                assert: |name, actual| {
                    assert!(matches!(actual, Err(KernelError::Unknown(_))), "{name}");
                },
            },
            TestCase {
                name: "イベントIdが重複する場合、Unknownエラーが返る",
                fixture: Fixture::new(
                    vec![],
                    vec![EventModel::new(
                        &event_id,
                        &aggregate_id,
                        EventPayload::ItemsRemoved(vec![Id::generate()]),
                    )],
                ),
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Restaurant::new(restaurant_id.clone(), RESTAURANT_NAME.to_string()),
                    vec![],
                    2,
                ),
                events: vec![Event::new(
                    event_id.clone(),
                    EventPayload::ItemsAdded(vec![Item::new(
                        item_id.clone(),
                        ITEM_NAME.to_string(),
                        Price::Yen(1000),
                        ItemCategory::Food,
                    )]),
                )],
                assert: |name, actual| {
                    assert!(matches!(actual, Err(KernelError::Unknown(_))), "{name}");
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
            assert(name, result);

            fixture.rollback(&dynamodb).await?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_validation_err() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            events: Vec<Event>,
            assert: fn(name: &str, actual: Result<(), KernelError>),
        }

        let Prepare { repository, .. } = Prepare::without_container().await;

        let tests = [
            TestCase {
                name: "eventsが空配列の場合はEmptyEventエラーが返る",
                aggregate: Aggregate::default(),
                events: vec![],
                assert: |name, actual| {
                    assert!(matches!(actual, Err(KernelError::EmptyEvent)), "{name}");
                },
            },
            TestCase {
                name: "eventsに集約作成イベントが含まれる場合はInvalidEventsエラーが返る",
                aggregate: Aggregate::default(),
                events: vec![
                    Event::new(
                        Id::generate(),
                        EventPayload::ItemsRemoved(vec![Id::generate()]),
                    ),
                    Event::new(
                        Id::generate(),
                        EventPayload::AggregateCreated(Restaurant::new(
                            Id::generate(),
                            String::new(),
                        )),
                    ),
                ],
                assert: |name, actual| {
                    assert!(matches!(actual, Err(KernelError::InvalidEvents)), "{name}");
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
            let actual = repository.create(aggregate, events).await;
            assert(name, actual);
        }
        Ok(())
    }

    struct Prepare {
        container: Option<ContainerAsync<DynamoDb>>,
        repository: CommandRepository,
        dynamodb: aws_sdk_dynamodb::Client,
    }

    impl Prepare {
        async fn with_container() -> Result<Self, Error> {
            // NOTE: デフォルトのDockerコンテキスト以外を使っている場合にtestcontainersが正しく動作しないため、
            // 環境変数の `DOCKER_HOST` にホストを設定する必要がある
            // read mores: https://github.com/testcontainers/testcontainers-rs/issues/627
            option_env!("DOCKER_HOST").unwrap_or_else(|| panic!("DOCKER_HOST must be set"));

            let container = DynamoDb.start().await?;
            let endpoint = format!(
                "http://{}:{}",
                container.get_host().await?,
                container.get_host_port_ipv4(8000).await?
            );
            let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
                .endpoint_url(endpoint)
                .test_credentials()
                .load()
                .await;
            let dynamodb = aws_sdk_dynamodb::Client::new(&config);

            dynamodb
                .create_table()
                .table_name(AGGREGATE_TABLE_NAME)
                .attribute_definitions(
                    AttributeDefinition::builder()
                        .attribute_name("id")
                        .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
                        .build()?,
                )
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("id")
                        .key_type(KeyType::Hash)
                        .build()?,
                )
                .billing_mode(BillingMode::Provisioned)
                .provisioned_throughput(
                    ProvisionedThroughput::builder()
                        .read_capacity_units(20)
                        .write_capacity_units(20)
                        .build()?,
                )
                .send()
                .await?;
            dynamodb
                .create_table()
                .table_name(EVENT_TABLE_NAME)
                .attribute_definitions(
                    AttributeDefinition::builder()
                        .attribute_name("aggregate_id")
                        .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
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
                        .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
                        .build()?,
                )
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("id")
                        .key_type(KeyType::Range)
                        .build()?,
                )
                .billing_mode(BillingMode::Provisioned)
                .provisioned_throughput(
                    ProvisionedThroughput::builder()
                        .read_capacity_units(20)
                        .write_capacity_units(20)
                        .build()?,
                )
                .send()
                .await?;

            Ok(Self {
                container: Some(container),
                repository: CommandRepository::new(dynamodb.clone()),
                dynamodb,
            })
        }

        async fn without_container() -> Self {
            let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
                .endpoint_url("http://localhost:8000")
                .test_credentials()
                .load()
                .await;
            let dynamodb = aws_sdk_dynamodb::Client::new(&config);
            Self {
                container: None,
                repository: CommandRepository::new(dynamodb.clone()),
                dynamodb,
            }
        }
    }

    struct Fixture {
        aggregates: Vec<AggregateModel>,
        events: Vec<EventModel>,
    }

    impl Fixture {
        fn new(aggregates: Vec<AggregateModel>, events: Vec<EventModel>) -> Self {
            Self { aggregates, events }
        }

        async fn run(&self, dynamodb: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
            for aggregate in &self.aggregates {
                dynamodb
                    .put_item()
                    .table_name(AGGREGATE_TABLE_NAME)
                    .set_item(Some(aggregate.to_item()?))
                    .send()
                    .await?;
            }
            for event in &self.events {
                dynamodb
                    .put_item()
                    .table_name(EVENT_TABLE_NAME)
                    .set_item(Some(event.to_item()?))
                    .send()
                    .await?;
            }
            Ok(())
        }

        async fn rollback(&self, dynamodb: &aws_sdk_dynamodb::Client) -> Result<(), Error> {
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
                    .key("id", aggregate.id_attribute_value()?)
                    .send()
                    .await?;
            }
            let events: Vec<EventModel> = serde_dynamo::from_items(
                dynamodb
                    .scan()
                    .table_name(EVENT_TABLE_NAME)
                    .send()
                    .await?
                    .items()
                    .to_vec(),
            )?;
            for event in events {
                dynamodb
                    .delete_item()
                    .table_name(EVENT_TABLE_NAME)
                    .key("id", event.id_attribute_value()?)
                    .key("aggregate_id", event.aggregate_id_attribute_value()?)
                    .send()
                    .await?;
            }
            Ok(())
        }
    }
}
