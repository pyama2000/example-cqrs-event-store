use crate::Id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Restaurant;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Item;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct User;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct DeliveryPerson;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Order {
    restaurant_id: Id<Restaurant>,
    user_id: Id<User>,
    delivery_address: String,
    status: OrderStatus,
}

impl Order {
    #[must_use]
    pub fn new(
        restaurant_id: Id<Restaurant>,
        user_id: Id<User>,
        delivery_address: String,
        status: OrderStatus,
    ) -> Self {
        Self {
            restaurant_id,
            user_id,
            delivery_address,
            status,
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

    #[must_use]
    pub fn status(&self) -> &OrderStatus {
        &self.status
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum OrderStatus {
    /// 注文を受け付けた状態
    #[default]
    Received,
    /// 飲食店が注文の準備をしている状態
    Preparing,
    /// 配達員を割り当てる状態
    AssigningDeliveryPerson {
        delivery_person_id: Id<DeliveryPerson>,
    },
    /// 飲食店が注文の準備が完了して、配達員が受け取りに来るのを待っている状態
    ReadyForPickup {
        delivery_person_id: Id<DeliveryPerson>,
    },
    /// 配達員が飲食店に向かっている、または飲食店で注文を受け取っている状態
    DeliveryPersonPickingUp {
        delivery_person_id: Id<DeliveryPerson>,
    },
    /// 配達員が注文をユーザーに届け、ユーザーが受け取った状態
    Delivered,
    /// ユーザー、飲食店、またはシステムによって注文がキャンセルされた状態
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
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
