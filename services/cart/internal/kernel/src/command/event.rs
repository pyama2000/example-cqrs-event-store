use crate::id::Id;

use super::model::entity::{Item, Tenant};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    Created,
    ItemAdded {
        tenant_id: Id<Tenant>,
        item_id: Id<Item>,
    },
    ItemRemoved {
        tenant_id: Id<Tenant>,
        item_id: Id<Item>,
    },
    OrderPlaced,
}
