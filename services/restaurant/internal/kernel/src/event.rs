use crate::{Command, Id, Item, Restaurant};

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPayload {
    AggregateCreated(Restaurant),
    ItemsAdded(Vec<Item>),
    ItemsRemoved(Vec<Id<Item>>),
}

impl From<Command> for Vec<Event> {
    fn from(value: Command) -> Self {
        let id = Id::generate();
        match value {
            Command::CrateAggregate(restaurant) => {
                vec![Event::new(id, EventPayload::AggregateCreated(restaurant))]
            }
            Command::AddItems(items) => vec![Event::new(id, EventPayload::ItemsAdded(items))],
            Command::RemoveItems(ids) => {
                vec![Event::new(id, EventPayload::ItemsRemoved(ids))]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Command, Event, EventPayload, Id, Item, Price, Restaurant};

    #[test]
    fn test_convert_command_into_event() {
        struct TestCase {
            name: &'static str,
            command: Command,
            expected: Vec<Event>,
        }
        let item_id: Id<Item> = Id::generate();
        let tests = [
            TestCase {
                name: "CreateRestaurantコマンドの場合はRestaurantCreatedイベントにのみ変換される",
                command: Command::CrateAggregate(Restaurant::new("テスト店舗".to_string())),
                expected: vec![Event::new(
                    Id::generate(),
                    EventPayload::AggregateCreated(Restaurant::new("テスト店舗".to_string())),
                )],
            },
            TestCase {
                name: "AddItemsコマンドの場合はItemsAddedイベントにのみ変換される",
                command: Command::AddItems(vec![
                    Item::new(item_id.clone(), "Food1".to_string(), Price::Yen(1000)),
                    Item::new(item_id.clone(), "Other1".to_string(), Price::Yen(500)),
                ]),
                expected: vec![Event::new(
                    Id::generate(),
                    EventPayload::ItemsAdded(vec![
                        Item::new(item_id.clone(), "Food1".to_string(), Price::Yen(1000)),
                        Item::new(item_id.clone(), "Other1".to_string(), Price::Yen(500)),
                    ]),
                )],
            },
            TestCase {
                name: "RemoveItemsコマンドの場合はItemsRemovedイベントにのみ変換される",
                command: Command::RemoveItems(vec![item_id.clone(), item_id.clone()]),
                expected: vec![Event::new(
                    Id::generate(),
                    EventPayload::ItemsRemoved(vec![item_id.clone(), item_id.clone()]),
                )],
            },
        ];
        for TestCase {
            name,
            command,
            expected,
        } in tests
        {
            let actual: Vec<Event> = command.into();
            assert_eq!(actual.len(), expected.len(), "{name}");
            for (actual, expected) in actual.iter().zip(expected.iter()) {
                assert_eq!(actual.payload(), expected.payload(), "{name}");
            }
        }
    }
}
