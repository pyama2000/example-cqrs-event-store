use crate::id::Id;

use super::model::entity::{Item, Tenant};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    Create,
    AddItem {
        tenant_id: Id<Tenant>,
        item_id: Id<Item>,
    },
    RemoveItem {
        tenant_id: Id<Tenant>,
        item_id: Id<Item>,
    },
    PlaceOrder,
}
