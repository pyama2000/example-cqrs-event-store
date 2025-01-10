use crate::Id;

use super::Item;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    Create { name: String },
    AddItems { items: Vec<Item> },
    RemoveItems { item_ids: Vec<Id<Item>> },
}
