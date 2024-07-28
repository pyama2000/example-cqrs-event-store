use kernel::Id;
use serde::{Deserialize, Serialize};

use super::{Order, OrderItem};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventModel {
    id: String,
    aggregate_id: String,
    payload: EventPayload,
}

impl EventModel {
    pub(crate) fn new(
        id: &Id<kernel::Event>,
        aggregate_id: &Id<kernel::Aggregate>,
        payload: kernel::EventPayload,
    ) -> Self {
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

    #[must_use]
    pub fn payload(&self) -> &EventPayload {
        &self.payload
    }

    pub(crate) fn to_item<T: From<serde_dynamo::Item>>(
        &self,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(serde_dynamo::to_item(self)?)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPayload {
    ReceivedV1 {
        order: Order,
        order_items: Vec<OrderItem>,
    },
    PreparationStartedV1,
    DeliveryPersonAssignedV1 {
        delivery_person_id: String,
    },
    ReadyForPickupV1,
    DeliveryPersonPickedUpV1,
    DeliveredV1,
    CancelledV1,
}

impl From<kernel::EventPayload> for EventPayload {
    fn from(value: kernel::EventPayload) -> Self {
        match value {
            kernel::EventPayload::Received { order, items } => EventPayload::ReceivedV1 {
                order: order.into(),
                order_items: items.into_iter().map(Into::into).collect(),
            },
            kernel::EventPayload::PreparationStarted => EventPayload::PreparationStartedV1,
            kernel::EventPayload::DeliveryPersonAssigned { delivery_person_id } => {
                EventPayload::DeliveryPersonAssignedV1 {
                    delivery_person_id: delivery_person_id.to_string(),
                }
            }
            kernel::EventPayload::ReadyForPickup => EventPayload::ReadyForPickupV1,
            kernel::EventPayload::DeliveryPersonPickedUp => EventPayload::DeliveryPersonPickedUpV1,
            kernel::EventPayload::Delivered => EventPayload::DeliveredV1,
            kernel::EventPayload::Cancelled => EventPayload::CancelledV1,
        }
    }
}
