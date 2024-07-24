pub mod aggregate;
#[allow(clippy::module_inception)]
pub mod command;
pub mod entity;
pub mod event;

pub use aggregate::Aggregate;
pub use command::Command;
pub use entity::{DeliveryPerson, Order, OrderItem, OrderStatus};
pub use event::{Event, EventPayload};
