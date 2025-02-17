#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tenant {
    id: String,
    name: String,
}

impl Tenant {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl From<kernel::query::Tenant> for Tenant {
    fn from(value: kernel::query::Tenant) -> Self {
        Self {
            id: value.id().to_string(),
            name: value.name().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item {
    id: String,
    name: String,
    price: u32,
}

impl Item {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn price(&self) -> u32 {
        self.price
    }
}

impl From<kernel::query::Item> for Item {
    fn from(value: kernel::query::Item) -> Self {
        Self {
            id: value.id().to_string(),
            name: value.name().to_string(),
            price: value.price(),
        }
    }
}
