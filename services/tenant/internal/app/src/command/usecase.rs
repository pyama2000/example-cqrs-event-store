use std::future::Future;

use kernel::{Aggregate, Command, CommandProcessor, Id};
use tracing::instrument;

use super::{CommandUseCaseError, Item, Tenant};

type Result<T> = core::result::Result<T, CommandUseCaseError>;

/// ユースケースのインターフェイス
pub trait CommandUseCaseExt {
    /// テナントを作成する
    fn create(&self, tenant: Tenant) -> impl Future<Output = Result<Id<Aggregate>>> + Send;

    /// テナントに商品を追加する
    fn add_items(
        &self,
        tenant_id: Id<Aggregate>,
        items: Vec<Item>,
    ) -> impl Future<Output = Result<Vec<Id<kernel::Item>>>> + Send;

    /// テナントから商品を削除する
    fn remove_items(
        &self,
        tenant_id: Id<Aggregate>,
        item_ids: Vec<Id<kernel::Item>>,
    ) -> impl Future<Output = Result<()>> + Send;
}

/// ユースケースの実態
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandUseCase<P: CommandProcessor> {
    processor: P,
}

impl<P: CommandProcessor> CommandUseCase<P> {
    pub fn new(processor: P) -> Self {
        Self { processor }
    }
}

impl<P> CommandUseCaseExt for CommandUseCase<P>
where
    P: CommandProcessor + Send + Sync + 'static,
{
    #[instrument(skip(self), err, ret)]
    async fn create(&self, tenant: Tenant) -> Result<Id<Aggregate>> {
        let mut aggregate = Aggregate::default();
        let aggregate_id = aggregate.id().clone();
        let events = aggregate
            .apply_command(Command::Create { name: tenant.name })
            .map_err(|e| match e {
                kernel::CommandKernelError::InvalidTenantName => {
                    CommandUseCaseError::InvalidArgument
                }
                e => CommandUseCaseError::KernelError(e),
            })?;
        self.processor
            .create(
                aggregate,
                events
                    .first()
                    .ok_or_else(|| CommandUseCaseError::Unknown("event not returned".into()))?
                    .clone(),
            )
            .await?;
        Ok(aggregate_id)
    }

    #[instrument(skip(self), err, ret)]
    async fn add_items(
        &self,
        id: Id<Aggregate>,
        items: Vec<Item>,
    ) -> Result<Vec<Id<kernel::Item>>> {
        let items: Vec<kernel::Item> = items.into_iter().map(Into::into).collect();
        let item_ids: Vec<_> = items.iter().map(|x| x.id().clone()).collect();
        let mut aggregate = self
            .processor
            .get(id)
            .await?
            .ok_or_else(|| CommandUseCaseError::NotFound)?;
        let events = aggregate
            .apply_command(Command::AddItems { items })
            .map_err(|e| match e {
                kernel::CommandKernelError::AggregateVersionOverflowed => {
                    CommandUseCaseError::Overflowed
                }
                kernel::CommandKernelError::InvalidItemName
                | kernel::CommandKernelError::EmptyItems => CommandUseCaseError::InvalidArgument,
                kernel::CommandKernelError::Unknown(e) => CommandUseCaseError::Unknown(e),
                _ => CommandUseCaseError::KernelError(e),
            })?;
        self.processor.update(aggregate, events).await?;
        Ok(item_ids)
    }

    #[instrument(skip(self), err, ret)]
    async fn remove_items(&self, id: Id<Aggregate>, item_ids: Vec<Id<kernel::Item>>) -> Result<()> {
        let mut aggregate = self
            .processor
            .get(id)
            .await?
            .ok_or_else(|| CommandUseCaseError::NotFound)?;
        // テナントの商品にないIDを削除する
        let item_ids: Vec<_> = item_ids
            .into_iter()
            .filter(|id| aggregate.items().iter().any(|item| item.id() == id))
            .collect();
        if item_ids.is_empty() {
            // NOTE: 引数に指定された全ての商品IDがテナントに存在しない場合は集約の更新・イベントの作成をせずに処理を終了する
            return Ok(());
        }
        let events = aggregate
            .apply_command(Command::RemoveItems { item_ids })
            .map_err(|e| match e {
                kernel::CommandKernelError::AggregateVersionOverflowed => {
                    CommandUseCaseError::Overflowed
                }
                kernel::CommandKernelError::EmptyItemIds => CommandUseCaseError::InvalidArgument,
                kernel::CommandKernelError::Unknown(e) => CommandUseCaseError::Unknown(e),
                _ => CommandUseCaseError::KernelError(e),
            })?;
        self.processor.update(aggregate, events).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use kernel::command::processor::MockCommandProcessor;
    use kernel::{Aggregate, CommandProcessor, Event, Id, Item};

    use crate::{CommandUseCase, CommandUseCaseError, CommandUseCaseExt};

    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn test_remove_items_ok() -> Result<(), Error> {
        struct TestCase<P: CommandProcessor> {
            name: &'static str,
            tenant_id: Id<Aggregate>,
            item_ids: Vec<Id<Item>>,
            processor: P,
        }

        let items: Vec<_> = (1..=5)
            .map(|i| Item::new(Id::generate(), String::new(), i * 1000))
            .collect();
        let item_ids: Vec<_> = items.iter().map(|i| i.id().clone()).collect();
        let aggregate_id: Id<Aggregate> = Id::generate();
        let aggregate = Aggregate::new(aggregate_id.clone(), String::new(), items.clone(), 1);
        let tests = [
            {
                const TEST_NAME: &str =
                    "集約が存在し削除する商品IDを全て保持している場合は正常に処理される";
                TestCase {
                    name: TEST_NAME,
                    tenant_id: aggregate_id.clone(),
                    item_ids: item_ids[1..3].to_vec(),
                    processor: {
                        let mut processor = MockCommandProcessor::new();
                        {
                            let aggregate = aggregate.clone();
                            processor.expect_get().returning(move |_| {
                                let aggregate = aggregate.clone();
                                Box::pin(async move { Ok(Some(aggregate)) })
                            });
                        }
                        {
                            let aggregate_id = aggregate_id.clone();
                            let items = items.clone();
                            let item_ids = item_ids.clone();
                            processor
                                .expect_update()
                                .returning(move |aggregate, events| {
                                    let items =
                                        vec![items[0].clone(), items[3].clone(), items[4].clone()];
                                    let item_ids = item_ids.clone();
                                    assert_eq!(
                                        aggregate,
                                        Aggregate::new(
                                            aggregate_id.clone(),
                                            String::new(),
                                            items,
                                            2
                                        ),
                                        "{TEST_NAME}"
                                    );
                                    assert_eq!(
                                        events,
                                        vec![Event::ItemsRemoved {
                                            item_ids: item_ids[1..3].to_vec()
                                        }],
                                        "{TEST_NAME}"
                                    );
                                    Box::pin(async { Ok(()) })
                                });
                        }
                        processor
                    },
                }
            },
            {
                const TEST_NAME: &str =
                    "集約が存在し削除する商品IDを一部保持している場合は正常に処理される";
                TestCase {
                    name: TEST_NAME,
                    tenant_id: aggregate_id.clone(),
                    item_ids: {
                        let mut ids = item_ids[1..3].to_vec();
                        ids.push(Id::generate());
                        ids
                    },
                    processor: {
                        let mut processor = MockCommandProcessor::new();
                        {
                            let aggregate = aggregate.clone();
                            processor.expect_get().returning(move |_| {
                                let aggregate = aggregate.clone();
                                Box::pin(async move { Ok(Some(aggregate)) })
                            });
                        }
                        {
                            let aggregate_id = aggregate_id.clone();
                            let items = items.clone();
                            let item_ids = item_ids.clone();
                            processor
                                .expect_update()
                                .returning(move |aggregate, events| {
                                    let items =
                                        vec![items[0].clone(), items[3].clone(), items[4].clone()];
                                    let item_ids = item_ids.clone();
                                    assert_eq!(
                                        aggregate,
                                        Aggregate::new(
                                            aggregate_id.clone(),
                                            String::new(),
                                            items,
                                            2
                                        ),
                                        "{TEST_NAME}"
                                    );
                                    assert_eq!(
                                        events,
                                        vec![Event::ItemsRemoved {
                                            item_ids: item_ids[1..3].to_vec()
                                        }],
                                        "{TEST_NAME}"
                                    );
                                    Box::pin(async { Ok(()) })
                                });
                        }
                        processor
                    },
                }
            },
            {
                const TEST_NAME: &str =
                    "集約が存在し削除する商品IDを保持していない場合は正常に処理される";
                TestCase {
                    name: TEST_NAME,
                    tenant_id: aggregate_id.clone(),
                    item_ids: (0..10).map(|_| Id::generate()).collect(),
                    processor: {
                        let mut processor = MockCommandProcessor::new();
                        {
                            let aggregate = aggregate.clone();
                            processor.expect_get().returning(move |_| {
                                let aggregate = aggregate.clone();
                                Box::pin(async move { Ok(Some(aggregate)) })
                            });
                        }
                        processor.expect_update().never();
                        processor
                    },
                }
            },
        ];
        for TestCase {
            name,
            tenant_id,
            item_ids,
            processor,
        } in tests
        {
            let usecase: CommandUseCase<MockCommandProcessor> = CommandUseCase::new(processor);
            let result = usecase.remove_items(tenant_id, item_ids).await;
            assert!(result.is_ok(), "{name}: result must be ok: {result:#?}");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_remove_items_err() -> Result<(), Error> {
        struct TestCase<P: CommandProcessor> {
            name: &'static str,
            tenant_id: Id<Aggregate>,
            item_ids: Vec<Id<Item>>,
            processor: P,
            assert: fn(name: &str, actual: CommandUseCaseError),
        }

        let tests = [TestCase {
            name: "集約が存在しない場合はNotFoundが返る",
            tenant_id: Id::generate(),
            item_ids: Vec::new(),
            processor: {
                let mut processor = MockCommandProcessor::new();
                processor
                    .expect_get()
                    .returning(|_| Box::pin(async { Ok(None) }));
                processor
            },
            assert: |name, actual| {
                assert!(matches!(actual, CommandUseCaseError::NotFound), "{name}");
            },
        }];
        for TestCase {
            name,
            tenant_id,
            item_ids,
            processor,
            assert,
        } in tests
        {
            let usecase: CommandUseCase<MockCommandProcessor> = CommandUseCase::new(processor);
            let result = usecase.remove_items(tenant_id, item_ids).await;
            assert!(result.is_err(), "{name}: result must be error: {result:#?}");
            assert(name, result.err().unwrap());
        }
        Ok(())
    }
}
