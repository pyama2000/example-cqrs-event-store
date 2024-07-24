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
    price: Price,
}

impl Item {
    #[must_use]
    pub fn new(name: String, price: Price) -> Self {
        Self { name, price }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn price(&self) -> &Price {
        &self.price
    }
}

impl From<Item> for kernel::Item {
    fn from(value: Item) -> Self {
        Self::new(Id::generate(), value.name, value.price.into())
    }
}

impl From<kernel::Item> for Item {
    fn from(value: kernel::Item) -> Self {
        Self {
            name: value.name().to_string(),
            price: value.price().clone().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Price {
    Yen(u64),
}

impl From<Price> for kernel::Price {
    fn from(value: Price) -> Self {
        match value {
            Price::Yen(x) => Self::Yen(x),
        }
    }
}

impl From<kernel::Price> for Price {
    fn from(value: kernel::Price) -> Self {
        match value {
            kernel::Price::Yen(x) => Self::Yen(x),
        }
    }
}
