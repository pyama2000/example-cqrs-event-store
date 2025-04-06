use kernel::Id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tenant {
    pub(crate) name: String,
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
    pub(crate) name: String,
    pub(crate) price: u32,
}

impl Item {
    /// Creates a new [`Item`].
    #[must_use]
    pub fn new(name: String, price: u32) -> Self {
        Self { name, price }
    }
}

impl From<Item> for kernel::Item {
    fn from(Item { name, price }: Item) -> Self {
        Self::new(Id::generate(), name, price)
    }
}
