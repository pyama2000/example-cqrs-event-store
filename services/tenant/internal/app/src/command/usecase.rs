use std::future::Future;

use kernel::{CommandProcessor, Id};

use super::{CommandUseCaseError, Item, Tenant};

/// ユースケースのインターフェイス
pub trait CommandUseCaseExt {
    fn create(
        &self,
        tenant: Tenant,
    ) -> impl Future<Output = Result<Id<Tenant>, CommandUseCaseError>> + Send;

    fn add_items(
        &self,
        tenant_id: Id<Tenant>,
        items: Vec<Item>,
    ) -> impl Future<Output = Result<Vec<Id<Item>>, CommandUseCaseError>> + Send;

    fn remove_items(
        &self,
        tenant_id: Id<Tenant>,
        item_ids: Vec<Id<Item>>,
    ) -> impl Future<Output = Result<(), CommandUseCaseError>> + Send;
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
    async fn create(&self, _: Tenant) -> Result<Id<Tenant>, CommandUseCaseError> {
        todo!()
    }

    async fn add_items(
        &self,
        _: Id<Tenant>,
        _: Vec<Item>,
    ) -> Result<Vec<Id<Item>>, CommandUseCaseError> {
        todo!()
    }

    async fn remove_items(
        &self,
        _: Id<Tenant>,
        _: Vec<Id<Item>>,
    ) -> Result<(), CommandUseCaseError> {
        todo!()
    }
}
