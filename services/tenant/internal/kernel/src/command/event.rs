use crate::Id;

use super::{Command, Item};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    Created { name: String },
    ItemsAdded { items: Vec<Item> },
    ItemsRemoved { item_ids: Vec<Id<Item>> },
}

impl From<Command> for Vec<Event> {
    fn from(value: Command) -> Self {
        match value {
            Command::Create { name } => vec![Event::Created { name }],
            Command::AddItems { items } => vec![Event::ItemsAdded { items }],
            Command::RemoveItems { item_ids } => vec![Event::ItemsRemoved { item_ids }],
        }
    }
}
