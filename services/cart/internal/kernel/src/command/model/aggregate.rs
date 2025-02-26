use std::collections::HashMap;

use crate::id::Id;

use super::entity::{Item, Tenant};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Aggregate {
    id: Id<Aggregate>,
    item_ids_by_tenant_id: HashMap<Id<Tenant>, Vec<Id<Item>>>,
    /// 集約のバージョン
    version: u128,
}

impl Aggregate {
    #[must_use]
    pub fn new(
        id: Id<Aggregate>,
        item_ids_by_tenant_id: HashMap<Id<Tenant>, Vec<Id<Item>>>,
        version: u128,
    ) -> Self {
        Self {
            id,
            item_ids_by_tenant_id,
            version,
        }
    }

    /// 集約のID
    #[must_use]
    pub fn id(&self) -> &Id<Aggregate> {
        &self.id
    }

    #[must_use]
    pub fn item_ids_by_tenant_id(&self) -> &HashMap<Id<Tenant>, Vec<Id<Item>>> {
        &self.item_ids_by_tenant_id
    }

    /// 集約のバージョン
    #[must_use]
    pub fn version(&self) -> u128 {
        self.version
    }
}
