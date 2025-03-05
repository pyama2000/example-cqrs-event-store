use std::future::Future;

use kernel::command::command::Command;
use kernel::command::model::aggregate::Aggregate;
use kernel::command::model::entity::{Item, Tenant};
use kernel::command::processor::CommandProcessor;
use kernel::id::Id;

use super::error::CommandUseCaseError;

/// ユースケースのインターフェイス
pub trait CommandUseCaseExt {
    /// カートを作成する
    fn create(
        &self,
    ) -> impl Future<Output = Result<Result<Id<Aggregate>, CommandUseCaseError>, anyhow::Error>> + Send;

    /// カートに商品を追加する
    fn add_item(
        &self,
        id: Id<Aggregate>,
        tenant_id: Id<Tenant>,
        item_id: Id<Item>,
    ) -> impl Future<Output = Result<Result<(), CommandUseCaseError>, anyhow::Error>> + Send;

    /// カートの商品を削除する
    fn remove_item(
        &self,
        id: Id<Aggregate>,
        tenant_id: Id<Tenant>,
        item_id: Id<Item>,
    ) -> impl Future<Output = Result<Result<(), CommandUseCaseError>, anyhow::Error>> + Send;

    /// カートの商品を注文する
    fn place_order(
        &self,
        id: Id<Aggregate>,
    ) -> impl Future<Output = Result<Result<(), CommandUseCaseError>, anyhow::Error>> + Send;
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
    async fn create(&self) -> Result<Result<Id<Aggregate>, CommandUseCaseError>, anyhow::Error> {
        use anyhow::Context as _;

        let mut aggregate = Aggregate::default();
        let id = aggregate.id().clone();
        let event = aggregate
            .apply_command(Command::Create)?
            .pop()
            .with_context(|| "event not present")?;
        self.processor.create(aggregate, event).await??;
        Ok(Ok(id))
    }

    async fn add_item(
        &self,
        id: Id<Aggregate>,
        tenant_id: Id<Tenant>,
        item_id: Id<Item>,
    ) -> Result<Result<(), CommandUseCaseError>, anyhow::Error> {
        let Some(mut aggregate): Option<Aggregate> = self.processor.get(id).await?? else {
            return Ok(Err(CommandUseCaseError::AggregateNotFound));
        };
        let events = aggregate.apply_command(Command::AddItem { tenant_id, item_id })?;
        self.processor.update(aggregate, events).await??;
        Ok(Ok(()))
    }

    async fn remove_item(
        &self,
        id: Id<Aggregate>,
        tenant_id: Id<Tenant>,
        item_id: Id<Item>,
    ) -> Result<Result<(), CommandUseCaseError>, anyhow::Error> {
        let Some(mut aggregate): Option<Aggregate> = self.processor.get(id).await?? else {
            return Ok(Err(CommandUseCaseError::AggregateNotFound));
        };
        let events = match aggregate.apply_command(Command::RemoveItem { tenant_id, item_id }) {
            Ok(events) => events,
            Err(e) => match e {
                kernel::command::error::CommandKernelError::TenantNotFound
                | kernel::command::error::CommandKernelError::ItemNotFound => return Ok(Ok(())),
                _ => return Err(e.into()),
            },
        };
        self.processor.update(aggregate, events).await??;
        Ok(Ok(()))
    }

    async fn place_order(
        &self,
        id: Id<Aggregate>,
    ) -> Result<Result<(), CommandUseCaseError>, anyhow::Error> {
        let Some(mut aggregate): Option<Aggregate> = self.processor.get(id).await?? else {
            return Ok(Err(CommandUseCaseError::AggregateNotFound));
        };
        let events = aggregate.apply_command(Command::PlaceOrder)?;
        self.processor.update(aggregate, events).await??;
        Ok(Ok(()))
    }
}
