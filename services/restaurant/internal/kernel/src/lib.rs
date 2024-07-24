#![allow(clippy::module_name_repetitions)]

pub mod aggregate;
pub mod command;
pub mod entity;
pub mod error;
pub mod event;
pub mod id;
pub mod processor;

pub use aggregate::Aggregate;
pub use command::Command;
pub use entity::{Item, Price, Restaurant};
pub use error::KernelError;
pub use event::{Event, EventPayload};
pub use id::Id;
pub use processor::{CommandProcessor, QueryProcessor};
