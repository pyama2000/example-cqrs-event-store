use crate::Id;

use super::{Command, Item};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    id: u64,
    payload: EventPayload,
}

impl Event {
    #[must_use]
    pub fn new(id: u64, payload: EventPayload) -> Self {
        Self { id, payload }
    }

    /// イベントのID
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// イベントのペイロード
    #[must_use]
    pub fn payload(&self) -> &EventPayload {
        &self.payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPayload {
    Created { name: String },
    ItemsAdded { items: Vec<Item> },
    ItemsRemoved { item_ids: Vec<Id<Item>> },
}

impl From<Command> for Vec<EventPayload> {
    fn from(value: Command) -> Self {
        match value {
            Command::Create { name } => vec![EventPayload::Created { name }],
            Command::AddItems { items } => vec![EventPayload::ItemsAdded { items }],
            Command::RemoveItems { item_ids } => vec![EventPayload::ItemsRemoved { item_ids }],
        }
    }
}
