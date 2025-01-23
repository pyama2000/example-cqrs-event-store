use std::future::Future;

use kernel::{Aggregate, Id, QueryProcessor};
use tracing::instrument;

use super::{Item, Tenant};

type Result<T> = core::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// ユースケースのインターフェイス
pub trait QueryUseCaseExt {
    /// テナントの一覧を取得する
    fn list_tenants(&self) -> impl Future<Output = Result<Vec<Tenant>>> + Send;

    /// テナント商品の一覧を取得する
    fn list_items(
        &self,
        tenant_id: Id<Aggregate>,
    ) -> impl Future<Output = Result<Option<Vec<Item>>>> + Send;
}

/// ユースケースの実態
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QueryUseCase<P: QueryProcessor> {
    processor: P,
}

impl<P: QueryProcessor> QueryUseCase<P> {
    #[must_use]
    pub fn new(processor: P) -> Self {
        Self { processor }
    }
}

impl<P> QueryUseCaseExt for QueryUseCase<P>
where
    P: QueryProcessor + Send + Sync + 'static,
{
    #[instrument(skip(self), err, ret)]
    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        Ok(self
            .processor
            .list_tenants()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    #[instrument(skip(self), err, ret)]
    async fn list_items(&self, tenant_id: Id<Aggregate>) -> Result<Option<Vec<Item>>> {
        Ok(self
            .processor
            .list_items(tenant_id)
            .await?
            .map(|items| items.into_iter().map(Item::from).collect()))
    }
}
