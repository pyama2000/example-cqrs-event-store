use std::future::Future;

use kernel::{Aggregate, Command, CommandProcessor, Id};

use crate::{AppError, Item, Restaurant};

pub trait CommandService {
    fn create_restaurant(
        &self,
        restaurant: Restaurant,
    ) -> impl Future<Output = Result<Id<Aggregate>, AppError>> + Send;

    fn add_items(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
        items: Vec<Item>,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn remove_items(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
        item_ids: Vec<Id<kernel::Item>>,
    ) -> impl Future<Output = Result<(), AppError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandUseCase<P: CommandProcessor> {
    processor: P,
}

impl<P: CommandProcessor> CommandUseCase<P> {
    pub fn new(processor: P) -> Self {
        Self { processor }
    }
}

impl<P: CommandProcessor + Send + Sync + 'static> CommandService for CommandUseCase<P> {
    async fn create_restaurant(&self, restaurant: Restaurant) -> Result<Id<Aggregate>, AppError> {
        let (aggregate, events) =
            Aggregate::default().apply_command(Command::CrateAggregate(restaurant.into()))?;
        let aggregate_id = aggregate.id().clone();
        self.processor.create(aggregate, events).await?;
        Ok(aggregate_id)
    }

    async fn add_items(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
        items: Vec<Item>,
    ) -> Result<(), AppError> {
        let aggregate = self.processor.get(aggregate_id).await?;
        if aggregate.is_conflicted(current_aggregate_version) {
            return Err(AppError::AggregateConflicted);
        }
        let (aggregate, events) = aggregate.apply_command(Command::AddItems(
            items.into_iter().map(Into::into).collect(),
        ))?;
        Ok(self.processor.update(aggregate, events).await?)
    }

    async fn remove_items(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
        item_ids: Vec<Id<kernel::Item>>,
    ) -> Result<(), AppError> {
        let aggregate = self.processor.get(aggregate_id).await?;
        if aggregate.is_conflicted(current_aggregate_version) {
            return Err(AppError::AggregateConflicted);
        }
        let (aggregate, events) = aggregate.apply_command(Command::RemoveItems(item_ids))?;
        Ok(self.processor.update(aggregate, events).await?)
    }
}

#[cfg(test)]
mod tests {
    use kernel::processor::MockCommandProcessor;
    use kernel::{Aggregate, CommandProcessor, Id, KernelError};

    use crate::{AppError, CommandService, CommandUseCase, Item, Restaurant};

    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    const RESTAURANT_NAME: &str = "テスト店舗";

    #[tokio::test]
    async fn test_create_restaurant() -> Result<(), Error> {
        struct TestCase<P: CommandProcessor> {
            name: &'static str,
            processor: P,
            restaurant: Restaurant,
            assert: fn(name: &str, actual: Result<Id<Aggregate>, AppError>),
        }

        let tests = [TestCase {
            name: "処理に成功した場合は集約のIdが返ってくる",
            processor: {
                let mut processor = MockCommandProcessor::new();
                processor
                    .expect_create()
                    .returning(|_, _| Box::pin(async { Ok(()) }));
                processor
            },
            restaurant: Restaurant::new(RESTAURANT_NAME.to_string()),
            assert: |name, actual| {
                assert!(actual.is_ok(), "{name}");
            },
        }];

        for TestCase {
            name,
            processor,
            restaurant,
            assert,
        } in tests
        {
            let usecase = CommandUseCase::new(processor);
            let actual = usecase.create_restaurant(restaurant).await;
            assert(name, actual);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_add_items_err() -> Result<(), Error> {
        struct TestCase<P: CommandProcessor> {
            name: &'static str,
            processor: P,
            aggregate_id: Id<Aggregate>,
            current_aggregate_version: u64,
            items: Vec<Item>,
            assert: fn(name: &str, actual: Result<(), AppError>),
        }

        let tests = [
            TestCase {
                name: "現在の集約バージョンと異なる場合、AggregateConflictedエラーが返る",
                processor: {
                    let mut processor = MockCommandProcessor::new();
                    processor.expect_get().returning(|_| {
                        Box::pin(async {
                            Ok(Aggregate::new(
                                Id::generate(),
                                kernel::Restaurant::new(
                                    Id::generate(),
                                    RESTAURANT_NAME.to_string(),
                                ),
                                vec![],
                                2,
                            ))
                        })
                    });
                    processor
                },
                aggregate_id: Id::generate(),
                current_aggregate_version: 1,
                items: vec![],
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(AppError::AggregateConflicted)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "商品の配列が空の場合、エラーが返る",
                processor: {
                    let mut processor = MockCommandProcessor::new();
                    processor.expect_get().returning(|_| {
                        Box::pin(async {
                            Ok(Aggregate::new(
                                Id::generate(),
                                kernel::Restaurant::new(
                                    Id::generate(),
                                    RESTAURANT_NAME.to_string(),
                                ),
                                vec![],
                                1,
                            ))
                        })
                    });
                    processor
                },
                aggregate_id: Id::generate(),
                current_aggregate_version: 1,
                items: vec![],
                assert: |name, actual| {
                    assert!(
                        matches!(
                            actual,
                            Err(AppError::KernelError(e)) if matches!(e, KernelError::EntitiesIsEmpty)
                        ),
                        "{name}"
                    );
                },
            },
        ];

        for TestCase {
            name,
            processor,
            aggregate_id,
            current_aggregate_version,
            items,
            assert,
        } in tests
        {
            let usecase = CommandUseCase::new(processor);
            let actual = usecase
                .add_items(aggregate_id, current_aggregate_version, items)
                .await;
            assert(name, actual);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_remove_items_err() -> Result<(), Error> {
        struct TestCase<P: CommandProcessor> {
            name: &'static str,
            processor: P,
            aggregate_id: Id<Aggregate>,
            current_aggregate_version: u64,
            item_ids: Vec<Id<kernel::Item>>,
            assert: fn(name: &str, actual: Result<(), AppError>),
        }

        let tests = [
            TestCase {
                name: "現在の集約バージョンと異なる場合、AggregateConflictedエラーが返る",
                processor: {
                    let mut processor = MockCommandProcessor::new();
                    processor.expect_get().returning(|_| {
                        Box::pin(async {
                            Ok(Aggregate::new(
                                Id::generate(),
                                kernel::Restaurant::new(
                                    Id::generate(),
                                    RESTAURANT_NAME.to_string(),
                                ),
                                vec![],
                                2,
                            ))
                        })
                    });
                    processor
                },
                aggregate_id: Id::generate(),
                current_aggregate_version: 1,
                item_ids: vec![],
                assert: |name, actual| {
                    assert!(
                        matches!(actual, Err(AppError::AggregateConflicted)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "商品の配列が空の場合、エラーが返る",
                processor: {
                    let mut processor = MockCommandProcessor::new();
                    processor.expect_get().returning(|_| {
                        Box::pin(async {
                            Ok(Aggregate::new(
                                Id::generate(),
                                kernel::Restaurant::new(
                                    Id::generate(),
                                    RESTAURANT_NAME.to_string(),
                                ),
                                vec![],
                                1,
                            ))
                        })
                    });
                    processor
                },
                aggregate_id: Id::generate(),
                current_aggregate_version: 1,
                item_ids: vec![],
                assert: |name, actual| {
                    assert!(
                        matches!(
                            actual,
                            Err(AppError::KernelError(e)) if matches!(e, KernelError::EntitiesIsEmpty)
                        ),
                        "{name}"
                    );
                },
            },
        ];

        for TestCase {
            name,
            processor,
            aggregate_id,
            current_aggregate_version,
            item_ids,
            assert,
        } in tests
        {
            let usecase = CommandUseCase::new(processor);
            let actual = usecase
                .remove_items(aggregate_id, current_aggregate_version, item_ids)
                .await;
            assert(name, actual);
        }

        Ok(())
    }
}
