use std::future::Future;

use crate::{Aggregate, Event, Id, KernelError};

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
