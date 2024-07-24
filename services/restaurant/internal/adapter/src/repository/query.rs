use kernel::{Aggregate, Id, Item, KernelError, QueryProcessor, Restaurant};
use sqlx::{MySql, Pool};

use crate::{RestaurantItemModel, RestaurantModel};

#[derive(Debug, Clone)]
pub struct QueryRepository {
    mysql: Pool<MySql>,
}

impl QueryRepository {
    #[must_use]
    pub fn new(mysql: Pool<MySql>) -> Self {
        Self { mysql }
    }
}

impl QueryProcessor for QueryRepository {
    async fn list_restaurants(&self) -> Result<Vec<Restaurant>, KernelError> {
        let models: Vec<RestaurantModel> =
            sqlx::query_as("SELECT aggregate_id, restaurant_name FROM restaurant")
                .fetch_all(&self.mysql)
                .await
                .map_err(|e| KernelError::Unknown(e.into()))?;
        let results: Vec<Result<Restaurant, _>> =
            models.into_iter().map(TryInto::try_into).collect();
        if results.iter().any(Result::is_err) {
            return Err(KernelError::Unknown("convert restaurant model".into()));
        }
        let restaurants: Vec<_> = results.into_iter().map(|x| x.unwrap()).collect();
        Ok(restaurants)
    }

    async fn list_items(&self, aggregate_id: Id<Aggregate>) -> Result<Vec<Item>, KernelError> {
        let models: Vec<RestaurantItemModel> = sqlx::query_as(
            "SELECT item_id, item_name, price FROM restaurant_item WHERE aggregate_id = ?",
        )
        .bind(aggregate_id.to_string())
        .fetch_all(&self.mysql)
        .await
        .map_err(|e| KernelError::Unknown(e.into()))?;
        let results: Vec<Result<Item, _>> = models.into_iter().map(TryInto::try_into).collect();
        if results.iter().any(Result::is_err) {
            return Err(KernelError::Unknown("convert restaurant model".into()));
        }
        let items: Vec<_> = results.into_iter().map(|x| x.unwrap()).collect();
        Ok(items)
    }
}
