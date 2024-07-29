use kernel::Id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Restaurant;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct User;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeliveryPerson;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Order {
    pub(crate) restaurant_id: Id<Restaurant>,
    pub(crate) user_id: Id<User>,
    pub(crate) delivery_address: String,
}

impl Order {
    #[must_use]
    pub fn new(restaurant_id: Id<Restaurant>, user_id: Id<User>, delivery_address: String) -> Self {
        Self {
            restaurant_id,
            user_id,
            delivery_address,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderItem {
    item_id: Id<Item>,
    price: u64,
    quantity: u64,
}

impl OrderItem {
    #[must_use]
    pub fn new(item_id: Id<Item>, price: u64, quantity: u64) -> Self {
        Self {
            item_id,
            price,
            quantity,
        }
    }
}

impl TryFrom<OrderItem> for kernel::OrderItem {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn try_from(
        OrderItem {
            item_id,
            price,
            quantity,
        }: OrderItem,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(item_id.to_string().parse()?, price, quantity))
    }
}
