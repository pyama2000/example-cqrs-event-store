use std::future::Future;

use kernel::{CommandProcessor, Id};

use super::{CommandUseCaseError, Item, Tenant};

type Result<T> = core::result::Result<T, CommandUseCaseError>;

/// ユースケースのインターフェイス
pub trait CommandUseCaseExt {
    /// テナントを作成する
    fn create(&self, tenant: Tenant) -> impl Future<Output = Result<Id<Tenant>>> + Send;

    /// テナントに商品を追加する
    fn add_items(
        &self,
        tenant_id: Id<Tenant>,
        items: Vec<Item>,
    ) -> impl Future<Output = Result<Vec<Id<Item>>>> + Send;

    /// テナントから商品を削除する
    fn remove_items(
        &self,
        tenant_id: Id<Tenant>,
        item_ids: Vec<Id<Item>>,
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
    async fn create(&self, _: Tenant) -> Result<Id<Tenant>> {
        todo!()
    }

    async fn add_items(&self, _: Id<Tenant>, _: Vec<Item>) -> Result<Vec<Id<Item>>> {
        todo!()
    }

    async fn remove_items(&self, _: Id<Tenant>, _: Vec<Id<Item>>) -> Result<()> {
        todo!()
    }
}
