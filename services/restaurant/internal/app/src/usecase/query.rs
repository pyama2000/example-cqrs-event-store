use std::future::Future;

use kernel::{Aggregate, Id, QueryProcessor};

use crate::{AppError, Item, Restaurant};

pub trait QueryService {
    fn list_restaurants(
        &self,
    ) -> impl Future<Output = Result<Vec<(Id<Aggregate>, Restaurant)>, AppError>> + Send;

    fn list_items(
        &self,
        aggregate_id: Id<Aggregate>,
    ) -> impl Future<Output = Result<Vec<(Id<kernel::Item>, Item)>, AppError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QueryUseCase<P: QueryProcessor> {
    processor: P,
}

impl<P: QueryProcessor> QueryUseCase<P> {
    pub fn new(processor: P) -> Self {
        Self { processor }
    }
}

impl<P: QueryProcessor + Send + Sync + 'static> QueryService for QueryUseCase<P> {
    async fn list_restaurants(&self) -> Result<Vec<(Id<Aggregate>, Restaurant)>, AppError> {
        let restaurant_by_aggregate_id_list = self.processor.list_restaurants().await?;
        Ok(restaurant_by_aggregate_id_list
            .into_iter()
            .map(|(id, r)| (id, r.into()))
            .collect())
    }

    async fn list_items(
        &self,
        aggregate_id: Id<Aggregate>,
    ) -> Result<Vec<(Id<kernel::Item>, Item)>, AppError> {
        let items = self.processor.list_items(aggregate_id).await?;
        Ok(items
            .into_iter()
            .map(|x| (x.id().clone(), x.into()))
            .collect())
    }
}
