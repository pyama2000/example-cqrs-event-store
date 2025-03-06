use anyhow::Context as _;
use aws_sdk_dynamodb::types::TransactWriteItem;

use crate::command::model::{AggregateModel, EventSequenceModel};
use crate::{AGGREGATE_TABLE_NAME, EVENT_SEQUENCE_TABLE_NAME, EVENT_STORE_TABLE_NAME};

use super::model::EventStoreModel;

/// コマンド操作を行うリポジトリ
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

impl kernel::command::processor::CommandProcessor for CommandRepository {
    async fn create(
        &self,
        aggregate: kernel::command::model::aggregate::Aggregate,
        event: kernel::command::event::Event,
    ) -> Result<Result<(), kernel::command::error::CommandKernelError>, anyhow::Error> {
        use aws_sdk_dynamodb::types::Put;

        if !matches!(event, kernel::command::event::Event::Created { .. }) {
            return Ok(Err(
                kernel::command::error::CommandKernelError::InvalidEvents {
                    events: vec![event],
                },
            ));
        }

        let mut transact_items = Vec::new();
        let sequence = EventSequenceModel::new(aggregate.id().to_string(), 0);

        transact_items.push(put_event(EventStoreModel::new(
            sequence.latest_event_id(),
            aggregate.id().to_string(),
            event.into(),
        ))?);
        transact_items.push(
            TransactWriteItem::builder()
                .put(
                    Put::builder()
                        .table_name(EVENT_SEQUENCE_TABLE_NAME)
                        .set_item(Some(sequence.try_into()?))
                        .condition_expression("attribute_not_exists(aggregate_id)")
                        .build()
                        .with_context(|| "put event sequence model")?,
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
                        .with_context(|| "put aggregate model")?,
                )
                .build(),
        );

        self.dynamodb
            .transact_write_items()
            .set_transact_items(Some(transact_items))
            .send()
            .await
            .with_context(|| "transact write items")?;
        Ok(Ok(()))
    }

    async fn get(
        &self,
        id: kernel::id::Id<kernel::command::model::aggregate::Aggregate>,
    ) -> Result<
        Result<
            Option<kernel::command::model::aggregate::Aggregate>,
            kernel::command::error::CommandKernelError,
        >,
        anyhow::Error,
    > {
        use aws_sdk_dynamodb::operation::get_item::GetItemError::ResourceNotFoundException;
        use aws_sdk_dynamodb::types::AttributeValue;

        let result = self
            .dynamodb
            .get_item()
            .table_name(AGGREGATE_TABLE_NAME)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await;
        let item = match result {
            Ok(output) => match output.item {
                Some(item) => item,
                None => return Ok(Ok(None)),
            },
            Err(e) => match e.into_service_error() {
                ResourceNotFoundException(_) => return Ok(Ok(None)),
                e => return Err(e.into()),
            },
        };
        let model: AggregateModel = serde_dynamo::from_item(item)
            .with_context(|| "from DynamoDB item to AggregateModel")?;
        Ok(Ok(Some(model.try_into()?)))
    }

    async fn update(
        &self,
        aggregate: kernel::command::model::aggregate::Aggregate,
        events: Vec<kernel::command::event::Event>,
    ) -> Result<Result<(), kernel::command::error::CommandKernelError>, anyhow::Error> {
        use aws_sdk_dynamodb::types::{AttributeValue, Update};

        if events.is_empty()
            || events
                .iter()
                .any(|event| matches!(event, kernel::command::event::Event::Created { .. }))
        {
            return Ok(Err(
                kernel::command::error::CommandKernelError::InvalidEvents { events },
            ));
        }

        let aggregate_id = AttributeValue::S(aggregate.id().to_string());
        let sequence: EventSequenceModel = serde_dynamo::from_item(
            self.dynamodb
                .get_item()
                .table_name(EVENT_SEQUENCE_TABLE_NAME)
                .key("aggregate_id", aggregate_id.clone())
                .send()
                .await
                .with_context(|| "get event sequence item")?
                .item
                .with_context(|| "event sequence item not present")?,
        )
        .with_context(|| "from DynamoDB item to EventSequenceModel")?;

        let mut transact_items = Vec::new();
        let mut new_event_id = sequence.latest_event_id();
        for event in events {
            new_event_id += 1;
            transact_items.push(put_event(EventStoreModel::new(
                new_event_id,
                aggregate.id().to_string(),
                event.into(),
            ))?);
        }
        transact_items.push(
            TransactWriteItem::builder()
                .update(
                    Update::builder()
                        .table_name(EVENT_SEQUENCE_TABLE_NAME)
                        .key("aggregate_id", aggregate_id.clone())
                        .expression_attribute_values(
                            ":current_latest_event_id",
                            sequence.latest_event_id_attribute_value()?,
                        )
                        .expression_attribute_values(
                            ":new_latest_event_id",
                            AttributeValue::N(new_event_id.to_string()),
                        )
                        .update_expression("SET latest_event_id = :new_latest_event_id")
                        .condition_expression(
                            "attribute_exists(aggregate_id) AND latest_event_id = :current_latest_event_id",
                        )
                        .build()
                        .with_context(|| "update event sequence")?,
                )
                .build(),
        );
        let aggregate: AggregateModel = aggregate.into();
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
                                aggregate
                                    .version()
                                    .checked_sub(1)
                                    .with_context(|| "invalid aggregate version")?,
                            )
                            .with_context(|| "AggregateModel to AttributeValue")?,
                        )
                        .expression_attribute_values(
                            ":new_version",
                            aggregate.version_attribute_value()?,
                        )
                        .expression_attribute_values(
                            ":new_payload",
                            aggregate.payload_attribute_value()?,
                        )
                        .update_expression("SET version = :new_version, payload = :new_payload")
                        .condition_expression("attribute_exists(id) AND version = :current_version")
                        .build()
                        .with_context(|| "update aggregate")?,
                )
                .build(),
        );

        self.dynamodb
            .transact_write_items()
            .set_transact_items(Some(transact_items))
            .send()
            .await
            .with_context(|| "transect write items")?;
        Ok(Ok(()))
    }
}

/// イベントをイベントストアに追加する
fn put_event(event: EventStoreModel) -> Result<TransactWriteItem, anyhow::Error> {
    use aws_sdk_dynamodb::types::Put;

    let tx = TransactWriteItem::builder()
        .put(
            Put::builder()
                .table_name(EVENT_STORE_TABLE_NAME)
                .set_item(Some(event.try_into()?))
                .condition_expression(
                    "attribute_not_exists(id) AND attribute_not_exists(aggregate_id)",
                )
                .build()
                .with_context(|| "put event store model")?,
        )
        .build();
    Ok(tx)
}
