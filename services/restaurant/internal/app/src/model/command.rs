use kernel::Id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Restaurant {
    name: String,
}

impl Restaurant {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self { name }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl From<Restaurant> for kernel::Restaurant {
    fn from(value: Restaurant) -> Self {
        Self::new(value.name)
    }
}

impl From<kernel::Restaurant> for Restaurant {
    fn from(value: kernel::Restaurant) -> Self {
        Self {
            name: value.name().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item {
    name: String,
    price: u64,
}

impl Item {
    #[must_use]
    pub fn new(name: String, price: u64) -> Self {
        Self { name, price }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn price(&self) -> u64 {
        self.price
    }
}

impl From<Item> for kernel::Item {
    fn from(value: Item) -> Self {
        Self::new(Id::generate(), value.name, value.price)
    }
}

impl From<kernel::Item> for Item {
    fn from(value: kernel::Item) -> Self {
        Self {
            name: value.name().to_string(),
            price: value.price(),
        }
    }
}
