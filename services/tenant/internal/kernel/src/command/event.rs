use crate::Id;

use super::Item;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    Created { name: String },
    ItemsAdded { items: Vec<Item> },
    ItemsRemoved { item_ids: Vec<Id<Item>> },
}
