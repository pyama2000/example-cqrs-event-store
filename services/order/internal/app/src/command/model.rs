use kernel::Id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Restaurant;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct User;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Item;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DeliveryPerson;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Order {
    restaurant_id: Id<Restaurant>,
    user_id: Id<User>,
    delivery_address: String,
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

    #[must_use]
    pub fn restaurant_id(&self) -> &Id<Restaurant> {
        &self.restaurant_id
    }

    #[must_use]
    pub fn user_id(&self) -> &Id<User> {
        &self.user_id
    }

    #[must_use]
    pub fn delivery_address(&self) -> &str {
        &self.delivery_address
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

    #[must_use]
    pub fn item_id(&self) -> &Id<Item> {
        &self.item_id
    }

    #[must_use]
    pub fn price(&self) -> u64 {
        self.price
    }

    #[must_use]
    pub fn quantity(&self) -> u64 {
        self.quantity
    }
}
