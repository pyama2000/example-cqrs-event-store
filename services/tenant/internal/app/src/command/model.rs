#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tenant {
    name: String,
}

impl Tenant {
    /// Creates a new [`Tenant`].
    #[must_use]
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item {
    name: String,
    price: u32,
}

impl Item {
    /// Creates a new [`Item`].
    #[must_use]
    pub fn new(name: String, price: u32) -> Self {
        Self { name, price }
    }
}
