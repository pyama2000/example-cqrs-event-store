use std::future::Future;

use kernel::{Aggregate, Command, CommandProcessor, Id};

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
    async fn create(&self, tenant: Tenant) -> Result<Id<Aggregate>> {
        let mut aggregate = Aggregate::default();
        let aggregate_id = aggregate.id().clone();
        let events = aggregate.apply_command(Command::Create { name: tenant.name })?;
        self.processor
            .create(aggregate, events)
            .await
            .map_err(|e| match e {
                kernel::CommandKernelError::InvalidTenantName => {
                    CommandUseCaseError::InvalidArgument
                }
                kernel::CommandKernelError::Unknown(e) => CommandUseCaseError::Unknown(e),
                _ => CommandUseCaseError::KernelError(e),
            })?;
        Ok(aggregate_id)
    }

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
                kernel::CommandKernelError::AggregateVersionOverflowed
                | kernel::CommandKernelError::EventOverflowed => CommandUseCaseError::Overflowed,
                kernel::CommandKernelError::InvalidItemName
                | kernel::CommandKernelError::EmptyItems => CommandUseCaseError::InvalidArgument,
                kernel::CommandKernelError::Unknown(e) => CommandUseCaseError::Unknown(e),
                _ => CommandUseCaseError::KernelError(e),
            })?;
        self.processor.update(aggregate, events).await?;
        Ok(item_ids)
    }

    async fn remove_items(&self, id: Id<Aggregate>, item_ids: Vec<Id<kernel::Item>>) -> Result<()> {
        let mut aggregate = self
            .processor
            .get(id)
            .await?
            .ok_or_else(|| CommandUseCaseError::NotFound)?;
        let events = aggregate
            .apply_command(Command::RemoveItems { item_ids })
            .map_err(|e| match e {
                kernel::CommandKernelError::AggregateVersionOverflowed
                | kernel::CommandKernelError::EventOverflowed => CommandUseCaseError::Overflowed,
                kernel::CommandKernelError::EmptyItemIds => CommandUseCaseError::InvalidArgument,
                kernel::CommandKernelError::Unknown(e) => CommandUseCaseError::Unknown(e),
                _ => CommandUseCaseError::KernelError(e),
            })?;
        self.processor.update(aggregate, events).await?;
        Ok(())
    }
}
