#![allow(clippy::module_name_repetitions)]

pub mod command;
pub mod persistence;

pub(crate) use command::AggregateModel;
pub use command::{CommandRepository, EventModel, EventPayload, Order, OrderItem};
pub use persistence::dynamodb;
