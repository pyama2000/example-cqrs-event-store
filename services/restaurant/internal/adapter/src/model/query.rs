use serde::Deserialize;
use sqlx::prelude::FromRow;

#[derive(FromRow, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RestaurantModel {
    aggregate_id: String,
    restaurant_name: String,
}

impl RestaurantModel {
    pub(crate) fn aggregate_id(&self) -> &str {
        &self.aggregate_id
    }

    pub(crate) fn restaurant_name(&self) -> &str {
        &self.restaurant_name
    }
}

#[derive(FromRow, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RestaurantItemModel {
    item_id: String,
    item_name: String,
    price: u64,
}

impl TryFrom<RestaurantItemModel> for kernel::Item {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn try_from(value: RestaurantItemModel) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.item_id.parse()?,
            value.item_name,
            kernel::Price::Yen(value.price),
        ))
    }
}
