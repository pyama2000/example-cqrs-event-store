use crate::id::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cart;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tenant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item {
    id: Id<Item>,
    tenant_id: Id<Tenant>,
    quantity: u32,
}

impl Item {
    /// Create new [`Item`]
    #[must_use]
    pub fn new(id: Id<Item>, tenant_id: Id<Tenant>, quantity: u32) -> Self {
        Self {
            id,
            tenant_id,
            quantity,
        }
    }

    #[must_use]
    pub fn id(&self) -> Id<Item> {
        self.id
    }

    #[must_use]
    pub fn tenant_id(&self) -> Id<Tenant> {
        self.tenant_id
    }

    #[must_use]
    pub fn quantity(&self) -> u32 {
        self.quantity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum OrderStatus {
    #[default]
    Created,
    Prepared,
    PickedUp,
    Delivered,
    Canceled,
}
