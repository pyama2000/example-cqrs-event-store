use crate::{DeliveryPerson, Id, Order, OrderItem};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    Receive {
        order: Order,
        items: Vec<OrderItem>,
    },
    Prepare,
    AssigningDeliveryPerson {
        delivery_person_id: Id<DeliveryPerson>,
    },
    ReadyForPickup,
    DeliveryPersonPickingUp,
    Delivered,
    Cancel,
}
