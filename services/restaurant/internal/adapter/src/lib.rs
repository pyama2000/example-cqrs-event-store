#![allow(clippy::module_name_repetitions)]

pub(crate) mod model;
pub mod persistence;
pub mod repository;

pub(crate) use model::{AggregateModel, EventModel};
pub use persistence::dynamodb;
pub use repository::CommandRepository;
