use std::future::Future;

use kernel::id::Id;
use kernel::query::model::{Cart, Order, Tenant};
use kernel::query::processor::QueryProcessor;

use super::error::QueryUseCaseError;

/// ユースケースのインターフェイス
pub trait QueryUseCaseExt {
    /// 注文を取得する
    fn get_by_order_id(
        &self,
        id: Id<Order>,
    ) -> impl Future<Output = Result<Result<Option<Order>, QueryUseCaseError>, anyhow::Error>> + Send;

    /// 注文を取得する
    fn get_by_cart_id(
        &self,
        id: Id<Cart>,
    ) -> impl Future<Output = Result<Result<Option<Order>, QueryUseCaseError>, anyhow::Error>> + Send;

    /// テナントに入った注文のID一覧を取得する
    fn list_tenant_received_order_ids(
        &self,
        tenant_id: Id<Tenant>,
    ) -> impl Future<Output = Result<Result<Vec<Id<Order>>, QueryUseCaseError>, anyhow::Error>> + Send;

    /// 準備完了状態になった注文のID一覧を取得する
    fn list_prepared_order_ids(
        &self,
    ) -> impl Future<Output = Result<Result<Vec<Id<Order>>, QueryUseCaseError>, anyhow::Error>> + Send;
}

/// ユースケースの実態
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QueryUseCase<P: QueryProcessor> {
    processor: P,
}

impl<P: QueryProcessor> QueryUseCase<P> {
    pub fn new(processor: P) -> Self {
        Self { processor }
    }
}

impl<P> QueryUseCaseExt for QueryUseCase<P>
where
    P: QueryProcessor + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self), err(Debug), ret)]
    async fn get_by_order_id(
        &self,
        id: Id<Order>,
    ) -> Result<Result<Option<Order>, QueryUseCaseError>, anyhow::Error> {
        Ok(Ok(self.processor.get_by_order_id(id).await??))
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
    async fn get_by_cart_id(
        &self,
        id: Id<Cart>,
    ) -> Result<Result<Option<Order>, QueryUseCaseError>, anyhow::Error> {
        Ok(Ok(self.processor.get_by_cart_id(id).await??))
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
    async fn list_tenant_received_order_ids(
        &self,
        tenant_id: Id<Tenant>,
    ) -> Result<Result<Vec<Id<Order>>, QueryUseCaseError>, anyhow::Error> {
        Ok(Ok(self
            .processor
            .list_tenant_received_order_ids(tenant_id)
            .await??))
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
    async fn list_prepared_order_ids(
        &self,
    ) -> Result<Result<Vec<Id<Order>>, QueryUseCaseError>, anyhow::Error> {
        Ok(Ok(self.processor.list_prepared_order_ids().await??))
    }
}
