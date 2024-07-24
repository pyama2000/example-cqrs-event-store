use kernel::Aggregate;
use serde::{Deserialize, Serialize};

use super::{Item, Restaurant};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AggregateModel {
    id: String,
    payload: Payload,
    version: u64,
}

impl AggregateModel {
    pub(crate) fn new(aggregate: Aggregate) -> Self {
        let id = aggregate.id().to_string();
        let version = aggregate.version();
        let payload = aggregate.into();
        Self {
            id,
            payload,
            version,
        }
    }

    pub(crate) fn to_item<T: From<serde_dynamo::Item>>(&self) -> Result<T, Error> {
        Ok(serde_dynamo::to_item(self)?)
    }

    pub(crate) fn id_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, Error> {
        Ok(serde_dynamo::to_attribute_value(&self.id)?)
    }

    pub(crate) fn payload_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, Error> {
        Ok(serde_dynamo::to_attribute_value(&self.payload)?)
    }

    pub(crate) fn version(&self) -> u64 {
        self.version
    }

    pub(crate) fn version_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, Error> {
        Ok(serde_dynamo::to_attribute_value(self.version)?)
    }
}

impl TryInto<Aggregate> for AggregateModel {
    type Error = Error;

    fn try_into(self) -> Result<Aggregate, Self::Error> {
        let (restaurant, items) = match self.payload {
            Payload::V1 { restaurant, items } => (restaurant, items),
        };
        let restaurant = match restaurant {
            Restaurant::V1 { name } => kernel::Restaurant::new(name),
        };
        let results: Vec<_> = items
            .into_iter()
            .map(|x| match x {
                Item::V1 {
                    id,
                    name,
                    price,
                    category,
                } => Ok::<kernel::Item, Self::Error>(kernel::Item::new(
                    id.parse()?,
                    name,
                    match price {
                        crate::model::command::entity::Price::Yen(v) => kernel::Price::Yen(v),
                    },
                    match category {
                        crate::model::command::entity::ItemCategory::Food => {
                            kernel::ItemCategory::Food
                        }
                        crate::model::command::entity::ItemCategory::Drink => {
                            kernel::ItemCategory::Drink
                        }
                        crate::model::command::entity::ItemCategory::Other(v) => {
                            kernel::ItemCategory::Other(v)
                        }
                    },
                )),
            })
            .collect();
        if results.iter().any(Result::is_err) {
            return Err("convert adapter items to kernel items".into());
        }
        let items: Vec<_> = results.into_iter().map(Result::unwrap).collect();
        Ok(Aggregate::new(
            self.id.parse()?,
            restaurant,
            items,
            self.version,
        ))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Payload {
    V1 {
        restaurant: Restaurant,
        items: Vec<Item>,
    },
}

impl From<Aggregate> for Payload {
    fn from(value: Aggregate) -> Self {
        let items: Vec<_> = value
            .items()
            .iter()
            .cloned()
            .map(std::convert::Into::into)
            .collect();
        Self::V1 {
            restaurant: value.restaurant().clone().into(),
            items,
        }
    }
}
