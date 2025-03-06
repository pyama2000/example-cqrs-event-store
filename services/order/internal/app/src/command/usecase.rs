use std::future::Future;

use kernel::command::command::Command;
use kernel::command::model::aggregate::Aggregate;
use kernel::command::model::entity::Cart;
use kernel::id::Id;

use super::error::CommandUseCaseError;
use super::model::Item;

/// ユースケースのインターフェイス
pub trait CommandUseCaseExt {
    /// 注文を作成する
    fn create(
        &self,
        cart_id: Id<Cart>,
        items: Vec<Item>,
    ) -> impl Future<Output = Result<Result<Id<Aggregate>, CommandUseCaseError>, anyhow::Error>> + Send;

    /// テナントが商品の準備が完了した
    fn prepared(
        &self,
        id: Id<Aggregate>,
    ) -> impl Future<Output = Result<Result<(), CommandUseCaseError>, anyhow::Error>> + Send;

    /// 配達員が商品を受け取った
    fn picked_up(
        &self,
        id: Id<Aggregate>,
    ) -> impl Future<Output = Result<Result<(), CommandUseCaseError>, anyhow::Error>> + Send;

    /// 商品の受け渡しが完了した
    fn delivered(
        &self,
        id: Id<Aggregate>,
    ) -> impl Future<Output = Result<Result<(), CommandUseCaseError>, anyhow::Error>> + Send;

    /// 注文を何らかの理由でキャンセルする
    fn cancel(
        &self,
        id: Id<Aggregate>,
    ) -> impl Future<Output = Result<Result<(), CommandUseCaseError>, anyhow::Error>> + Send;
}

/// ユースケースの実態
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandUseCase<P: kernel::command::processor::CommandProcessor> {
    processor: P,
}

impl<P: kernel::command::processor::CommandProcessor> CommandUseCase<P> {
    pub fn new(processor: P) -> Self {
        Self { processor }
    }
}

impl<P> CommandUseCaseExt for CommandUseCase<P>
where
    P: kernel::command::processor::CommandProcessor + Send + Sync + 'static,
{
    async fn create(
        &self,
        cart_id: Id<Cart>,
        items: Vec<Item>,
    ) -> Result<Result<Id<Aggregate>, CommandUseCaseError>, anyhow::Error> {
        use anyhow::Context as _;

        let mut aggregate = Aggregate::default();
        let id = aggregate.id().clone();
        let event = aggregate
            .apply_command(Command::Create {
                cart_id,
                items: items.into_iter().map(Into::into).collect(),
            })?
            .pop()
            .with_context(|| "event not present")?;
        self.processor.create(aggregate, event).await??;
        Ok(Ok(id))
    }

    async fn prepared(
        &self,
        id: Id<Aggregate>,
    ) -> Result<Result<(), CommandUseCaseError>, anyhow::Error> {
        let Some(mut aggregate) = self.processor.get(id).await?? else {
            return Ok(Err(CommandUseCaseError::AggregateNotFound));
        };
        let events = aggregate.apply_command(Command::Prepared)?;
        self.processor.update(aggregate, events).await??;
        Ok(Ok(()))
    }

    async fn picked_up(
        &self,
        id: Id<Aggregate>,
    ) -> Result<Result<(), CommandUseCaseError>, anyhow::Error> {
        let Some(mut aggregate) = self.processor.get(id).await?? else {
            return Ok(Err(CommandUseCaseError::AggregateNotFound));
        };
        let events = aggregate.apply_command(Command::PickedUp)?;
        self.processor.update(aggregate, events).await??;
        Ok(Ok(()))
    }

    async fn delivered(
        &self,
        id: Id<Aggregate>,
    ) -> Result<Result<(), CommandUseCaseError>, anyhow::Error> {
        let Some(mut aggregate) = self.processor.get(id).await?? else {
            return Ok(Err(CommandUseCaseError::AggregateNotFound));
        };
        let events = aggregate.apply_command(Command::Delivered)?;
        self.processor.update(aggregate, events).await??;
        Ok(Ok(()))
    }

    async fn cancel(
        &self,
        id: Id<Aggregate>,
    ) -> Result<Result<(), CommandUseCaseError>, anyhow::Error> {
        let Some(mut aggregate) = self.processor.get(id).await?? else {
            return Ok(Err(CommandUseCaseError::AggregateNotFound));
        };
        let events = aggregate.apply_command(Command::Cancel)?;
        self.processor.update(aggregate, events).await??;
        Ok(Ok(()))
    }
}
