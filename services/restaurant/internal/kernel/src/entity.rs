use crate::Id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
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

/// 商品
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item {
    id: Id<Item>,
    name: String,
    price: Price,
}

impl Item {
    #[must_use]
    pub fn new(id: Id<Item>, name: String, price: Price) -> Self {
        Self {
            id,
            name,
            price,
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
}

/// 商品の価格
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Price {
    Yen(u64),
}
