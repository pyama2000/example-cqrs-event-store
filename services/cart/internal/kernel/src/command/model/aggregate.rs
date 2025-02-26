use std::collections::HashMap;

use crate::command::command::Command;
use crate::command::error::CommandKernelError;
use crate::command::event::Event;
use crate::id::Id;

use super::entity::{Item, Tenant};

pub trait ApplyCommand {
    /// 集約にコマンドを実行する
    ///
    /// 集約にコマンドを実行すると、コマンドに応じて集約の状態を変更し、集約の状態を変更したイベントを返す
    ///
    /// # Errors
    ///
    /// コマンド実行時にドメインエラーが発生したら [`CommandKernelError`] を成功状態で返し、例外エラーが発生したら [`anyhow::Error`] を返す
    ///
    /// [`anyhow::Error`]: https://docs.rs/anyhow/latest/anyhow/struct.Error.html
    fn apply_command(
        &mut self,
        command: Command,
    ) -> Result<Result<Vec<Event>, CommandKernelError>, anyhow::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Aggregate {
    id: Id<Aggregate>,
    items: HashMap<Id<Tenant>, HashMap<Id<Item>, u32>>,
    is_order_placed: bool,
    /// 集約のバージョン
    version: u128,
}

impl Aggregate {
    #[must_use]
    pub fn new(
        id: Id<Aggregate>,
        items: HashMap<Id<Tenant>, HashMap<Id<Item>, u32>>,
        is_order_placed: bool,
        version: u128,
    ) -> Self {
        Self {
            id,
            items,
            is_order_placed,
            version,
        }
    }

    /// 集約のID
    #[must_use]
    pub fn id(&self) -> &Id<Aggregate> {
        &self.id
    }

    /// テナントごとの商品
    #[must_use]
    pub fn items(&self) -> &HashMap<Id<Tenant>, HashMap<Id<Item>, u32>> {
        &self.items
    }

    /// 集約のバージョン
    #[must_use]
    pub fn version(&self) -> u128 {
        self.version
    }
}

impl ApplyCommand for Aggregate {
    fn apply_command(
        &mut self,
        command: Command,
    ) -> Result<Result<Vec<Event>, CommandKernelError>, anyhow::Error> {
        use anyhow::Context as _;

        if command != Command::Create && self.version == 0 {
            return Ok(Err(CommandKernelError::AggregateNotCreated));
        }
        if self.is_order_placed {
            return Ok(Err(CommandKernelError::OrderAlreadyPlaced));
        }

        let events: Vec<Event> = match command {
            Command::Create => {
                if self.version != 0 {
                    return Ok(Err(CommandKernelError::AggregateAlreadyCreated));
                }
                vec![Event::Created]
            }
            Command::AddItem { tenant_id, item_id } => {
                let quantity_by_item_id = self
                    .items
                    .entry(tenant_id.clone())
                    .or_insert(HashMap::from([(item_id.clone(), 0)]));
                quantity_by_item_id
                    .entry(item_id.clone())
                    .and_modify(|quantity| *quantity += 1)
                    .or_insert(1);
                vec![Event::ItemAdded { tenant_id, item_id }]
            }
            Command::RemoveItem { tenant_id, item_id } => {
                if let Some(quantity_by_item_id) = self.items.get_mut(&tenant_id) {
                    if let Some(quantity) = quantity_by_item_id.get_mut(&item_id) {
                        *quantity -= 1;
                        if *quantity == 0 {
                            quantity_by_item_id.remove(&item_id);
                            if quantity_by_item_id.is_empty() {
                                self.items.remove(&tenant_id);
                            }
                        }
                    } else {
                        return Ok(Err(CommandKernelError::ItemNotFound));
                    }
                } else {
                    return Ok(Err(CommandKernelError::ItemNotFound));
                }
                vec![Event::ItemRemoved { tenant_id, item_id }]
            }
            Command::PlaceOrder => {
                if self.items.is_empty() {
                    return Ok(Err(CommandKernelError::PlaceOrder {
                        message: "item is empty".to_string(),
                    }));
                }
                self.is_order_placed = true;
                vec![Event::OrderPlaced]
            }
        };

        self.version = self
            .version
            .checked_add(1)
            .with_context(|| "Cannot update aggregate version")?;
        Ok(Ok(events))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::Result;

    use crate::command::command::Command;
    use crate::command::error::CommandKernelError;
    use crate::command::event::Event;
    use crate::command::model::aggregate::{Aggregate, ApplyCommand as _};
    use crate::command::model::entity::{Item, Tenant};
    use crate::id::Id;

    #[test]
    fn test_apply_command_create_ok() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            expected_aggregate: Aggregate,
            expected_events: Vec<Event>,
        }

        let aggregate_id: Id<Aggregate> = Id::generate();

        let tests = [TestCase {
            name: "未作成の集約にCreate実行時は集約が更新されCreatedが返る",
            aggregate: Aggregate {
                id: aggregate_id.clone(),
                ..Default::default()
            },
            expected_aggregate: Aggregate {
                id: aggregate_id.clone(),
                items: HashMap::new(),
                is_order_placed: false,
                version: 1,
            },
            expected_events: vec![Event::Created],
        }];

        for TestCase {
            name,
            mut aggregate,
            expected_aggregate,
            expected_events,
        } in tests
        {
            let actual_events = aggregate.apply_command(Command::Create)??;
            pretty_assertions::assert_eq!(
                actual_events,
                expected_events,
                "{name}: events not equaled"
            );
            pretty_assertions::assert_eq!(
                aggregate,
                expected_aggregate,
                "{name}: aggregate not equaled"
            );
        }

        Ok(())
    }

    #[test]
    fn test_apply_command_create_err() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            expected: CommandKernelError,
        }

        let tests = [TestCase {
            name: "作成済みの集約にCreate実行時はAggregateAlreadyCreatedが返る",
            aggregate: Aggregate {
                version: 1,
                ..Default::default()
            },
            expected: CommandKernelError::AggregateAlreadyCreated,
        }];

        for TestCase {
            name,
            mut aggregate,
            expected,
        } in tests
        {
            let actual = aggregate.apply_command(Command::Create)?;
            assert_eq!(actual, Err(expected), "{name}");
        }
        Ok(())
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_apply_command_add_item_ok() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            command: Command,
            expected_aggregate: Aggregate,
            expected_events: Vec<Event>,
        }

        let aggregate_id: Id<Aggregate> = Id::generate();
        let tenant_id_1: Id<Tenant> = Id::generate();
        let tenant_id_2: Id<Tenant> = Id::generate();
        let item_id_1: Id<Item> = Id::generate();
        let item_id_2: Id<Item> = Id::generate();

        let tests = [
            TestCase {
                name: "AddItem実行時は集約に商品が追加されItemAddedが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    version: 1,
                    ..Default::default()
                },
                command: Command::AddItem {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_1.clone(),
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([(
                        tenant_id_1.clone(),
                        HashMap::from_iter([(item_id_1.clone(), 1)]),
                    )]),
                    is_order_placed: false,
                    version: 2,
                },
                expected_events: vec![Event::ItemAdded {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_1.clone(),
                }],
            },
            TestCase {
                name: "同一の商品をAddItem実行時は集約に商品が追加されItemAddedが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([(
                        tenant_id_1.clone(),
                        HashMap::from_iter([(item_id_1.clone(), 1)]),
                    )]),
                    is_order_placed: false,
                    version: 2,
                },
                command: Command::AddItem {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_1.clone(),
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([(
                        tenant_id_1.clone(),
                        HashMap::from_iter([(item_id_1.clone(), 2)]),
                    )]),
                    is_order_placed: false,
                    version: 3,
                },
                expected_events: vec![Event::ItemAdded {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_1.clone(),
                }],
            },
            TestCase {
                name: "同一テナントにAddItem実行時は集約に商品が追加されItemAddedが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([(
                        tenant_id_1.clone(),
                        HashMap::from_iter([(item_id_1.clone(), 1)]),
                    )]),
                    is_order_placed: false,
                    version: 2,
                },
                command: Command::AddItem {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_2.clone(),
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([(
                        tenant_id_1.clone(),
                        HashMap::from_iter([(item_id_1.clone(), 1), (item_id_2.clone(), 1)]),
                    )]),
                    is_order_placed: false,
                    version: 3,
                },
                expected_events: vec![Event::ItemAdded {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_2.clone(),
                }],
            },
            TestCase {
                name: "別テナントでAddItem実行時は集約に商品が追加されItemAddedが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([(
                        tenant_id_1.clone(),
                        HashMap::from_iter([(item_id_1.clone(), 1)]),
                    )]),
                    is_order_placed: false,
                    version: 2,
                },
                command: Command::AddItem {
                    tenant_id: tenant_id_2.clone(),
                    item_id: item_id_2.clone(),
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([
                        (
                            tenant_id_1.clone(),
                            HashMap::from_iter([(item_id_1.clone(), 1)]),
                        ),
                        (
                            tenant_id_2.clone(),
                            HashMap::from_iter([(item_id_2.clone(), 1)]),
                        ),
                    ]),
                    is_order_placed: false,
                    version: 3,
                },
                expected_events: vec![Event::ItemAdded {
                    tenant_id: tenant_id_2.clone(),
                    item_id: item_id_2.clone(),
                }],
            },
        ];

        for TestCase {
            name,
            mut aggregate,
            command,
            expected_aggregate,
            expected_events,
        } in tests
        {
            let actual_events = aggregate.apply_command(command)??;
            pretty_assertions::assert_eq!(
                actual_events,
                expected_events,
                "{name}: events not equaled"
            );
            pretty_assertions::assert_eq!(
                aggregate,
                expected_aggregate,
                "{name}: aggregate not equaled"
            );
        }

        Ok(())
    }

    #[test]
    fn test_apply_command_add_item_err() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            command: Command,
            expected: CommandKernelError,
        }

        let tests = [
            TestCase {
                name: "未作成の集約にAddItem実行時はAggregateNotCreatedが返る",
                aggregate: Aggregate::default(),
                command: Command::AddItem {
                    tenant_id: Id::generate(),
                    item_id: Id::generate(),
                },
                expected: CommandKernelError::AggregateNotCreated,
            },
            TestCase {
                name: "カートの商品が注文済みの場合はOrderAlreadyPlacedが返る",
                aggregate: Aggregate {
                    is_order_placed: true,
                    version: 1,
                    ..Default::default()
                },
                command: Command::AddItem {
                    tenant_id: Id::generate(),
                    item_id: Id::generate(),
                },
                expected: CommandKernelError::OrderAlreadyPlaced,
            },
        ];

        for TestCase {
            name,
            mut aggregate,
            command,
            expected,
        } in tests
        {
            let actual = aggregate.apply_command(command)?;
            assert_eq!(actual, Err(expected), "{name}");
        }

        Ok(())
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_apply_command_remove_item_ok() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            command: Command,
            expected_aggregate: Aggregate,
            expected_events: Vec<Event>,
        }

        let aggregate_id: Id<Aggregate> = Id::generate();
        let tenant_id_1: Id<Tenant> = Id::generate();
        let tenant_id_2: Id<Tenant> = Id::generate();
        let item_id_1: Id<Item> = Id::generate();
        let item_id_2: Id<Item> = Id::generate();

        let tests = [
            TestCase {
                name: "RemoveItem実行時に集約の商品が削除されItemRemovedが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([(
                        tenant_id_1.clone(),
                        HashMap::from_iter([(item_id_1.clone(), 1)]),
                    )]),
                    is_order_placed: false,
                    version: 2,
                },
                command: Command::RemoveItem {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_1.clone(),
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::new(),
                    is_order_placed: false,
                    version: 3,
                },
                expected_events: vec![Event::ItemRemoved {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_1.clone(),
                }],
            },
            TestCase {
                name: "複数商品があるときにRemoveItem実行時に集約の商品が削除されItemRemovedが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([
                        (
                            tenant_id_1.clone(),
                            HashMap::from_iter([(item_id_1.clone(), 1)]),
                        ),
                        (
                            tenant_id_2.clone(),
                            HashMap::from_iter([(item_id_2.clone(), 1)]),
                        ),
                    ]),
                    is_order_placed: false,
                    version: 3,
                },
                command: Command::RemoveItem {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_1.clone(),
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([(
                        tenant_id_2.clone(),
                        HashMap::from_iter([(item_id_2.clone(), 1)]),
                    )]),
                    is_order_placed: false,
                    version: 4,
                },
                expected_events: vec![Event::ItemRemoved {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_1.clone(),
                }],
            },
            TestCase {
                name: "複数商品があるときにRemoveItem実行時に集約の商品が削除されItemRemovedが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([(
                        tenant_id_1.clone(),
                        HashMap::from_iter([(item_id_1.clone(), 1), (item_id_2.clone(), 1)]),
                    )]),
                    is_order_placed: false,
                    version: 3,
                },
                command: Command::RemoveItem {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_1.clone(),
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([(
                        tenant_id_1.clone(),
                        HashMap::from_iter([(item_id_2.clone(), 1)]),
                    )]),
                    is_order_placed: false,
                    version: 4,
                },
                expected_events: vec![Event::ItemRemoved {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_1.clone(),
                }],
            },
            TestCase {
                name: "複数商品があるときにRemoveItem実行時に集約の商品が削除されItemRemovedが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([(
                        tenant_id_1.clone(),
                        HashMap::from_iter([(item_id_1.clone(), 2)]),
                    )]),
                    is_order_placed: false,
                    version: 3,
                },
                command: Command::RemoveItem {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_1.clone(),
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    items: HashMap::from_iter([(
                        tenant_id_1.clone(),
                        HashMap::from_iter([(item_id_1.clone(), 1)]),
                    )]),
                    is_order_placed: false,
                    version: 4,
                },
                expected_events: vec![Event::ItemRemoved {
                    tenant_id: tenant_id_1.clone(),
                    item_id: item_id_1.clone(),
                }],
            },
        ];

        for TestCase {
            name,
            mut aggregate,
            command,
            expected_aggregate,
            expected_events,
        } in tests
        {
            let actual_events = aggregate.apply_command(command)??;
            pretty_assertions::assert_eq!(
                actual_events,
                expected_events,
                "{name}: events not equaled"
            );
            pretty_assertions::assert_eq!(
                aggregate,
                expected_aggregate,
                "{name}: aggregate not equaled"
            );
        }

        Ok(())
    }

    #[test]
    fn test_apply_command_remove_item_err() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            command: Command,
            expected: CommandKernelError,
        }

        let tests = [
            TestCase {
                name: "未作成の集約にRemoveItem実行時はAggregateNotCreatedが返る",
                aggregate: Aggregate::default(),
                command: Command::RemoveItem {
                    tenant_id: Id::generate(),
                    item_id: Id::generate(),
                },
                expected: CommandKernelError::AggregateNotCreated,
            },
            TestCase {
                name: "テナントIDがないRemoveItem実行時はItemNotFoundが返る",
                aggregate: Aggregate {
                    items: HashMap::from([(Id::generate(), HashMap::from([(Id::generate(), 1)]))]),
                    version: 1,
                    ..Default::default()
                },
                command: Command::RemoveItem {
                    tenant_id: Id::generate(),
                    item_id: Id::generate(),
                },
                expected: CommandKernelError::ItemNotFound,
            },
            {
                let tenant_id: Id<Tenant> = Id::generate();
                TestCase {
                    name: "商品IDがないRemoveItem実行時はItemNotFoundが返る",
                    aggregate: Aggregate {
                        items: HashMap::from([(
                            tenant_id.clone(),
                            HashMap::from([(Id::generate(), 1)]),
                        )]),
                        version: 1,
                        ..Default::default()
                    },
                    command: Command::RemoveItem {
                        tenant_id,
                        item_id: Id::generate(),
                    },
                    expected: CommandKernelError::ItemNotFound,
                }
            },
            TestCase {
                name: "カートの商品が注文済みの場合はOrderAlreadyPlacedが返る",
                aggregate: Aggregate {
                    is_order_placed: true,
                    version: 1,
                    ..Default::default()
                },
                command: Command::RemoveItem {
                    tenant_id: Id::generate(),
                    item_id: Id::generate(),
                },
                expected: CommandKernelError::OrderAlreadyPlaced,
            },
        ];

        for TestCase {
            name,
            mut aggregate,
            command,
            expected,
        } in tests
        {
            let actual = aggregate.apply_command(command)?;
            assert_eq!(actual, Err(expected), "{name}");
        }

        Ok(())
    }

    #[test]
    fn test_apply_command_place_order_ok() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            expected_aggregate: Aggregate,
            expected_events: Vec<Event>,
        }

        let aggregate_id: Id<Aggregate> = Id::generate();
        let tenant_id: Id<Tenant> = Id::generate();
        let item_id: Id<Item> = Id::generate();

        let tests = [TestCase {
            name: "PlaceOrder実行時は集約が更新されOrderPlacedが返る",
            aggregate: Aggregate {
                id: aggregate_id.clone(),
                items: HashMap::from_iter([(
                    tenant_id.clone(),
                    HashMap::from_iter([(item_id.clone(), 1)]),
                )]),
                is_order_placed: false,
                version: 2,
            },
            expected_aggregate: Aggregate {
                id: aggregate_id.clone(),
                items: HashMap::from_iter([(
                    tenant_id.clone(),
                    HashMap::from_iter([(item_id.clone(), 1)]),
                )]),
                is_order_placed: true,
                version: 3,
            },
            expected_events: vec![Event::OrderPlaced],
        }];

        for TestCase {
            name,
            mut aggregate,
            expected_aggregate,
            expected_events,
        } in tests
        {
            let actual_events = aggregate.apply_command(Command::PlaceOrder)??;
            pretty_assertions::assert_eq!(
                actual_events,
                expected_events,
                "{name}: events not equaled"
            );
            pretty_assertions::assert_eq!(
                aggregate,
                expected_aggregate,
                "{name}: aggregate not equaled"
            );
        }

        Ok(())
    }

    #[test]
    fn test_apply_command_place_order_err() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            expected: CommandKernelError,
        }

        let tests = [
            TestCase {
                name: "未作成の集約にPlaceOrder実行時はAggregateNotCreatedが返る",
                aggregate: Aggregate::default(),
                expected: CommandKernelError::AggregateNotCreated,
            },
            TestCase {
                name: "カートに商品がない場合にPlaceOrder実行時はPlaceOrderが返る",
                aggregate: Aggregate {
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::PlaceOrder {
                    message: "item is empty".to_string(),
                },
            },
            TestCase {
                name: "カートの商品が注文済みの場合はOrderAlreadyPlacedが返る",
                aggregate: Aggregate {
                    is_order_placed: true,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::OrderAlreadyPlaced,
            },
        ];

        for TestCase {
            name,
            mut aggregate,
            expected,
        } in tests
        {
            let actual = aggregate.apply_command(Command::PlaceOrder)?;
            assert_eq!(actual, Err(expected), "{name}");
        }

        Ok(())
    }

    #[test]
    fn test_apply_command_exception() {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            command: Command,
            expected: anyhow::Error,
        }

        let tests = [TestCase {
            name: "バージョンが最大値の集約にコマンド実行時はエラーが返る",
            aggregate: Aggregate {
                version: u128::MAX,
                ..Default::default()
            },
            command: Command::AddItem {
                tenant_id: Id::generate(),
                item_id: Id::generate(),
            },
            expected: anyhow::anyhow!("Cannot update aggregate version"),
        }];

        for TestCase {
            name,
            mut aggregate,
            command,
            expected,
        } in tests
        {
            let actual = aggregate.apply_command(command);
            assert_eq!(
                actual.err().unwrap().to_string(),
                expected.to_string(),
                "{name}"
            );
        }
    }
}
