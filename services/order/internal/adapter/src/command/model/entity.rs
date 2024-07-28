use serde::{Deserialize, Serialize};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Order {
    V1 {
        restaurant_id: String,
        user_id: String,
        delivery_address: String,
        status: OrderStatus,
    },
}

impl From<kernel::Order> for Order {
    fn from(value: kernel::Order) -> Self {
        Self::V1 {
            restaurant_id: value.restaurant_id().to_string(),
            user_id: value.user_id().to_string(),
            delivery_address: value.delivery_address().to_string(),
            status: value.status().clone().into(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderStatus {
    Received,
    Preparing,
    DeliveryPersonAssigned { delivery_person_id: String },
    ReadyForPickup { delivery_person_id: String },
    DeliveryPersonPickedUp { delivery_person_id: String },
    Delivered,
    Cancelled,
}

impl From<kernel::OrderStatus> for OrderStatus {
    fn from(value: kernel::OrderStatus) -> Self {
        match value {
            kernel::OrderStatus::Received => OrderStatus::Received,
            kernel::OrderStatus::Preparing => OrderStatus::Preparing,
            kernel::OrderStatus::DeliveryPersonAssigned { delivery_person_id } => {
                OrderStatus::DeliveryPersonAssigned {
                    delivery_person_id: delivery_person_id.to_string(),
                }
            }
            kernel::OrderStatus::ReadyForPickup { delivery_person_id } => {
                OrderStatus::ReadyForPickup {
                    delivery_person_id: delivery_person_id.to_string(),
                }
            }
            kernel::OrderStatus::DeliveryPersonPickedUp { delivery_person_id } => {
                OrderStatus::DeliveryPersonPickedUp {
                    delivery_person_id: delivery_person_id.to_string(),
                }
            }
            kernel::OrderStatus::Delivered => OrderStatus::Delivered,
            kernel::OrderStatus::Cancelled => OrderStatus::Cancelled,
        }
    }
}

impl TryFrom<OrderStatus> for kernel::OrderStatus {
    type Error = Error;

    fn try_from(value: OrderStatus) -> Result<Self, Self::Error> {
        let status = match value {
            OrderStatus::Received => kernel::OrderStatus::Received,
            OrderStatus::Preparing => kernel::OrderStatus::Preparing,
            OrderStatus::DeliveryPersonAssigned { delivery_person_id } => {
                kernel::OrderStatus::DeliveryPersonAssigned {
                    delivery_person_id: delivery_person_id.parse()?,
                }
            }
            OrderStatus::ReadyForPickup { delivery_person_id } => {
                kernel::OrderStatus::ReadyForPickup {
                    delivery_person_id: delivery_person_id.parse()?,
                }
            }
            OrderStatus::DeliveryPersonPickedUp { delivery_person_id } => {
                kernel::OrderStatus::DeliveryPersonPickedUp {
                    delivery_person_id: delivery_person_id.parse()?,
                }
            }
            OrderStatus::Delivered => kernel::OrderStatus::Delivered,
            OrderStatus::Cancelled => kernel::OrderStatus::Cancelled,
        };
        Ok(status)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderItem {
    V1 {
        item_id: String,
        price: u64,
        quantity: u64,
    },
}

impl From<kernel::OrderItem> for OrderItem {
    fn from(value: kernel::OrderItem) -> Self {
        Self::V1 {
            item_id: value.item_id().to_string(),
            price: value.price(),
            quantity: value.quantity(),
        }
    }
}

impl TryFrom<OrderItem> for kernel::OrderItem {
    type Error = Error;

    fn try_from(value: OrderItem) -> Result<Self, Self::Error> {
        let item = match value {
            OrderItem::V1 {
                item_id,
                price,
                quantity,
            } => Self::new(item_id.parse()?, price, quantity),
        };
        Ok(item)
    }
}
