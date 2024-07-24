use crate::{Id, Order, OrderItem};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Aggregate {
    id: Id<Aggregate>,
    order: Order,
    order_items: Vec<OrderItem>,
}

impl Aggregate {
    #[must_use]
    pub fn new(id: Id<Aggregate>, order: Order, order_items: Vec<OrderItem>) -> Self {
        Self {
            id,
            order,
            order_items,
        }
    }

    #[must_use]
    pub fn id(&self) -> &Id<Aggregate> {
        &self.id
    }

    #[must_use]
    pub fn order(&self) -> &Order {
        &self.order
    }

    #[must_use]
    pub fn order_items(&self) -> &[OrderItem] {
        &self.order_items
    }

    #[must_use]
    pub fn total_price(&self) -> u64 {
        self.order_items
            .iter()
            .map(|x| x.price() * x.quantity())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Aggregate, Id, Order, OrderItem, OrderStatus};

    #[test]
    fn test_total_price() {
        let aggregate = Aggregate::new(
            Id::generate(),
            Order::new(
                Id::generate(),
                Id::generate(),
                String::new(),
                OrderStatus::default(),
            ),
            vec![
                OrderItem::new(Id::generate(), 1000, 5),
                OrderItem::new(Id::generate(), 500, 2),
            ],
        );
        assert_eq!(aggregate.total_price(), 6000);
    }
}
