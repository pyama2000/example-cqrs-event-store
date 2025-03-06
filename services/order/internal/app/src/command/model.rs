#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item {
    id: kernel::id::Id<kernel::command::model::entity::Item>,
    tenant_id: kernel::id::Id<kernel::command::model::entity::Tenant>,
    quantity: u32,
}

impl Item {
    #[must_use]
    pub fn new(
        id: kernel::id::Id<kernel::command::model::entity::Item>,
        tenant_id: kernel::id::Id<kernel::command::model::entity::Tenant>,
        quantity: u32,
    ) -> Self {
        Self {
            id,
            tenant_id,
            quantity,
        }
    }

    #[must_use]
    pub fn id(&self) -> kernel::id::Id<kernel::command::model::entity::Item> {
        self.id
    }

    #[must_use]
    pub fn tenant_id(&self) -> kernel::id::Id<kernel::command::model::entity::Tenant> {
        self.tenant_id
    }

    #[must_use]
    pub fn quantity(&self) -> u32 {
        self.quantity
    }
}

impl From<Item> for kernel::command::model::entity::Item {
    fn from(value: Item) -> Self {
        let Item {
            id,
            tenant_id,
            quantity,
        } = value;
        Self::new(id, tenant_id, quantity)
    }
}
