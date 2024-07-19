use crate::Id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Restaurant {
    id: Id<Restaurant>,
    name: String,
}

impl Restaurant {
    #[must_use]
    pub fn new(id: Id<Restaurant>, name: String) -> Self {
        Self { id, name }
    }

    #[must_use]
    pub fn id(&self) -> &Id<Restaurant> {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// 商品
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item {
    id: Id<Item>,
    name: String,
    price: Price,
    category: ItemCategory,
}

impl Item {
    #[must_use]
    pub fn new(id: Id<Item>, name: String, price: Price, category: ItemCategory) -> Self {
        Self {
            id,
            name,
            price,
            category,
        }
    }

    #[must_use]
    pub fn id(&self) -> &Id<Item> {
        &self.id
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

/// 商品のカテゴリー
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemCategory {
    Food,
    Drink,
    Other(String),
}

/// 商品の価格
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Price {
    Yen(u64),
}
