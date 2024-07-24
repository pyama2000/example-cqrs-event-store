use kernel::Id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Restaurant {
    name: String,
}

impl Restaurant {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self { name }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl From<Restaurant> for kernel::Restaurant {
    fn from(value: Restaurant) -> Self {
        Self::new(Id::generate(), value.name)
    }
}

impl From<kernel::Restaurant> for Restaurant {
    fn from(value: kernel::Restaurant) -> Self {
        Self {
            name: value.name().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item {
    name: String,
    price: Price,
    category: ItemCategory,
}

impl Item {
    #[must_use]
    pub fn new(name: String, price: Price, category: ItemCategory) -> Self {
        Self {
            name,
            price,
            category,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn price(&self) -> &Price {
        &self.price
    }

    #[must_use]
    pub fn category(&self) -> &ItemCategory {
        &self.category
    }
}

impl From<Item> for kernel::Item {
    fn from(value: Item) -> Self {
        Self::new(
            Id::generate(),
            value.name,
            value.price.into(),
            value.category.into(),
        )
    }
}

impl From<kernel::Item> for Item {
    fn from(value: kernel::Item) -> Self {
        Self {
            name: value.name().to_string(),
            price: value.price().clone().into(),
            category: value.category().clone().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemCategory {
    Food,
    Drink,
    Other(String),
}

impl From<ItemCategory> for kernel::ItemCategory {
    fn from(value: ItemCategory) -> Self {
        match value {
            ItemCategory::Food => kernel::ItemCategory::Food,
            ItemCategory::Drink => kernel::ItemCategory::Drink,
            ItemCategory::Other(x) => kernel::ItemCategory::Other(x),
        }
    }
}

impl From<kernel::ItemCategory> for ItemCategory {
    fn from(value: kernel::ItemCategory) -> Self {
        match value {
            kernel::ItemCategory::Food => Self::Food,
            kernel::ItemCategory::Drink => Self::Drink,
            kernel::ItemCategory::Other(x) => Self::Other(x),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Price {
    Yen(u64),
}

impl From<Price> for kernel::Price {
    fn from(value: Price) -> Self {
        match value {
            Price::Yen(x) => Self::Yen(x),
        }
    }
}

impl From<kernel::Price> for Price {
    fn from(value: kernel::Price) -> Self {
        match value {
            kernel::Price::Yen(x) => Self::Yen(x),
        }
    }
}
