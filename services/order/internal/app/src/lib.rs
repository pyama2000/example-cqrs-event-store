#![allow(clippy::module_name_repetitions)]

pub mod command;
pub mod error;

pub use command::{CommandService, Order, OrderItem};
pub use error::AppError;
