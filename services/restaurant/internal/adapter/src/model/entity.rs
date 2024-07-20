use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Restaurant {
    V1 { id: String, name: String },
}

impl From<kernel::Restaurant> for Restaurant {
    fn from(value: kernel::Restaurant) -> Self {
        Self::V1 {
            id: value.id().to_string(),
            name: value.name().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
    V1 {
        id: String,
        name: String,
        price: Price,
        category: ItemCategory,
    },
}

impl From<kernel::Item> for Item {
    fn from(value: kernel::Item) -> Self {
        Self::V1 {
            id: value.id().to_string(),
            name: value.name().to_string(),
            price: value.price().clone().into(),
            category: value.category().clone().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Price {
    Yen(u64),
}

impl From<kernel::Price> for Price {
    fn from(value: kernel::Price) -> Self {
        match value {
            kernel::Price::Yen(v) => Self::Yen(v),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemCategory {
    Food,
    Drink,
    Other(String),
}

impl From<kernel::ItemCategory> for ItemCategory {
    fn from(value: kernel::ItemCategory) -> Self {
        match value {
            kernel::ItemCategory::Food => Self::Food,
            kernel::ItemCategory::Drink => Self::Drink,
            kernel::ItemCategory::Other(v) => Self::Other(v),
        }
    }
}
