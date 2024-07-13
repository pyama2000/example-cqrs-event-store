use crate::{Id, Item, Restaurant};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Aggregate {
    id: Id<Aggregate>,
    restaurant: Restaurant,
    items: Vec<Item>,
    version: u64,
}

impl Aggregate {
    #[must_use]
    pub fn new(id: Id<Aggregate>, restaurant: Restaurant, items: Vec<Item>, version: u64) -> Self {
        Self {
            id,
            restaurant,
            items,
            version,
        }
    }

    #[must_use]
    pub fn id(&self) -> &Id<Aggregate> {
        &self.id
    }

    #[must_use]
    pub fn restaurant(&self) -> &Restaurant {
        &self.restaurant
    }

    #[must_use]
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }
}
