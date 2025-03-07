use crate::id::Id;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Cart;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Tenant;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Order {
    id: Id<Order>,
    items: Vec<Item>,
    status: OrderStatus,
}

impl Order {
    #[must_use]
    pub fn new(id: Id<Order>, items: Vec<Item>, status: OrderStatus) -> Self {
        Self { id, items, status }
    }

    #[must_use]
    pub fn id(&self) -> &Id<Order> {
        &self.id
    }

    #[must_use]
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    #[must_use]
    pub fn status(&self) -> OrderStatus {
        self.status
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item {
    id: Id<Item>,
    tenant_id: Id<Tenant>,
    quantity: u32,
}

impl Item {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OrderStatus {
    Received,
    Prepared,
    OnTheWay,
    Delivered,
    Canceled,
}
