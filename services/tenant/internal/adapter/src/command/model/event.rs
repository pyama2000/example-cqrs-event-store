use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};

use super::Item;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct EventStoreModel {
    id: u64,
    aggregate_id: String,
    payload: EventPayload,
}

impl EventStoreModel {
    pub(crate) fn new(id: u64, aggregate_id: String, payload: EventPayload) -> Self {
        Self {
            id,
            aggregate_id,
            payload,
        }
    }

    #[cfg(test)]
    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    #[cfg(test)]
    pub(crate) fn aggregate_id(&self) -> &str {
        &self.aggregate_id
    }
}

impl<S> TryFrom<EventStoreModel> for HashMap<String, AttributeValue, S>
where
    S: std::hash::BuildHasher,
    HashMap<String, AttributeValue, S>: From<serde_dynamo::Item>,
{
    type Error = Error;

    fn try_from(value: EventStoreModel) -> Result<Self, Self::Error> {
        Ok(serde_dynamo::to_item(value)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum EventPayload {
    TenantCreatedV1 { name: String },
    ItemsAddedV1 { items: Vec<Item> },
    ItemsRemoved { item_ids: Vec<String> },
}

impl From<kernel::Event> for EventPayload {
    fn from(value: kernel::Event) -> Self {
        match value {
            kernel::Event::Created { name } => Self::TenantCreatedV1 { name },
            kernel::Event::ItemsAdded { items } => Self::ItemsAddedV1 {
                items: items.into_iter().map(Into::into).collect(),
            },
            kernel::Event::ItemsRemoved { item_ids } => Self::ItemsRemoved {
                item_ids: item_ids.into_iter().map(|x| x.to_string()).collect(),
            },
        }
    }
}

impl TryFrom<EventPayload> for AttributeValue {
    type Error = Error;

    fn try_from(value: EventPayload) -> Result<Self, Self::Error> {
        Ok(serde_dynamo::to_attribute_value(value)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct EventSequenceModel {
    aggregate_id: String,
    version: u64,
}

impl EventSequenceModel {
    pub(crate) fn new(aggregate_id: String, version: u64) -> Self {
        Self {
            aggregate_id,
            version,
        }
    }

    pub(crate) fn version(&self) -> u64 {
        self.version
    }

    pub(crate) fn version_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, Error> {
        Ok(serde_dynamo::to_attribute_value(self.version)?)
    }

    #[cfg(test)]
    pub(crate) fn aggregate_id(&self) -> &str {
        &self.aggregate_id
    }
}

impl<S> TryFrom<EventSequenceModel> for HashMap<String, AttributeValue, S>
where
    S: std::hash::BuildHasher,
    HashMap<String, AttributeValue, S>: From<serde_dynamo::Item>,
{
    type Error = Error;

    fn try_from(value: EventSequenceModel) -> Result<Self, Self::Error> {
        Ok(serde_dynamo::to_item(value)?)
    }
}
