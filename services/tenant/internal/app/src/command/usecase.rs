use std::future::Future;

use kernel::CommandProcessor;

use super::{CommandUseCaseError, Item, Tenant};

/// ユースケースのインターフェイス
pub trait CommandUseCaseExt {
    fn create(
        &self,
        tenant: Tenant,
    ) -> impl Future<
        Output = Result<
            String, // TODO: kernelでID構造体を定義したら変更する
            CommandUseCaseError,
        >,
    > + Send;

    fn add_items(
        &self,
        tenant_id: String, // TODO: kernelでID構造体を定義したら変更する
        items: Vec<Item>,
    ) -> impl Future<
        Output = Result<
            Vec<
                String, // TODO: kernelでID構造体を定義したら変更する
            >,
            CommandUseCaseError,
        >,
    > + Send;

    fn remove_items(
        &self,
        tenant_id: String,     // TODO: kernelでID構造体を定義したら変更する
        item_ids: Vec<String>, // TODO: kernelでID構造体を定義したら変更する
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
    async fn create(&self, _: Tenant) -> Result<String, CommandUseCaseError> {
        todo!()
    }

    async fn add_items(&self, _: String, _: Vec<Item>) -> Result<Vec<String>, CommandUseCaseError> {
        todo!()
    }

    async fn remove_items(&self, _: String, _: Vec<String>) -> Result<(), CommandUseCaseError> {
        todo!()
    }
}
