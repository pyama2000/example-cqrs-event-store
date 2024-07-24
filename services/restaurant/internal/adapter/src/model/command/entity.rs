use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Restaurant {
    V1 { name: String },
}

impl Restaurant {
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Restaurant::V1 { name } => name,
        }
    }
}

impl From<kernel::Restaurant> for Restaurant {
    fn from(value: kernel::Restaurant) -> Self {
        Self::V1 {
            name: value.name().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
    V1 {
        id: String,
        name: String,
        price: u64,
    },
}

impl Item {
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Item::V1 { id, .. } => id,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Item::V1 { name, .. } => name,
        }
    }

    #[must_use]
    pub fn price(&self) -> u64 {
        match self {
            Item::V1 { price, .. } => *price,
        }
    }
}

impl From<kernel::Item> for Item {
    fn from(value: kernel::Item) -> Self {
        Self::V1 {
            id: value.id().to_string(),
            name: value.name().to_string(),
            price: value.price(),
        }
    }
}
