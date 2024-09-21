#[cfg(feature = "command")]
use crate::{Command, Event, EventPayload, KernelError};
use crate::{Id, Item, Restaurant};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Aggregate {
    id: Id<Aggregate>,
    restaurant: Restaurant,
    items: Vec<Item>,
    version: u64,
}

impl Aggregate {
    #[must_use]
    pub fn new(id: Id<Aggregate>, restaurant: Restaurant, items: Vec<Item>, version: u64) -> Self {
        Self {
            id,
            restaurant,
            items,
            version,
        }
    }

    #[must_use]
    pub fn id(&self) -> &Id<Aggregate> {
        &self.id
    }

    #[must_use]
    pub fn restaurant(&self) -> &Restaurant {
        &self.restaurant
    }

    #[must_use]
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// 集約にコマンドを実行する
    ///
    /// # Errors
    ///
    /// * `AggregateAlreadyCreated`: 作成済みの集約を再作成するときのエラー
    /// * `AggregateNotCreated`: 未作成の集約に集約を作成するコマンド以外を実行するときのエラー
    /// * `AggregateVersionOverflow`: バージョンが最大値の集約にコマンドを実行するときのエラー
    /// * `InvalidRestaurantName`: 飲食店名の名前が不正なときのエラー
    /// * `InvalidItemName`: 商品の名前が不正なときのエラー
    #[cfg(feature = "command")]
    pub fn apply_command(self, command: Command) -> Result<(Aggregate, Vec<Event>), KernelError> {
        if matches!(&command, Command::CrateAggregate(..)) && self.version.ne(&0) {
            return Err(KernelError::AggregateAlreadyCreated);
        }
        if !matches!(&command, Command::CrateAggregate(..)) && self.version.eq(&0) {
            return Err(KernelError::AggregateNotCreated);
        }
        if matches!(&command, Command::CrateAggregate(restaurant) if restaurant.name().is_empty()) {
            return Err(KernelError::InvalidRestaurantName);
        }
        if matches!(&command, Command::AddItems(v) if v.is_empty())
            || matches!(&command, Command::RemoveItems(v) if v.is_empty())
        {
            return Err(KernelError::EntitiesIsEmpty);
        }
        if matches!(&command, Command::AddItems(v) if v.iter().any(|x| x.name().is_empty())) {
            return Err(KernelError::InvalidItemName);
        }

        let mut aggregate = self;
        let events: Vec<Event> = command.into();
        for event in &events.clone() {
            match event.payload() {
                EventPayload::AggregateCreated(restaurant) => {
                    aggregate.restaurant = restaurant.clone();
                }
                EventPayload::ItemsAdded(items) => {
                    for item in items {
                        aggregate.items.push(item.clone());
                    }
                }
                EventPayload::ItemsRemoved(item_ids) => {
                    aggregate.items.retain(|x| !item_ids.contains(x.id()));
                }
            }
        }
        aggregate.version = aggregate
            .version
            .checked_add(1)
            .ok_or(KernelError::AggregateVersionOverflow)?;

        Ok((aggregate, events))
    }

    #[must_use]
    #[cfg(feature = "command")]
    pub fn is_conflicted(&self, version: u64) -> bool {
        version != self.version
    }
}

#[cfg(test)]
mod tests {
    use crate::{Aggregate, Command, Event, EventPayload, Id, Item, KernelError, Restaurant};

    const RESTAURANT_NAME: &str = "テスト店舗";
    const ITEM_NAME: &str = "テスト商品";

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_apply_command_ok() {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            command: Command,
            expected: (Aggregate, Vec<Event>),
        }
        let aggregate_id: Id<Aggregate> = Id::generate();
        let item_id: Id<Item> = Id::generate();
        let tests = [
            TestCase {
                name: "未作成の集約に集約作成コマンド実行時は集約と集約作成イベントが返る",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Restaurant::new(String::default()),
                    Vec::default(),
                    u64::default(),
                ),
                command: Command::CrateAggregate(Restaurant::new(RESTAURANT_NAME.to_string())),
                expected: (
                    Aggregate::new(
                        aggregate_id.clone(),
                        Restaurant::new(RESTAURANT_NAME.to_string()),
                        vec![],
                        1,
                    ),
                    vec![Event::new(
                        Id::generate(),
                        EventPayload::AggregateCreated(Restaurant::new(
                            RESTAURANT_NAME.to_string(),
                        )),
                    )],
                ),
            },
            TestCase {
                name: "作成済集約に商品追加コマンド実行時は集約に商品が追加され、イベントが返る",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Restaurant::new(RESTAURANT_NAME.to_string()),
                    vec![],
                    1,
                ),
                command: Command::AddItems(vec![Item::new(
                    item_id.clone(),
                    ITEM_NAME.to_string(),
                    1000,
                )]),
                expected: (
                    Aggregate::new(
                        aggregate_id.clone(),
                        Restaurant::new(RESTAURANT_NAME.to_string()),
                        vec![Item::new(item_id.clone(), ITEM_NAME.to_string(), 1000)],
                        2,
                    ),
                    vec![Event::new(
                        Id::generate(),
                        EventPayload::ItemsAdded(vec![Item::new(
                            item_id.clone(),
                            ITEM_NAME.to_string(),
                            1000,
                        )]),
                    )],
                ),
            },
            TestCase {
                name:
                    "作成済集約に複数商品の追加コマンド実行時は集約に商品が追加され、イベントが返る",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Restaurant::new(RESTAURANT_NAME.to_string()),
                    vec![],
                    1,
                ),
                command: Command::AddItems(vec![
                    Item::new(item_id.clone(), ITEM_NAME.to_string(), 1000),
                    Item::new(item_id.clone(), "商品2".to_string(), 500),
                ]),
                expected: (
                    Aggregate::new(
                        aggregate_id.clone(),
                        Restaurant::new(RESTAURANT_NAME.to_string()),
                        vec![
                            Item::new(item_id.clone(), ITEM_NAME.to_string(), 1000),
                            Item::new(item_id.clone(), "商品2".to_string(), 500),
                        ],
                        2,
                    ),
                    vec![Event::new(
                        Id::generate(),
                        EventPayload::ItemsAdded(vec![
                            Item::new(item_id.clone(), ITEM_NAME.to_string(), 1000),
                            Item::new(item_id.clone(), "商品2".to_string(), 500),
                        ]),
                    )],
                ),
            },
            TestCase {
                name: "すでに商品のある集約に商品追加コマンド実行時は集約に商品が追加される",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Restaurant::new(RESTAURANT_NAME.to_string()),
                    vec![Item::new(item_id.clone(), ITEM_NAME.to_string(), 1000)],
                    2,
                ),
                command: Command::AddItems(vec![Item::new(
                    item_id.clone(),
                    "商品2".to_string(),
                    500,
                )]),
                expected: (
                    Aggregate::new(
                        aggregate_id.clone(),
                        Restaurant::new(RESTAURANT_NAME.to_string()),
                        vec![
                            Item::new(item_id.clone(), ITEM_NAME.to_string(), 1000),
                            Item::new(item_id.clone(), "商品2".to_string(), 500),
                        ],
                        3,
                    ),
                    vec![Event::new(
                        Id::generate(),
                        EventPayload::ItemsAdded(vec![Item::new(
                            item_id.clone(),
                            "商品2".to_string(),
                            500,
                        )]),
                    )],
                ),
            },
            TestCase {
                name: "すでに商品のある集約に商品削除コマンド実行時は集約から商品が削除される",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Restaurant::new(RESTAURANT_NAME.to_string()),
                    vec![
                        Item::new(item_id.clone(), ITEM_NAME.to_string(), 1000),
                        Item::new(item_id.clone(), "商品2".to_string(), 500),
                    ],
                    2,
                ),
                command: Command::RemoveItems(vec![item_id.clone()]),
                expected: (
                    Aggregate::new(
                        aggregate_id.clone(),
                        Restaurant::new(RESTAURANT_NAME.to_string()),
                        vec![],
                        3,
                    ),
                    vec![Event::new(
                        Id::generate(),
                        EventPayload::ItemsRemoved(vec![item_id.clone()]),
                    )],
                ),
            },
        ];
        for TestCase {
            name,
            aggregate,
            command,
            expected,
        } in tests
        {
            let result = aggregate.apply_command(command);
            assert!(result.is_ok(), "{name}: apply_command must be ok");
            let actual = result.unwrap();
            assert_eq!(actual.0, expected.0, "{name}: Aggregate");
            assert_eq!(actual.1.len(), expected.1.len(), "{name}: Events length");
            for (actual, expected) in actual.1.iter().zip(expected.1.iter()) {
                assert_eq!(actual.payload(), expected.payload(), "{name}: EventPayload");
            }
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_apply_command_err() {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            command: Command,
            #[allow(clippy::type_complexity)]
            assert: fn(name: &str, actual: Result<(Aggregate, Vec<Event>), KernelError>),
        }
        let tests = [
            TestCase {
                name:
                    "作成済みの集約に集約作成コマンドを実行時はAggregateAlreadyCreatedエラーが返る",
                aggregate: Aggregate {
                    id: Id::generate(),
                    restaurant: Restaurant::new(RESTAURANT_NAME.to_string()),
                    items: vec![],
                    version: 1,
                },
                command: Command::CrateAggregate(Restaurant::new(RESTAURANT_NAME.to_string())),
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::AggregateAlreadyCreated)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "未作成集約に集約作成コマンド以外を実行時はAggregateNotCreatedエラーが返る",
                aggregate: Aggregate {
                    id: Id::generate(),
                    restaurant: Restaurant::new(String::new()),
                    items: vec![],
                    version: 0,
                },
                command: Command::AddItems(vec![Item::new(
                    Id::generate(),
                    ITEM_NAME.to_string(),
                    1000,
                )]),
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::AggregateNotCreated)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name:
                    "集約作成コマンド実行時に店名が空文字の場合はInvaildRestaurantNameエラーが返る",
                aggregate: Aggregate {
                    id: Id::generate(),
                    restaurant: Restaurant::new(String::new()),
                    items: vec![],
                    version: 0,
                },
                command: Command::CrateAggregate(Restaurant::new(String::new())),
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::InvalidRestaurantName)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "商品追加コマンドを実行時に商品名が空文字の場合はInvalidItemNameエラーが返る",
                aggregate: Aggregate {
                    id: Id::generate(),
                    restaurant: Restaurant::new(RESTAURANT_NAME.to_string()),
                    items: vec![],
                    version: 1,
                },
                command: Command::AddItems(vec![
                    Item::new(Id::generate(), ITEM_NAME.to_string(), 1000),
                    Item::new(Id::generate(), String::new(), 1000),
                ]),
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::InvalidItemName)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name:
                    "バージョンが最大値の集約にコマンド実行時にAggregateVersionOverflowエラーが返る",
                aggregate: Aggregate {
                    id: Id::generate(),
                    restaurant: Restaurant::new(RESTAURANT_NAME.to_string()),
                    items: vec![],
                    version: u64::MAX,
                },
                command: Command::AddItems(vec![Item::new(
                    Id::generate(),
                    ITEM_NAME.to_string(),
                    1000,
                )]),
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::AggregateVersionOverflow)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "配列が空な商品追加コマンド実行時にEntitiesIsEmptyエラーが返る",
                aggregate: Aggregate {
                    id: Id::generate(),
                    restaurant: Restaurant::new(RESTAURANT_NAME.to_string()),
                    items: vec![],
                    version: 1,
                },
                command: Command::AddItems(vec![]),
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::EntitiesIsEmpty)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "配列が空な商品削除コマンド実行時にEntitiesIsEmptyエラーが返る",
                aggregate: Aggregate {
                    id: Id::generate(),
                    restaurant: Restaurant::new(RESTAURANT_NAME.to_string()),
                    items: vec![],
                    version: 1,
                },
                command: Command::RemoveItems(vec![]),
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::EntitiesIsEmpty)),
                        "{name}"
                    );
                },
            },
        ];
        for TestCase {
            name,
            aggregate,
            command,
            assert,
        } in tests
        {
            let actual = aggregate.apply_command(command);
            assert(name, actual);
        }
    }
}
