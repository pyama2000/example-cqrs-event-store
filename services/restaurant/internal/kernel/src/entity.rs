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
    price: u64,
}

impl Item {
    #[must_use]
    pub fn new(id: Id<Item>, name: String, price: u64) -> Self {
        Self { id, name, price }
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
    pub fn price(&self) -> u64 {
        self.price
    }
}
