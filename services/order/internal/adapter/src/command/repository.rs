use aws_sdk_dynamodb::operation::get_item::GetItemError;
use aws_sdk_dynamodb::types::{AttributeValue, Put, TransactWriteItem, Update};
use kernel::{CommandProcessor, EventPayload, KernelError};

use crate::{AggregateModel, EventModel};

const AGGREGATE_TABLE_NAME: &str = "order-aggregate";
const EVENT_TABLE_NAME: &str = "order-event";

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
    async fn create(
        &self,
        aggregate: kernel::Aggregate,
        events: Vec<kernel::Event>,
    ) -> Result<(), KernelError> {
        if events.is_empty() {
            return Err(KernelError::EmptyEvents);
        }
        if events
            .first()
            .is_some_and(|x| !matches!(x.payload(), EventPayload::Received { .. }))
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
        transact_items.push(
            TransactWriteItem::builder()
                .put(
                    Put::builder()
                        .table_name(AGGREGATE_TABLE_NAME)
                        .set_item(Some(AggregateModel::from(aggregate).to_item()?))
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

    async fn get(
        &self,
        id: kernel::Id<kernel::Aggregate>,
    ) -> Result<kernel::Aggregate, KernelError> {
        let model: AggregateModel = match self
            .dynamodb
            .get_item()
            .table_name(AGGREGATE_TABLE_NAME)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
        {
            Ok(o) => serde_dynamo::from_item(
                o.item()
                    .ok_or_else(|| KernelError::AggregateNotFound)?
                    .clone(),
            )
            .map_err(|e| KernelError::Unknown(e.into()))?,
            Err(e) => {
                let e = match e.into_service_error() {
                    GetItemError::ResourceNotFoundException(_) => KernelError::AggregateNotFound,
                    e => KernelError::Unknown(e.into()),
                };
                return Err(e);
            }
        };
        Ok(model.try_into()?)
    }

    async fn update(
        &self,
        aggregate: kernel::Aggregate,
        events: Vec<kernel::Event>,
    ) -> Result<(), KernelError> {
        if events.is_empty() {
            return Err(KernelError::EmptyEvents);
        }
        if events
            .iter()
            .any(|x| matches!(x.payload(), EventPayload::Received { .. }))
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
        let aggregate_model: AggregateModel = aggregate.into();
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
