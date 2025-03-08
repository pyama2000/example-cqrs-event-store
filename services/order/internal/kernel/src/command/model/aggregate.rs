use crate::command::command::Command;
use crate::command::error::CommandKernelError;
use crate::command::event::Event;
use crate::id::Id;

use super::entity::{Cart, Item, OrderStatus};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Aggregate {
    id: Id<Aggregate>,
    cart_id: Id<Cart>,
    items: Vec<Item>,
    status: OrderStatus,
    /// 集約のバージョン
    version: u64,
}

impl Aggregate {
    #[must_use]
    pub fn new(
        id: Id<Aggregate>,
        cart_id: Id<Cart>,
        items: Vec<Item>,
        status: OrderStatus,
        version: u64,
    ) -> Self {
        Self {
            id,
            cart_id,
            items,
            status,
            version,
        }
    }

    /// 集約のID
    #[must_use]
    pub fn id(&self) -> &Id<Aggregate> {
        &self.id
    }

    #[must_use]
    pub fn cart_id(&self) -> Id<Cart> {
        self.cart_id
    }

    #[must_use]
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    #[must_use]
    pub fn status(&self) -> OrderStatus {
        self.status
    }

    /// 集約のバージョン
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// 集約にコマンドを実行する
    ///
    /// 集約にコマンドを実行すると、コマンドに応じて集約の状態を変更し、集約の状態を変更したイベントを返す
    ///
    /// # Errors
    ///
    /// コマンド実行時にドメインエラーが発生したら [`CommandKernelError`] を成功状態で返し、例外エラーが発生したら [`anyhow::Error`] を返す
    ///
    /// [`anyhow::Error`]: https://docs.rs/anyhow/latest/anyhow/struct.Error.html
    #[tracing::instrument(skip(self), err(Debug), ret)]
    pub fn apply_command(&mut self, command: Command) -> Result<Vec<Event>, CommandKernelError> {
        if !matches!(command, Command::Create { .. }) && self.version == 0 {
            return Err(CommandKernelError::AggregateNotCreated);
        }
        let events: Vec<Event> = match command {
            Command::Create { cart_id, items } => {
                if self.version != 0 {
                    return Err(CommandKernelError::AggregateAlreadyCreated);
                }
                let items: Vec<Item> = items
                    .into_iter()
                    .filter(|item| item.quantity() != 0)
                    .collect();
                if items.is_empty() {
                    return Err(CommandKernelError::ItemsIsEmpty);
                }
                self.cart_id = cart_id;
                self.items.clone_from(&items);
                vec![Event::Created { cart_id, items }]
            }
            Command::Prepared => {
                if self.status != OrderStatus::Created {
                    return Err(CommandKernelError::InvalidOperation {
                        current_status: self.status,
                    });
                }
                self.status = OrderStatus::Prepared;
                vec![Event::Prepared]
            }
            Command::PickedUp => {
                if self.status != OrderStatus::Prepared {
                    return Err(CommandKernelError::InvalidOperation {
                        current_status: self.status,
                    });
                }
                self.status = OrderStatus::PickedUp;
                vec![Event::PickedUp]
            }
            Command::Delivered => {
                if self.status != OrderStatus::PickedUp {
                    return Err(CommandKernelError::InvalidOperation {
                        current_status: self.status,
                    });
                }
                self.status = OrderStatus::Delivered;
                vec![Event::Delivered]
            }
            Command::Cancel => {
                if self.status == OrderStatus::Delivered || self.status == OrderStatus::Canceled {
                    return Err(CommandKernelError::InvalidOperation {
                        current_status: self.status,
                    });
                }
                self.status = OrderStatus::Canceled;
                vec![Event::Canceled]
            }
        };
        self.version = self
            .version
            .checked_add(1)
            .ok_or(CommandKernelError::AggregateVersionOverflowed)?;
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::command::command::Command;
    use crate::command::error::CommandKernelError;
    use crate::command::event::Event;
    use crate::command::model::aggregate::Aggregate;
    use crate::command::model::entity::{Cart, Item, OrderStatus, Tenant};
    use crate::id::Id;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_apply_command_create_ok() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            command: Command,
            expected_aggregate: Aggregate,
            expected_events: Vec<Event>,
        }

        let tests = [
            {
                let cart_id: Id<Cart> = Id::generate();
                let items = vec![Item::new(Id::generate(), Id::generate(), 1)];
                let aggregate = Aggregate::default();
                TestCase {
                    name: "未作成の集約に1商品をCreate実行時は集約が更新されCreatedが返る",
                    aggregate: aggregate.clone(),
                    command: Command::Create {
                        cart_id,
                        items: items.clone(),
                    },
                    expected_aggregate: Aggregate {
                        cart_id,
                        items: items.clone(),
                        status: OrderStatus::Created,
                        version: 1,
                        ..aggregate
                    },
                    expected_events: vec![Event::Created { cart_id, items }],
                }
            },
            {
                let cart_id: Id<Cart> = Id::generate();
                let items = vec![
                    Item::new(Id::generate(), Id::generate(), 1),
                    Item::new(Id::generate(), Id::generate(), 1),
                ];
                let aggregate = Aggregate::default();
                TestCase {
                    name: "未作成の集約に複数商品をCreate実行時は集約が更新されCreatedが返る",
                    aggregate: aggregate.clone(),
                    command: Command::Create {
                        cart_id,
                        items: items.clone(),
                    },
                    expected_aggregate: Aggregate {
                        cart_id,
                        items: items.clone(),
                        status: OrderStatus::Created,
                        version: 1,
                        ..aggregate
                    },
                    expected_events: vec![Event::Created {
                        cart_id,
                        items: items.clone(),
                    }],
                }
            },
            {
                let cart_id: Id<Cart> = Id::generate();
                let item_id: Id<Item> = Id::generate();
                let tenant_id: Id<Tenant> = Id::generate();
                let items = vec![
                    Item::new(Id::generate(), Id::generate(), 0),
                    Item::new(item_id, tenant_id, 1),
                ];
                let aggregate = Aggregate::default();
                TestCase {
                    name: "未作成の集約に一部商品数0をCreate実行時は集約が更新されCreatedが返る",
                    aggregate: aggregate.clone(),
                    command: Command::Create {
                        cart_id,
                        items: items.clone(),
                    },
                    expected_aggregate: Aggregate {
                        cart_id,
                        items: vec![Item::new(item_id, tenant_id, 1)],
                        status: OrderStatus::Created,
                        version: 1,
                        ..aggregate
                    },
                    expected_events: vec![Event::Created {
                        cart_id,
                        items: vec![Item::new(item_id, tenant_id, 1)],
                    }],
                }
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
            let actual_events = aggregate.apply_command(command)?;
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
    fn test_apply_command_create_err() {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            command: Command,
            expected: CommandKernelError,
        }

        let tests = [
            TestCase {
                name: "作成済みの集約にCreate実行時はAggregateAlreadyCreatedが返る",
                aggregate: Aggregate {
                    version: 1,
                    ..Default::default()
                },
                command: Command::Create {
                    cart_id: Id::generate(),
                    items: vec![Item::new(Id::generate(), Id::generate(), 1)],
                },
                expected: CommandKernelError::AggregateAlreadyCreated,
            },
            TestCase {
                name: "注文商品が空の場合はItemsIsEmptyが返る",
                aggregate: Aggregate::default(),
                command: Command::Create {
                    cart_id: Id::generate(),
                    items: Vec::new(),
                },
                expected: CommandKernelError::ItemsIsEmpty,
            },
            TestCase {
                name: "全ての注文商品数が0の場合はItemsIsEmptyが返る",
                aggregate: Aggregate::default(),
                command: Command::Create {
                    cart_id: Id::generate(),
                    items: vec![
                        Item::new(Id::generate(), Id::generate(), 0),
                        Item::new(Id::generate(), Id::generate(), 0),
                        Item::new(Id::generate(), Id::generate(), 0),
                    ],
                },
                expected: CommandKernelError::ItemsIsEmpty,
            },
        ];

        for TestCase {
            name,
            mut aggregate,
            command,
            expected,
        } in tests
        {
            let actual = aggregate.apply_command(command);
            assert_eq!(actual, Err(expected), "{name}");
        }
    }

    #[test]
    fn test_apply_command_prepared_ok() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            expected_aggregate: Aggregate,
            expected_events: Vec<Event>,
        }

        let tests = [{
            let aggregate = Aggregate {
                id: Id::generate(),
                cart_id: Id::generate(),
                items: vec![Item::new(Id::generate(), Id::generate(), 1)],
                status: OrderStatus::Created,
                version: 1,
            };
            TestCase {
                name: "ステータスがCreatedの集約にPrepared実行時は集約が更新されPreparedが返る",
                aggregate: aggregate.clone(),
                expected_aggregate: Aggregate {
                    status: OrderStatus::Prepared,
                    version: 2,
                    ..aggregate
                },
                expected_events: vec![Event::Prepared],
            }
        }];

        for TestCase {
            name,
            mut aggregate,
            expected_aggregate,
            expected_events,
        } in tests
        {
            let actual_events = aggregate.apply_command(Command::Prepared)?;
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
    fn test_apply_command_prepared_err() {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            expected: CommandKernelError,
        }

        let tests = [
            TestCase {
                name: "未作成の集約にPrepared実行時はAggregateNotCreatedが返る",
                aggregate: Aggregate::default(),
                expected: CommandKernelError::AggregateNotCreated,
            },
            TestCase {
                name: "バージョンが最大値の集約にコマンド実行時はAggregateVersionOverflowedが返る",
                aggregate: Aggregate {
                    version: u64::MAX,
                    ..Default::default()
                },
                expected: CommandKernelError::AggregateVersionOverflowed,
            },
            TestCase {
                name: "Preparedステータスの集約にPrepared実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::Prepared,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::Prepared,
                },
            },
            TestCase {
                name: "PickedUpステータスの集約にPrepared実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::PickedUp,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::PickedUp,
                },
            },
            TestCase {
                name: "Deliveredステータスの集約にPrepared実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::Delivered,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::Delivered,
                },
            },
            TestCase {
                name: "Canceledステータスの集約にPrepared実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::Canceled,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::Canceled,
                },
            },
        ];

        for TestCase {
            name,
            mut aggregate,
            expected,
        } in tests
        {
            let actual = aggregate.apply_command(Command::Prepared);
            assert_eq!(actual, Err(expected), "{name}");
        }
    }

    #[test]
    fn test_apply_command_picked_up_ok() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            expected_aggregate: Aggregate,
            expected_events: Vec<Event>,
        }

        let tests = [{
            let aggregate = Aggregate {
                id: Id::generate(),
                cart_id: Id::generate(),
                items: vec![Item::new(Id::generate(), Id::generate(), 1)],
                status: OrderStatus::Prepared,
                version: 1,
            };
            TestCase {
                name: "ステータスがPreparedの集約にPickedUp実行時は集約が更新されPickedUpが返る",
                aggregate: aggregate.clone(),
                expected_aggregate: Aggregate {
                    status: OrderStatus::PickedUp,
                    version: 2,
                    ..aggregate
                },
                expected_events: vec![Event::PickedUp],
            }
        }];

        for TestCase {
            name,
            mut aggregate,
            expected_aggregate,
            expected_events,
        } in tests
        {
            let actual_events = aggregate.apply_command(Command::PickedUp)?;
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
    fn test_apply_command_picked_up_err() {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            expected: CommandKernelError,
        }

        let tests = [
            TestCase {
                name: "未作成の集約にPickedUp実行時はAggregateNotCreatedが返る",
                aggregate: Aggregate::default(),
                expected: CommandKernelError::AggregateNotCreated,
            },
            TestCase {
                name: "バージョンが最大値の集約にコマンド実行時はAggregateVersionOverflowedが返る",
                aggregate: Aggregate {
                    version: u64::MAX,
                    status: OrderStatus::Prepared,
                    ..Default::default()
                },
                expected: CommandKernelError::AggregateVersionOverflowed,
            },
            TestCase {
                name: "Createdステータスの集約にPickedUp実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::Created,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::Created,
                },
            },
            TestCase {
                name: "PickedUpステータスの集約にPickedUp実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::PickedUp,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::PickedUp,
                },
            },
            TestCase {
                name: "Deliveredステータスの集約にPickedUp実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::Delivered,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::Delivered,
                },
            },
            TestCase {
                name: "Canceledステータスの集約にPickedUp実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::Canceled,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::Canceled,
                },
            },
        ];

        for TestCase {
            name,
            mut aggregate,
            expected,
        } in tests
        {
            let actual = aggregate.apply_command(Command::PickedUp);
            assert_eq!(actual, Err(expected), "{name}");
        }
    }

    #[test]
    fn test_apply_command_delivered_ok() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            expected_aggregate: Aggregate,
            expected_events: Vec<Event>,
        }

        let tests = [{
            let aggregate = Aggregate {
                id: Id::generate(),
                cart_id: Id::generate(),
                items: vec![Item::new(Id::generate(), Id::generate(), 1)],
                status: OrderStatus::PickedUp,
                version: 1,
            };
            TestCase {
                name: "ステータスがPickedUpの集約にDelivered実行時は集約が更新されDeliveredが返る",
                aggregate: aggregate.clone(),
                expected_aggregate: Aggregate {
                    status: OrderStatus::Delivered,
                    version: 2,
                    ..aggregate
                },
                expected_events: vec![Event::Delivered],
            }
        }];

        for TestCase {
            name,
            mut aggregate,
            expected_aggregate,
            expected_events,
        } in tests
        {
            let actual_events = aggregate.apply_command(Command::Delivered)?;
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
    fn test_apply_command_delivered_err() {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            expected: CommandKernelError,
        }

        let tests = [
            TestCase {
                name: "未作成の集約にDelivered実行時はAggregateNotCreatedが返る",
                aggregate: Aggregate::default(),
                expected: CommandKernelError::AggregateNotCreated,
            },
            TestCase {
                name: "バージョンが最大値の集約にコマンド実行時はAggregateVersionOverflowedが返る",
                aggregate: Aggregate {
                    version: u64::MAX,
                    status: OrderStatus::PickedUp,
                    ..Default::default()
                },
                expected: CommandKernelError::AggregateVersionOverflowed,
            },
            TestCase {
                name: "Createdステータスの集約にDelivered実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::Created,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::Created,
                },
            },
            TestCase {
                name: "Preparedステータスの集約にDelivered実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::Prepared,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::Prepared,
                },
            },
            TestCase {
                name: "Deliveredステータスの集約にDelivered実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::Delivered,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::Delivered,
                },
            },
            TestCase {
                name: "Canceledステータスの集約にDelivered実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::Canceled,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::Canceled,
                },
            },
        ];

        for TestCase {
            name,
            mut aggregate,
            expected,
        } in tests
        {
            let actual = aggregate.apply_command(Command::Delivered);
            assert_eq!(actual, Err(expected), "{name}");
        }
    }

    #[test]
    fn test_apply_command_cancel_ok() -> Result<()> {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            expected_aggregate: Aggregate,
            expected_events: Vec<Event>,
        }

        let tests = [
            {
                let aggregate = Aggregate {
                    id: Id::generate(),
                    cart_id: Id::generate(),
                    items: vec![Item::new(Id::generate(), Id::generate(), 1)],
                    status: OrderStatus::Created,
                    version: 1,
                };
                TestCase {
                    name: "ステータスがCreatedの集約にCancel実行時は集約が更新されCanceledが返る",
                    aggregate: aggregate.clone(),
                    expected_aggregate: Aggregate {
                        status: OrderStatus::Canceled,
                        version: 2,
                        ..aggregate
                    },
                    expected_events: vec![Event::Canceled],
                }
            },
            {
                let aggregate = Aggregate {
                    id: Id::generate(),
                    cart_id: Id::generate(),
                    items: vec![Item::new(Id::generate(), Id::generate(), 1)],
                    status: OrderStatus::Prepared,
                    version: 1,
                };
                TestCase {
                    name: "ステータスがPreparedの集約にCancel実行時は集約が更新されCanceledが返る",
                    aggregate: aggregate.clone(),
                    expected_aggregate: Aggregate {
                        status: OrderStatus::Canceled,
                        version: 2,
                        ..aggregate
                    },
                    expected_events: vec![Event::Canceled],
                }
            },
            {
                let aggregate = Aggregate {
                    id: Id::generate(),
                    cart_id: Id::generate(),
                    items: vec![Item::new(Id::generate(), Id::generate(), 1)],
                    status: OrderStatus::PickedUp,
                    version: 1,
                };
                TestCase {
                    name: "ステータスがPickedUpの集約にCancel実行時は集約が更新されCanceledが返る",
                    aggregate: aggregate.clone(),
                    expected_aggregate: Aggregate {
                        status: OrderStatus::Canceled,
                        version: 2,
                        ..aggregate
                    },
                    expected_events: vec![Event::Canceled],
                }
            },
        ];

        for TestCase {
            name,
            mut aggregate,
            expected_aggregate,
            expected_events,
        } in tests
        {
            let actual_events = aggregate.apply_command(Command::Cancel)?;
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
    fn test_apply_command_cancel_err() {
        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            expected: CommandKernelError,
        }

        let tests = [
            TestCase {
                name: "未作成の集約にCancel実行時はAggregateNotCreatedが返る",
                aggregate: Aggregate::default(),
                expected: CommandKernelError::AggregateNotCreated,
            },
            TestCase {
                name: "バージョンが最大値の集約にコマンド実行時はAggregateVersionOverflowedが返る",
                aggregate: Aggregate {
                    version: u64::MAX,
                    status: OrderStatus::Created,
                    ..Default::default()
                },
                expected: CommandKernelError::AggregateVersionOverflowed,
            },
            TestCase {
                name: "Deliveredステータスの集約にCancel実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::Delivered,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::Delivered,
                },
            },
            TestCase {
                name: "Canceledステータスの集約にCancel実行時はInvalidOperationが返る",
                aggregate: Aggregate {
                    status: OrderStatus::Canceled,
                    version: 1,
                    ..Default::default()
                },
                expected: CommandKernelError::InvalidOperation {
                    current_status: OrderStatus::Canceled,
                },
            },
        ];

        for TestCase {
            name,
            mut aggregate,
            expected,
        } in tests
        {
            let actual = aggregate.apply_command(Command::Cancel);
            assert_eq!(actual, Err(expected), "{name}");
        }
    }
}
