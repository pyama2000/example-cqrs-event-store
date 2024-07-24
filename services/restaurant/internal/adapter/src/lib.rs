#![allow(clippy::module_name_repetitions)]

pub mod model;
pub mod persistence;
pub mod repository;

pub(crate) use model::{AggregateModel, RestaurantItemModel, RestaurantModel};
pub use model::{EventModel, Payload};
pub use persistence::{dynamodb, mysql};
pub use repository::{CommandRepository, QueryRepository};
