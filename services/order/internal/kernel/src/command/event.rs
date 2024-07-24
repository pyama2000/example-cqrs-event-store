use crate::{Command, DeliveryPerson, Id, Order, OrderItem};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    id: Id<Event>,
    payload: EventPayload,
}

impl Event {
    #[must_use]
    pub fn new(id: Id<Event>, payload: EventPayload) -> Self {
        Self { id, payload }
    }

    #[must_use]
    pub fn id(&self) -> &Id<Event> {
        &self.id
    }

    #[must_use]
    pub fn payload(&self) -> &EventPayload {
        &self.payload
    }
}

impl From<Command> for Vec<Event> {
    fn from(value: Command) -> Self {
        match value {
            Command::Receive { order, items } => {
                vec![Event::new(
                    Id::generate(),
                    EventPayload::Received { order, items },
                )]
            }
            Command::Prepare => vec![Event::new(Id::generate(), EventPayload::PreparationStarted)],
            Command::AssigningDeliveryPerson { delivery_person_id } => {
                vec![Event::new(
                    Id::generate(),
                    EventPayload::DeliveryPersonAssigned { delivery_person_id },
                )]
            }
            Command::ReadyForPickup => {
                vec![Event::new(Id::generate(), EventPayload::ReadyForPickup)]
            }
            Command::DeliveryPersonPickingUp => vec![Event::new(
                Id::generate(),
                EventPayload::DeliveryPersonPickedUp,
            )],
            Command::Delivered => vec![Event::new(Id::generate(), EventPayload::Delivered)],
            Command::Cancel => vec![Event::new(Id::generate(), EventPayload::Cancelled)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPayload {
    Received {
        order: Order,
        items: Vec<OrderItem>,
    },
    PreparationStarted,
    DeliveryPersonAssigned {
        delivery_person_id: Id<DeliveryPerson>,
    },
    ReadyForPickup,
    DeliveryPersonPickedUp,
    Delivered,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use crate::{Command, Event, EventPayload, Id, Order, OrderItem, OrderStatus};

    #[test]
    fn test_convert_command_into_event() {
        struct TestCase {
            name: &'static str,
            command: Command,
            expected: Vec<Event>,
        }
        let restaurant_id = Id::generate();
        let user_id = Id::generate();
        let item_id = Id::generate();
        let delivery_person_id = Id::generate();
        let tests = [
            TestCase {
                name: "Receiveコマンドの場合はReceivedイベントにのみ変換される",
                command: Command::Receive {
                    order: Order::new(
                        restaurant_id.clone(),
                        user_id.clone(),
                        String::new(),
                        OrderStatus::Received,
                    ),
                    items: vec![OrderItem::new(item_id.clone(), 1000, 5)],
                },
                expected: vec![Event::new(
                    Id::generate(),
                    EventPayload::Received {
                        order: Order::new(
                            restaurant_id.clone(),
                            user_id.clone(),
                            String::new(),
                            OrderStatus::Received,
                        ),
                        items: vec![OrderItem::new(item_id.clone(), 1000, 5)],
                    },
                )],
            },
            TestCase {
                name: "Prepareコマンドの場合はPreparationStartedイベントにのみ変換される",
                command: Command::Prepare,
                expected: vec![Event::new(Id::generate(), EventPayload::PreparationStarted)],
            },
            TestCase {
                name: "AssigningDeliveryPersonコマンド→DeliveryPersonAssignedイベント",
                command: Command::AssigningDeliveryPerson {
                    delivery_person_id: delivery_person_id.clone(),
                },
                expected: vec![Event::new(
                    Id::generate(),
                    EventPayload::DeliveryPersonAssigned {
                        delivery_person_id: delivery_person_id.clone(),
                    },
                )],
            },
            TestCase {
                name: "ReadyForPickupコマンドの場合はReadyForPickupイベントにのみ変換される",
                command: Command::ReadyForPickup,
                expected: vec![Event::new(Id::generate(), EventPayload::ReadyForPickup)],
            },
            TestCase {
                name: "DeliveryPersonPickingUpコマンド→DeliveryPersonPickedUpイベント",
                command: Command::DeliveryPersonPickingUp,
                expected: vec![Event::new(
                    Id::generate(),
                    EventPayload::DeliveryPersonPickedUp,
                )],
            },
            TestCase {
                name: "Deliveredコマンドの場合はDeliveredイベントにのみ変換される",
                command: Command::Delivered,
                expected: vec![Event::new(Id::generate(), EventPayload::Delivered)],
            },
            TestCase {
                name: "Cancelコマンドの場合はCancelledイベントにのみ変換される",
                command: Command::Cancel,
                expected: vec![Event::new(Id::generate(), EventPayload::Cancelled)],
            },
        ];
        for TestCase {
            name,
            command,
            expected,
        } in tests
        {
            let actuals: Vec<Event> = command.into();
            assert_eq!(actuals.len(), expected.len(), "{name}");
            for (actual, expected) in actuals.iter().zip(expected.iter()) {
                assert_eq!(actual.payload(), expected.payload(), "{name}");
            }
        }
    }
}
