use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};

use super::Item;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    ) -> Result<T, Error> {
        Ok(serde_dynamo::to_attribute_value(self.version)?)
    }

    pub(crate) fn payload_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, Error> {
        Ok(serde_dynamo::to_attribute_value(&self.payload)?)
    }

    #[cfg(test)]
    pub(crate) fn new(id: String, version: u64, payload: AggregatePayload) -> Self {
        Self {
            id,
            version,
            payload,
        }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn id(&self) -> &str {
        &self.id
    }
}

impl From<kernel::Aggregate> for AggregateModel {
    fn from(value: kernel::Aggregate) -> Self {
        Self {
            id: value.id().to_string(),
            version: value.version(),
            payload: value.into(),
        }
    }
}

impl TryInto<kernel::Aggregate> for AggregateModel {
    type Error = Error;

    fn try_into(self) -> Result<kernel::Aggregate, Self::Error> {
        let (name, items) = match self.payload {
            AggregatePayload::V1 { name, items } => (name, items),
        };
        let items: Vec<kernel::Item> = items
            .into_iter()
            .map(|item| match item {
                Item::V1 { id, name, price } => {
                    Ok::<_, Error>(kernel::Item::new(id.parse()?, name, price))
                }
            })
            .collect::<Result<_, _>>()?;
        Ok(kernel::Aggregate::new(
            self.id.parse()?,
            name,
            items,
            self.version,
        ))
    }
}

impl<S> TryFrom<AggregateModel> for HashMap<String, AttributeValue, S>
where
    S: std::hash::BuildHasher,
    HashMap<String, AttributeValue, S>: From<serde_dynamo::Item>,
{
    type Error = Error;

    fn try_from(value: AggregateModel) -> Result<Self, Self::Error> {
        Ok(serde_dynamo::to_item(value)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AggregatePayload {
    V1 { name: String, items: Vec<Item> },
}

impl From<kernel::Aggregate> for AggregatePayload {
    fn from(value: kernel::Aggregate) -> Self {
        let items: Vec<Item> = value.items().iter().cloned().map(Into::into).collect();
        Self::V1 {
            name: value.name().to_string(),
            items,
        }
    }
}
