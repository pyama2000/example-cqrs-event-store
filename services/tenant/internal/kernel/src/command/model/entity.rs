use crate::Id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Item {
    id: Id<Item>,
    name: String,
    price: u32,
}

impl Item {
    /// Creates a new [`Item`].
    #[must_use]
    pub fn new(id: Id<Item>, name: String, price: u32) -> Self {
        Self { id, name, price }
    }

    /// 商品ID
    #[must_use]
    pub fn id(&self) -> &Id<Item> {
        &self.id
    }

    /// 商品名
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 商品の価格
    #[must_use]
    pub fn price(&self) -> u32 {
        self.price
    }
}
