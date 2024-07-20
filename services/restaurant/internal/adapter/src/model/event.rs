use kernel::{Aggregate, Event, EventPayload, Id};
use serde::{Deserialize, Serialize};

use super::{Item, Restaurant};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventModel {
    id: String,
    aggregate_id: String,
    payload: Payload,
}

impl EventModel {
    #[must_use]
    pub fn new(id: &Id<Event>, aggregate_id: &Id<Aggregate>, payload: EventPayload) -> Self {
        Self {
            id: id.to_string(),
            aggregate_id: aggregate_id.to_string(),
            payload: payload.into(),
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn aggregate_id(&self) -> &str {
        &self.aggregate_id
    }

    /// # Errors
    pub fn to_item<T: From<serde_dynamo::Item>>(&self) -> Result<T, Error> {
        Ok(serde_dynamo::to_item(self)?)
    }

    #[cfg(test)]
    pub(crate) fn id_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, Error> {
        Ok(serde_dynamo::to_attribute_value(&self.id)?)
    }

    #[cfg(test)]
    pub(crate) fn aggregate_id_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, Error> {
        Ok(serde_dynamo::to_attribute_value(&self.aggregate_id)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Payload {
    AggregateCreatedV1(Restaurant),
    ItemsAddedV1(Vec<Item>),
    ItemsRemovedV1(Vec<String>),
}

impl From<EventPayload> for Payload {
    fn from(value: EventPayload) -> Self {
        match value {
            EventPayload::AggregateCreated(restaurant) => {
                Self::AggregateCreatedV1(restaurant.into())
            }
            EventPayload::ItemsAdded(items) => {
                Self::ItemsAddedV1(items.into_iter().map(std::convert::Into::into).collect())
            }
            EventPayload::ItemsRemoved(ids) => {
                Self::ItemsRemovedV1(ids.into_iter().map(|x| x.to_string()).collect())
            }
        }
    }
}
