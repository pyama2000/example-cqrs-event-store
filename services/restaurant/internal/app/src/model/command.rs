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
}

impl From<Restaurant> for kernel::Restaurant {
    fn from(value: Restaurant) -> Self {
        Self::new(Id::generate(), value.name)
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
