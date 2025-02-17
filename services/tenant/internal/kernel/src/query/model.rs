#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Tenant {
    id: String,
    name: String,
}

impl Tenant {
    /// Creates a new [`Tenant`].
    #[must_use]
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Item {
    id: String,
    name: String,
    price: u32,
}

impl Item {
    /// Creates a new [`Item`].
    #[must_use]
    pub fn new(id: String, name: String, price: u32) -> Self {
        Self { id, name, price }
    }

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
