use crate::id::Id;

use super::model::entity::{Cart, Item};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Event {
    Created { cart_id: Id<Cart>, items: Vec<Item> },
    Prepared,
    PickedUp,
    Delivered,
    Canceled,
}
