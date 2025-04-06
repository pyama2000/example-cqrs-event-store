use crate::id::Id;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Tenant;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Cart {
    id: Id<Cart>,
    items: Vec<Item>,
}

impl Cart {
    #[must_use]
    pub fn new(id: Id<Cart>, items: Vec<Item>) -> Self {
        Self { id, items }
    }

    #[must_use]
    pub fn id(&self) -> &Id<Cart> {
        &self.id
    }

    #[must_use]
    pub fn items(&self) -> &[Item] {
        &self.items
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Item {
    tenant_id: Id<Tenant>,
    item_id: Id<Item>,
    quantity: u32,
}

impl Item {
    #[must_use]
    pub fn new(tenant_id: Id<Tenant>, item_id: Id<Item>, quantity: u32) -> Self {
        Self {
            tenant_id,
            item_id,
            quantity,
        }
    }

    #[must_use]
    pub fn tenant_id(&self) -> &Id<Tenant> {
        &self.tenant_id
    }

    #[must_use]
    pub fn item_id(&self) -> &Id<Item> {
        &self.item_id
    }

    #[must_use]
    pub fn quantity(&self) -> u32 {
        self.quantity
    }
}
