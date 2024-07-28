use serde::{Deserialize, Serialize};

use super::{Order, OrderItem};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AggregateModel {
    id: String,
    payload: AggregatePayload,
    version: u64,
}

impl AggregateModel {
    pub(crate) fn version(&self) -> u64 {
        self.version
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

    pub(crate) fn version_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, Error> {
        Ok(serde_dynamo::to_attribute_value(self.version)?)
    }
}

impl From<kernel::Aggregate> for AggregateModel {
    fn from(value: kernel::Aggregate) -> Self {
        let id = value.id().to_string();
        let version = value.version();
        Self {
            id,
            version,
            payload: value.into(),
        }
    }
}

impl TryFrom<AggregateModel> for kernel::Aggregate {
    type Error = Error;

    fn try_from(value: AggregateModel) -> Result<Self, Self::Error> {
        let (order, order_items) = match value.payload {
            AggregatePayload::V1 { order, order_items } => {
                let order = match order {
                    Order::V1 {
                        restaurant_id,
                        user_id,
                        delivery_address,
                        status,
                    } => kernel::Order::new(
                        restaurant_id.parse()?,
                        user_id.parse()?,
                        delivery_address,
                        status.try_into()?,
                    ),
                };
                let result: Result<Vec<kernel::OrderItem>, _> = order_items
                    .into_iter()
                    .map(kernel::OrderItem::try_from)
                    .collect();
                (order, result?)
            }
        };
        Ok(kernel::Aggregate::new(
            value.id.parse()?,
            order,
            order_items,
            value.version,
        ))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AggregatePayload {
    V1 {
        order: Order,
        order_items: Vec<OrderItem>,
    },
}

impl From<kernel::Aggregate> for AggregatePayload {
    fn from(value: kernel::Aggregate) -> Self {
        Self::V1 {
            order: value.order().clone().into(),
            order_items: value
                .order_items()
                .iter()
                .cloned()
                .map(Into::into)
                .collect(),
        }
    }
}
