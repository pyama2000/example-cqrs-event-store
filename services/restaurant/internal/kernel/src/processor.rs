use std::future::Future;

use crate::{Aggregate, Event, Id, Item, KernelError, Restaurant};

#[cfg_attr(feature = "mockall", mockall::automock)]
pub trait CommandProcessor {
    fn create(
        &self,
        aggregate: Aggregate,
        events: Vec<Event>,
    ) -> impl Future<Output = Result<(), KernelError>> + Send;

    fn get(&self, id: Id<Aggregate>)
        -> impl Future<Output = Result<Aggregate, KernelError>> + Send;

    fn update(
        &self,
        aggregate: Aggregate,
        events: Vec<Event>,
    ) -> impl Future<Output = Result<(), KernelError>> + Send;
}

#[cfg_attr(feature = "mockall", mockall::automock)]
pub trait QueryProcessor {
    fn list_restaurants(&self)
        -> impl Future<Output = Result<Vec<Restaurant>, KernelError>> + Send;

    fn list_items(
        &self,
        aggregate_id: Id<Aggregate>,
    ) -> impl Future<Output = Result<Vec<Item>, KernelError>> + Send;
}
