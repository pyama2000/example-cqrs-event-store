use std::collections::HashMap;

use anyhow::Context;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AggregateModel {
    id: String,
    version: u64,
    payload: AggregatePayload,
}

impl AggregateModel {
    pub(crate) fn version(&self) -> u64 {
        self.version
    }

    pub(crate) fn version_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, anyhow::Error> {
        Ok(serde_dynamo::to_attribute_value(self.version)?)
    }

    pub(crate) fn payload_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, anyhow::Error> {
        Ok(serde_dynamo::to_attribute_value(&self.payload)?)
    }
}

impl From<kernel::command::model::aggregate::Aggregate> for AggregateModel {
    fn from(value: kernel::command::model::aggregate::Aggregate) -> Self {
        Self {
            id: value.id().to_string(),
            version: value.version(),
            payload: value.into(),
        }
    }
}

impl TryFrom<AggregateModel> for kernel::command::model::aggregate::Aggregate {
    type Error = anyhow::Error;

    fn try_from(value: AggregateModel) -> Result<Self, Self::Error> {
        use kernel::command::model::entity::{Cart, Item, OrderStatus};
        use kernel::id::Id;

        let AggregateModel {
            id,
            version,
            payload,
        } = value;

        let (cart_id, items, status): (Id<Cart>, Vec<Item>, OrderStatus) = match payload {
            AggregatePayload::V1 {
                cart_id,
                items,
                order_status,
            } => (
                cart_id
                    .parse()
                    .with_context(|| format!("parse cart id: {cart_id}"))?,
                items
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                order_status.into(),
            ),
        };
        Ok(Self::new(
            id.parse().with_context(|| "parse aggregate id: {id}")?,
            cart_id,
            items,
            status,
            version,
        ))
    }
}

impl<S> TryFrom<AggregateModel> for HashMap<String, AttributeValue, S>
where
    S: std::hash::BuildHasher,
    HashMap<String, AttributeValue, S>: From<serde_dynamo::Item>,
{
    type Error = anyhow::Error;

    fn try_from(value: AggregateModel) -> Result<Self, Self::Error> {
        Ok(serde_dynamo::to_item(value)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum AggregatePayload {
    V1 {
        cart_id: String,
        items: Vec<Item>,
        order_status: OrderStatus,
    },
}

impl From<kernel::command::model::aggregate::Aggregate> for AggregatePayload {
    fn from(value: kernel::command::model::aggregate::Aggregate) -> Self {
        Self::V1 {
            cart_id: value.cart_id().to_string(),
            items: value.items().iter().copied().map(Into::into).collect(),
            order_status: value.status().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Item {
    V1 {
        id: String,
        tenant_id: String,
        quantity: u32,
    },
}

impl From<kernel::command::model::entity::Item> for Item {
    fn from(value: kernel::command::model::entity::Item) -> Self {
        Self::V1 {
            id: value.id().to_string(),
            tenant_id: value.tenant_id().to_string(),
            quantity: value.quantity(),
        }
    }
}

impl TryFrom<Item> for kernel::command::model::entity::Item {
    type Error = anyhow::Error;

    fn try_from(value: Item) -> Result<Self, Self::Error> {
        use kernel::command::model::entity::Tenant;
        use kernel::id::Id;

        let (id, tenant_id, quantity): (Id<Self>, Id<Tenant>, u32) = match value {
            Item::V1 {
                id,
                tenant_id,
                quantity,
            } => (
                id.parse().with_context(|| format!("parse item id: {id}"))?,
                tenant_id
                    .parse()
                    .with_context(|| format!("parse tenant id: {tenant_id}"))?,
                quantity,
            ),
        };
        Ok(Self::new(id, tenant_id, quantity))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum OrderStatus {
    Created,
    Prepared,
    PickedUp,
    Delivered,
    Canceled,
}

impl From<kernel::command::model::entity::OrderStatus> for OrderStatus {
    fn from(value: kernel::command::model::entity::OrderStatus) -> Self {
        match value {
            kernel::command::model::entity::OrderStatus::Created => Self::Created,
            kernel::command::model::entity::OrderStatus::Prepared => Self::Prepared,
            kernel::command::model::entity::OrderStatus::PickedUp => Self::PickedUp,
            kernel::command::model::entity::OrderStatus::Delivered => Self::Delivered,
            kernel::command::model::entity::OrderStatus::Canceled => Self::Canceled,
        }
    }
}

impl From<OrderStatus> for kernel::command::model::entity::OrderStatus {
    fn from(value: OrderStatus) -> Self {
        match value {
            OrderStatus::Created => Self::Created,
            OrderStatus::Prepared => Self::Prepared,
            OrderStatus::PickedUp => Self::PickedUp,
            OrderStatus::Delivered => Self::Delivered,
            OrderStatus::Canceled => Self::Canceled,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct EventStoreModel {
    id: u64,
    aggregate_id: String,
    payload: EventStorePayload,
}

impl EventStoreModel {
    #[must_use]
    pub(crate) fn new(id: u64, aggregate_id: String, payload: EventStorePayload) -> Self {
        Self {
            id,
            aggregate_id,
            payload,
        }
    }
}

impl<S> TryFrom<EventStoreModel> for HashMap<String, AttributeValue, S>
where
    S: std::hash::BuildHasher,
    HashMap<String, AttributeValue, S>: From<serde_dynamo::Item>,
{
    type Error = anyhow::Error;

    fn try_from(value: EventStoreModel) -> Result<Self, Self::Error> {
        serde_dynamo::to_item(value).with_context(|| "try from EventStoreModel")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum EventStorePayload {
    CreatedV1 { cart_id: String, items: Vec<Item> },
    PreparedV1,
    PickedUpV1,
    DeliveredV1,
    CanceledV1,
}

impl From<kernel::command::event::Event> for EventStorePayload {
    fn from(value: kernel::command::event::Event) -> Self {
        match value {
            kernel::command::event::Event::Created { cart_id, items } => Self::CreatedV1 {
                cart_id: cart_id.to_string(),
                items: items.into_iter().map(Into::into).collect(),
            },
            kernel::command::event::Event::Prepared => Self::PreparedV1,
            kernel::command::event::Event::PickedUp => Self::PickedUpV1,
            kernel::command::event::Event::Delivered => Self::DeliveredV1,
            kernel::command::event::Event::Canceled => Self::CanceledV1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct EventSequenceModel {
    aggregate_id: String,
    latest_event_id: u64,
}

impl EventSequenceModel {
    #[must_use]
    pub(crate) fn new(aggregate_id: String, latest_event_id: u64) -> Self {
        Self {
            aggregate_id,
            latest_event_id,
        }
    }

    #[must_use]
    pub(crate) fn latest_event_id(&self) -> u64 {
        self.latest_event_id
    }

    pub(crate) fn latest_event_id_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, anyhow::Error> {
        Ok(serde_dynamo::to_attribute_value(self.latest_event_id)?)
    }
}

impl<S> TryFrom<EventSequenceModel> for HashMap<String, AttributeValue, S>
where
    S: std::hash::BuildHasher,
    HashMap<String, AttributeValue, S>: From<serde_dynamo::Item>,
{
    type Error = anyhow::Error;

    fn try_from(value: EventSequenceModel) -> Result<Self, Self::Error> {
        serde_dynamo::to_item(value).with_context(|| "try from EventSequenceModel")
    }
}
