#[cfg(feature = "command")]
use crate::{Command, Event, KernelError};
use crate::{Id, Order, OrderItem};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Aggregate {
    id: Id<Aggregate>,
    order: Order,
    order_items: Vec<OrderItem>,
    version: u64,
}

impl Aggregate {
    #[must_use]
    pub fn new(id: Id<Aggregate>, order: Order, order_items: Vec<OrderItem>, version: u64) -> Self {
        Self {
            id,
            order,
            order_items,
            version,
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
    pub fn version(&self) -> u64 {
        self.version
    }

    #[must_use]
    pub fn total_price(&self) -> u64 {
        self.order_items
            .iter()
            .map(|x| x.price() * x.quantity())
            .sum()
    }

    #[must_use]
    #[cfg(feature = "command")]
    pub fn is_conflicted(&self, version: u64) -> bool {
        version != self.version
    }

    /// 集約にコマンドを実行する
    ///
    /// # Errors
    ///
    /// * `AggregateAlreadyCreated`: 作成済みの集約を再作成するときのエラー
    /// * `AggregateNotCreated`: 未作成の集約に集約を作成するコマンド以外を実行するときのエラー
    /// * `AggregateVersionOverflow`: バージョンが最大値の集約にコマンドを実行するときのエラー
    /// * `InvalidDeliveryAddress`: 配達先の住所が不正なときのエラー
    /// * `InvalidOrderItemQuantity`: 注文商品の数量が不正なときのエラー
    /// * `InvalidStatusChange`: 注文ステータスの遷移が不正なときのエラー
    /// * `OrderItemsIsEmpty`: 注文商品が空のときのエラー
    #[cfg(feature = "command")]
    pub fn apply_command(self, command: Command) -> Result<(Aggregate, Vec<Event>), KernelError> {
        use crate::{EventPayload, OrderStatus};

        if matches!(&command, Command::Receive { .. }) && self.version.ne(&0) {
            return Err(KernelError::AggregateAlreadyCreated);
        }
        if !matches!(&command, Command::Receive { .. }) && self.version.eq(&0) {
            return Err(KernelError::AggregateNotCreated);
        }
        if matches!(&command, Command::Receive { order, .. } if order.delivery_address().is_empty())
        {
            return Err(KernelError::InvalidDeliveryAddress);
        }
        if matches!(&command, Command::Receive { items, .. } if items.is_empty()) {
            return Err(KernelError::OrderItemsIsEmpty);
        }
        if matches!(&command, Command::Receive { items, .. } if items.iter().any(|x| x.quantity().eq(&0)))
        {
            return Err(KernelError::InvalidOrderItemQuantity);
        }

        let mut aggregate = self;
        let events: Vec<Event> = command.into();
        for event in &events {
            match event.payload() {
                EventPayload::Received { order, items } => match aggregate.order.status {
                    OrderStatus::Received => {
                        aggregate.order = order.clone();
                        aggregate.order_items.clone_from(items);
                    }
                    _ => return Err(KernelError::InvalidStatusChange),
                },
                EventPayload::PreparationStarted => match aggregate.order.status {
                    OrderStatus::Received => {
                        aggregate.order.status = OrderStatus::Preparing;
                    }
                    _ => return Err(KernelError::InvalidStatusChange),
                },
                EventPayload::DeliveryPersonAssigned { delivery_person_id } => {
                    match aggregate.order.status {
                        OrderStatus::Preparing => {
                            aggregate.order.status = OrderStatus::DeliveryPersonAssigned {
                                delivery_person_id: delivery_person_id.clone(),
                            }
                        }
                        _ => return Err(KernelError::InvalidStatusChange),
                    }
                }

                EventPayload::ReadyForPickup => match aggregate.order.status {
                    OrderStatus::DeliveryPersonAssigned { delivery_person_id } => {
                        aggregate.order.status = OrderStatus::ReadyForPickup { delivery_person_id }
                    }
                    _ => return Err(KernelError::InvalidStatusChange),
                },
                EventPayload::DeliveryPersonPickedUp => match aggregate.order.status {
                    OrderStatus::ReadyForPickup { delivery_person_id } => {
                        aggregate.order.status = OrderStatus::DeliveryPersonPickedUp {
                            delivery_person_id: delivery_person_id.clone(),
                        }
                    }
                    _ => return Err(KernelError::InvalidStatusChange),
                },
                EventPayload::Delivered => match aggregate.order.status {
                    OrderStatus::DeliveryPersonPickedUp { .. } => {
                        aggregate.order.status = OrderStatus::Delivered;
                    }
                    _ => return Err(KernelError::InvalidStatusChange),
                },
                EventPayload::Cancelled => aggregate.order.status = OrderStatus::Cancelled,
            }
        }
        aggregate.version = aggregate
            .version
            .checked_add(1)
            .ok_or(KernelError::AggregateVersionOverflow)?;

        Ok((aggregate, events))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Aggregate, Command, Event, EventPayload, Id, KernelError, Order, OrderItem, OrderStatus,
    };

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
            1,
        );
        assert_eq!(aggregate.total_price(), 6000);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_apply_command_ok() {
        const ADDRESS: &str = "東京都";

        struct TestCase {
            name: &'static str,
            aggregate: Aggregate,
            command: Command,
            expected: (Aggregate, Vec<Event>),
        }

        let aggregate_id = Id::generate();
        let restaurant_id = Id::generate();
        let user_id = Id::generate();
        let item_id = Id::generate();
        let delivery_person_id = Id::generate();
        let tests = [
            TestCase {
                name:
                    "未作成集約に受付コマンド実行時は集約に注文と商品が追加・ステータスが受付になる",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Order::default(),
                    Vec::default(),
                    u64::default(),
                ),
                command: Command::Receive {
                    order: Order::new(
                        restaurant_id.clone(),
                        user_id.clone(),
                        ADDRESS.to_string(),
                        OrderStatus::default(),
                    ),
                    items: vec![OrderItem::new(item_id.clone(), 1000, 5)],
                },
                expected: (
                    Aggregate::new(
                        aggregate_id.clone(),
                        Order::new(
                            restaurant_id.clone(),
                            user_id.clone(),
                            ADDRESS.to_string(),
                            OrderStatus::Received,
                        ),
                        vec![OrderItem::new(item_id.clone(), 1000, 5)],
                        1,
                    ),
                    vec![Event::new(
                        Id::generate(),
                        EventPayload::Received {
                            order: Order::new(
                                restaurant_id.clone(),
                                user_id.clone(),
                                ADDRESS.to_string(),
                                OrderStatus::Received,
                            ),
                            items: vec![OrderItem::new(item_id.clone(), 1000, 5)],
                        },
                    )],
                ),
            },
            TestCase {
                name: "作成済集約に準備中コマンド実行時は集約のステータスが準備中になる",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Order::new(
                        restaurant_id.clone(),
                        user_id.clone(),
                        ADDRESS.to_string(),
                        OrderStatus::Received,
                    ),
                    vec![OrderItem::new(item_id.clone(), 1000, 5)],
                    1,
                ),
                command: Command::Prepare,
                expected: (
                    Aggregate::new(
                        aggregate_id.clone(),
                        Order::new(
                            restaurant_id.clone(),
                            user_id.clone(),
                            ADDRESS.to_string(),
                            OrderStatus::Preparing,
                        ),
                        vec![OrderItem::new(item_id.clone(), 1000, 5)],
                        2,
                    ),
                    vec![Event::new(Id::generate(), EventPayload::PreparationStarted)],
                ),
            },
            TestCase {
                name: "作成済集約に配達員割当コマンド実行時は集約のステータスが配達員割当になる",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Order::new(
                        restaurant_id.clone(),
                        user_id.clone(),
                        ADDRESS.to_string(),
                        OrderStatus::Preparing,
                    ),
                    vec![OrderItem::new(item_id.clone(), 1000, 5)],
                    2,
                ),
                command: Command::AssigningDeliveryPerson {
                    delivery_person_id: delivery_person_id.clone(),
                },
                expected: (
                    Aggregate::new(
                        aggregate_id.clone(),
                        Order::new(
                            restaurant_id.clone(),
                            user_id.clone(),
                            ADDRESS.to_string(),
                            OrderStatus::DeliveryPersonAssigned {
                                delivery_person_id: delivery_person_id.clone(),
                            },
                        ),
                        vec![OrderItem::new(item_id.clone(), 1000, 5)],
                        3,
                    ),
                    vec![Event::new(
                        Id::generate(),
                        EventPayload::DeliveryPersonAssigned {
                            delivery_person_id: delivery_person_id.clone(),
                        },
                    )],
                ),
            },
            TestCase {
                name: "作成済集約に準備完了コマンド実行時は集約のステータスが準備完了になる",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Order::new(
                        restaurant_id.clone(),
                        user_id.clone(),
                        ADDRESS.to_string(),
                        OrderStatus::DeliveryPersonAssigned {
                            delivery_person_id: delivery_person_id.clone(),
                        },
                    ),
                    vec![OrderItem::new(item_id.clone(), 1000, 5)],
                    3,
                ),
                command: Command::ReadyForPickup,
                expected: (
                    Aggregate::new(
                        aggregate_id.clone(),
                        Order::new(
                            restaurant_id.clone(),
                            user_id.clone(),
                            ADDRESS.to_string(),
                            OrderStatus::ReadyForPickup {
                                delivery_person_id: delivery_person_id.clone(),
                            },
                        ),
                        vec![OrderItem::new(item_id.clone(), 1000, 5)],
                        4,
                    ),
                    vec![Event::new(Id::generate(), EventPayload::ReadyForPickup)],
                ),
            },
            TestCase {
                name: "作成済集約に配達員受取コマンド実行時は集約のステータスが配達員受取になる",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Order::new(
                        restaurant_id.clone(),
                        user_id.clone(),
                        ADDRESS.to_string(),
                        OrderStatus::ReadyForPickup {
                            delivery_person_id: delivery_person_id.clone(),
                        },
                    ),
                    vec![OrderItem::new(item_id.clone(), 1000, 5)],
                    4,
                ),
                command: Command::DeliveryPersonPickingUp,
                expected: (
                    Aggregate::new(
                        aggregate_id.clone(),
                        Order::new(
                            restaurant_id.clone(),
                            user_id.clone(),
                            ADDRESS.to_string(),
                            OrderStatus::DeliveryPersonPickedUp {
                                delivery_person_id: delivery_person_id.clone(),
                            },
                        ),
                        vec![OrderItem::new(item_id.clone(), 1000, 5)],
                        5,
                    ),
                    vec![Event::new(
                        Id::generate(),
                        EventPayload::DeliveryPersonPickedUp,
                    )],
                ),
            },
            TestCase {
                name: "作成済集約に配達コマンド実行時は集約のステータスが配達になる",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Order::new(
                        restaurant_id.clone(),
                        user_id.clone(),
                        ADDRESS.to_string(),
                        OrderStatus::DeliveryPersonPickedUp {
                            delivery_person_id: delivery_person_id.clone(),
                        },
                    ),
                    vec![OrderItem::new(item_id.clone(), 1000, 5)],
                    5,
                ),
                command: Command::Delivered,
                expected: (
                    Aggregate::new(
                        aggregate_id.clone(),
                        Order::new(
                            restaurant_id.clone(),
                            user_id.clone(),
                            ADDRESS.to_string(),
                            OrderStatus::Delivered,
                        ),
                        vec![OrderItem::new(item_id.clone(), 1000, 5)],
                        6,
                    ),
                    vec![Event::new(Id::generate(), EventPayload::Delivered)],
                ),
            },
            TestCase {
                name: "作成済集約にキャンセルコマンド実行時は集約のステータスがキャンセルになる",
                aggregate: Aggregate::new(
                    aggregate_id.clone(),
                    Order::new(
                        restaurant_id.clone(),
                        user_id.clone(),
                        ADDRESS.to_string(),
                        OrderStatus::DeliveryPersonAssigned {
                            delivery_person_id: delivery_person_id.clone(),
                        },
                    ),
                    vec![OrderItem::new(item_id.clone(), 1000, 5)],
                    3,
                ),
                command: Command::Cancel,
                expected: (
                    Aggregate::new(
                        aggregate_id.clone(),
                        Order::new(
                            restaurant_id.clone(),
                            user_id.clone(),
                            ADDRESS.to_string(),
                            OrderStatus::Cancelled,
                        ),
                        vec![OrderItem::new(item_id.clone(), 1000, 5)],
                        4,
                    ),
                    vec![Event::new(Id::generate(), EventPayload::Cancelled)],
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
                aggregate: Aggregate::new(
                    Id::generate(),
                    Order::new(
                        Id::generate(),
                        Id::generate(),
                        String::new(),
                        OrderStatus::default(),
                    ),
                    vec![],
                    1,
                ),
                command: Command::Receive {
                    order: Order::new(
                        Id::generate(),
                        Id::generate(),
                        String::new(),
                        OrderStatus::default(),
                    ),
                    items: vec![],
                },
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::AggregateAlreadyCreated)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "未作成集約に集約作成コマンド以外を実行時はAggregateNotCreatedエラーが返る",
                aggregate: Aggregate::default(),
                command: Command::Prepare,
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::AggregateNotCreated)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name:
                    "バージョンが最大値の集約にコマンド実行時にAggregateVersionOverflowエラーが返る",
                aggregate: Aggregate::new(
                    Id::generate(),
                    Order::new(
                        Id::generate(),
                        Id::generate(),
                        String::new(),
                        OrderStatus::default(),
                    ),
                    vec![],
                    u64::MAX,
                ),
                command: Command::Prepare,
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::AggregateVersionOverflow)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name:
                    "Receiveコマンドで配達先住所が空文字の場合はInvalidDeliveryAddressエラーが返る",
                aggregate: Aggregate::default(),
                command: Command::Receive {
                    order: Order::new(
                        Id::generate(),
                        Id::generate(),
                        String::new(),
                        OrderStatus::default(),
                    ),
                    items: vec![],
                },
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::InvalidDeliveryAddress)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "Receiveコマンドで注文商品が空の場合はOrderItemsIsEmptyエラーが返る",
                aggregate: Aggregate::default(),
                command: Command::Receive {
                    order: Order::new(
                        Id::generate(),
                        Id::generate(),
                        "東京都".to_string(),
                        OrderStatus::default(),
                    ),
                    items: vec![],
                },
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::OrderItemsIsEmpty)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name:
                    "Receiveコマンドで注文商品の数量が0の場合はInvalidOrderItemQuantityエラーが返る",
                aggregate: Aggregate::default(),
                command: Command::Receive {
                    order: Order::new(
                        Id::generate(),
                        Id::generate(),
                        "東京都".to_string(),
                        OrderStatus::default(),
                    ),
                    items: vec![
                        OrderItem::new(Id::generate(), 1000, 5),
                        OrderItem::new(Id::generate(), 1000, 0),
                    ],
                },
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::InvalidOrderItemQuantity)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name:
                    "注文状態がRecived以外でReceiveコマンドの場合はInvalidStatusChangeエラーが返る",
                aggregate: Aggregate::new(
                    Id::generate(),
                    Order::new(
                        Id::generate(),
                        Id::generate(),
                        String::new(),
                        OrderStatus::Preparing,
                    ),
                    vec![],
                    0,
                ),
                command: Command::Receive {
                    order: Order::new(
                        Id::generate(),
                        Id::generate(),
                        "東京都".to_string(),
                        OrderStatus::default(),
                    ),
                    items: vec![OrderItem::new(Id::generate(), 1000, 5)],
                },
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::InvalidStatusChange)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name:
                    "注文状態がReceived以外でPrepareコマンドの場合はInvalidStatusChangeエラーが返る",
                aggregate: Aggregate::new(
                    Id::generate(),
                    Order::new(
                        Id::generate(),
                        Id::generate(),
                        String::new(),
                        OrderStatus::Preparing,
                    ),
                    vec![],
                    1,
                ),
                command: Command::Prepare,
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::InvalidStatusChange)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name:
                    "status: !Preparing, command: AssigningDeliveryPerson, err: InvalidStatusChange",
                aggregate: Aggregate::new(
                    Id::generate(),
                    Order::new(
                        Id::generate(),
                        Id::generate(),
                        String::new(),
                        OrderStatus::Received,
                    ),
                    vec![],
                    1,
                ),
                command: Command::AssigningDeliveryPerson {
                    delivery_person_id: Id::generate(),
                },
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::InvalidStatusChange)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name:
                    "status:!AssigningDeliveryPerson, cmd: ReadyForPickup, err: InvalidStatusChange",
                aggregate: Aggregate::new(
                    Id::generate(),
                    Order::new(
                        Id::generate(),
                        Id::generate(),
                        String::new(),
                        OrderStatus::Received,
                    ),
                    vec![],
                    1,
                ),
                command: Command::ReadyForPickup,
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::InvalidStatusChange)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name:
                    "status:!ReadyForPickup, cmd: DeliveryPersonPickingUp, err: InvalidStatusChange",
                aggregate: Aggregate::new(
                    Id::generate(),
                    Order::new(
                        Id::generate(),
                        Id::generate(),
                        String::new(),
                        OrderStatus::Received,
                    ),
                    vec![],
                    1,
                ),
                command: Command::DeliveryPersonPickingUp,
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::InvalidStatusChange)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "status: !DeliveryPersonPickingUp, cmd: delivered, err: InvalidStatusChange",
                aggregate: Aggregate::new(
                    Id::generate(),
                    Order::new(
                        Id::generate(),
                        Id::generate(),
                        String::new(),
                        OrderStatus::Received,
                    ),
                    vec![],
                    1,
                ),
                command: Command::Delivered,
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(KernelError::InvalidStatusChange)),
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
