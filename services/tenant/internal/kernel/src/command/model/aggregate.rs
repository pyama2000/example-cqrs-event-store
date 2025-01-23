use tracing::instrument;

use crate::{Command, CommandKernelError, Event, Id};

use super::Item;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Aggregate {
    id: Id<Aggregate>,
    name: String,
    items: Vec<Item>,
    /// 集約のバージョン
    version: u64,
}

impl Aggregate {
    #[must_use]
    pub fn new(id: Id<Aggregate>, name: String, items: Vec<Item>, version: u64) -> Self {
        Self {
            id,
            name,
            items,
            version,
        }
    }

    /// 集約のID
    #[must_use]
    pub fn id(&self) -> &Id<Aggregate> {
        &self.id
    }

    /// テナント名
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 保有している商品の一覧
    #[must_use]
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// 集約のバージョン
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// 集約にコマンドを実行する
    ///
    /// # Errors
    #[instrument(skip(self), err, ret)]
    pub fn apply_command(&mut self, command: Command) -> Result<Vec<Event>, CommandKernelError> {
        if !matches!(&command, Command::Create { .. }) && self.version == 0 {
            return Err(CommandKernelError::AggregateNotCreated);
        }
        if matches!(&command, Command::Create { .. }) && self.version != 0 {
            return Err(CommandKernelError::AggregateAlreadyCreated);
        }
        if matches!(&command, Command::Create { name } if name.is_empty()) {
            return Err(CommandKernelError::InvalidTenantName);
        }
        if matches!(&command, Command::AddItems { items } if items.is_empty()) {
            return Err(CommandKernelError::EmptyItems);
        }
        if matches!(&command, Command::AddItems { items } if items.iter().any(|x| x.name().is_empty()))
        {
            return Err(CommandKernelError::InvalidItemName);
        }
        if matches!(&command, Command::RemoveItems { item_ids } if item_ids.is_empty()) {
            return Err(CommandKernelError::EmptyItemIds);
        }

        let events: Vec<Event> = command.into();
        for payload in &events {
            match payload {
                Event::Created { name } => self.name = name.to_string(),
                Event::ItemsAdded { items } => {
                    for item in items {
                        self.items.push(item.clone());
                    }
                }
                Event::ItemsRemoved { item_ids } => {
                    self.items.retain(|x| !item_ids.contains(x.id()));
                }
            }
        }
        self.version = self
            .version
            .checked_add(1)
            .ok_or_else(|| CommandKernelError::AggregateVersionOverflowed)?;
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Aggregate, Command, CommandKernelError, Event, Id, Item};

    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_apply_command_ok() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            command: Command,
            expected_aggregate: Aggregate,
            expected_events: Vec<Event>,
        }
        let aggregate_id: Id<Aggregate> = Id::generate();
        let item_id: Id<Item> = Id::generate();
        let item_id_2: Id<Item> = Id::generate();
        let tenant_name = "テストテナント".to_string();
        let item_name = "テスト商品".to_string();
        let tests = [
            TestCase {
                name: "未作成の集約に集約作成コマンド実行時は集約と集約作成イベントが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    ..Default::default()
                },
                command: Command::Create {
                    name: tenant_name.clone(),
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    name: tenant_name.clone(),
                    items: Vec::new(),
                    version: 1,
                },
                expected_events: vec![Event::Created {
                    name: tenant_name.clone(),
                }],
            },
            TestCase {
                name: "商品追加コマンド実行時、集約に商品が追加され商品追加イベントが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    name: tenant_name.clone(),
                    items: Vec::new(),
                    version: 1,
                },
                command: Command::AddItems {
                    items: vec![Item::new(item_id.clone(), item_name.clone(), 1000)],
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    name: tenant_name.clone(),
                    items: vec![Item::new(item_id.clone(), item_name.clone(), 1000)],
                    version: 2,
                },
                expected_events: vec![Event::ItemsAdded {
                    items: vec![Item::new(item_id.clone(), item_name.clone(), 1000)],
                }],
            },
            TestCase {
                name: "商品追加コマンド実行時、集約に商品が追加され商品追加イベントが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    name: tenant_name.clone(),
                    items: vec![Item::new(item_id.clone(), item_name.clone(), 1000)],
                    version: 2,
                },
                command: Command::AddItems {
                    items: vec![Item::new(item_id.clone(), item_name.clone(), 2000)],
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    name: tenant_name.clone(),
                    items: vec![
                        Item::new(item_id.clone(), item_name.clone(), 1000),
                        Item::new(item_id.clone(), item_name.clone(), 2000),
                    ],
                    version: 3,
                },
                expected_events: vec![Event::ItemsAdded {
                    items: vec![Item::new(item_id.clone(), item_name.clone(), 2000)],
                }],
            },
            TestCase {
                name: "商品削除コマンド実行時、集約から商品が削除され商品削除イベントが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    name: tenant_name.clone(),
                    items: vec![
                        Item::new(item_id.clone(), item_name.clone(), 1000),
                        Item::new(item_id_2.clone(), item_name.clone(), 2000),
                    ],
                    version: 2,
                },
                command: Command::RemoveItems {
                    item_ids: vec![item_id.clone()],
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    name: tenant_name.clone(),
                    items: vec![Item::new(item_id_2.clone(), item_name.clone(), 2000)],
                    version: 3,
                },
                expected_events: vec![Event::ItemsRemoved {
                    item_ids: vec![item_id.clone()],
                }],
            },
            TestCase {
                name: "商品削除コマンド実行時、集約から商品が削除され商品削除イベントが返る",
                aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    name: tenant_name.clone(),
                    items: vec![
                        Item::new(item_id.clone(), item_name.clone(), 1000),
                        Item::new(item_id_2.clone(), item_name.clone(), 2000),
                    ],
                    version: 2,
                },
                command: Command::RemoveItems {
                    item_ids: vec![item_id.clone(), item_id_2.clone()],
                },
                expected_aggregate: Aggregate {
                    id: aggregate_id.clone(),
                    name: tenant_name.clone(),
                    items: Vec::new(),
                    version: 3,
                },
                expected_events: vec![Event::ItemsRemoved {
                    item_ids: vec![item_id.clone(), item_id_2.clone()],
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
            let actual = aggregate.apply_command(command)?;
            assert_eq!(
                aggregate, expected_aggregate,
                "{name}: aggregate not equaled"
            );
            assert_eq!(actual, expected_events, "{name}: events not equaled");
        }
        Ok(())
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_apply_command_err() -> Result<(), Error> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            command: Command,
            assert: fn(name: &str, actual: CommandKernelError),
        }
        let tests = [
            TestCase {
                name: "未作成集約に作成コマンド以外実行時はAggregateNotCreatedが返る",
                aggregate: Aggregate::default(),
                command: Command::RemoveItems {
                    item_ids: vec![Id::generate()],
                },
                assert: |name, actual| {
                    assert!(
                        matches!(actual, CommandKernelError::AggregateNotCreated),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "作成済集約に作成コマンド実行時はAggregateAlreadyCreatedが返る",
                aggregate: Aggregate {
                    version: 1,
                    ..Default::default()
                },
                command: Command::Create {
                    name: String::new(),
                },
                assert: |name, actual| {
                    assert!(
                        matches!(actual, CommandKernelError::AggregateAlreadyCreated),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "テナント名が空文字の状態で作成コマンド実行時はInvalidTenantNameが返る",
                aggregate: Aggregate::default(),
                command: Command::Create {
                    name: String::new(),
                },
                assert: |name, actual| {
                    assert!(
                        matches!(actual, CommandKernelError::InvalidTenantName),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "追加する商品が空の状態で商品追加コマンド実行時はEmptyItemsが返る",
                aggregate: Aggregate {
                    version: 1,
                    ..Default::default()
                },
                command: Command::AddItems { items: Vec::new() },
                assert: |name, actual| {
                    assert!(matches!(actual, CommandKernelError::EmptyItems), "{name}");
                },
            },
            TestCase {
                name: "追加する商品名が空文字の状態で商品追加コマンド実行時はInvalidItemNameが返る",
                aggregate: Aggregate {
                    version: 1,
                    ..Default::default()
                },
                command: Command::AddItems {
                    items: vec![Item::new(Id::generate(), String::new(), 1000)],
                },
                assert: |name, actual| {
                    assert!(
                        matches!(actual, CommandKernelError::InvalidItemName),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "削除する商品が空の状態で商品削除コマンド実行時はEmptyItemIdsが返る",
                aggregate: Aggregate {
                    version: 1,
                    ..Default::default()
                },
                command: Command::RemoveItems {
                    item_ids: Vec::new(),
                },
                assert: |name, actual| {
                    assert!(matches!(actual, CommandKernelError::EmptyItemIds), "{name}");
                },
            },
            TestCase {
                name: "バージョンが最大値の集約にコマンド実行時はAggregateVersionOverflowedが返る",
                aggregate: Aggregate {
                    version: u64::MAX,
                    ..Default::default()
                },
                command: Command::RemoveItems {
                    item_ids: vec![Id::generate()],
                },
                assert: |name, actual| {
                    assert!(
                        matches!(actual, CommandKernelError::AggregateVersionOverflowed),
                        "{name}"
                    );
                },
            },
        ];
        for TestCase {
            name,
            mut aggregate,
            command,
            assert,
        } in tests
        {
            let result = aggregate.apply_command(command);
            assert(
                name,
                result
                    .err()
                    .ok_or(format!("{name}: apply_command must be error"))?,
            );
        }
        Ok(())
    }
}
