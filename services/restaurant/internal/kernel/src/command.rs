use crate::{Id, Item, Restaurant};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    CrateAggregate(Restaurant),
    AddItems(Vec<Item>),
    RemoveItems(Vec<Id<Item>>),
}
