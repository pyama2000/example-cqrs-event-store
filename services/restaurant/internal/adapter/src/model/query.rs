use std::str::FromStr;

use kernel::Id;
use serde::Deserialize;
use sqlx::prelude::FromRow;

#[derive(FromRow, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RestaurantModel {
    aggregate_id: String,
    restaurant_name: String,
}

impl TryFrom<RestaurantModel> for kernel::Restaurant {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn try_from(value: RestaurantModel) -> Result<Self, Self::Error> {
        Ok(Self::new(
            Id::from_str(&value.aggregate_id)?,
            value.restaurant_name,
        ))
    }
}

#[derive(FromRow, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RestaurantItemModel {
    item_id: String,
    item_name: String,
    price: u64,
}

impl TryFrom<RestaurantItemModel> for kernel::Item {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn try_from(value: RestaurantItemModel) -> Result<Self, Self::Error> {
        Ok(Self::new(
            Id::from_str(&value.item_id)?,
            value.item_name,
            kernel::Price::Yen(value.price),
            kernel::ItemCategory::Food,
        ))
    }
}
