#![allow(clippy::module_name_repetitions)]

pub mod model;
pub mod persistence;
pub mod repository;

pub(crate) use model::AggregateModel;
pub use model::EventModel;
pub use persistence::dynamodb;
pub use repository::CommandRepository;
