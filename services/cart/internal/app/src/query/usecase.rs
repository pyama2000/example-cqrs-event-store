use std::future::Future;

use kernel::id::Id;
use kernel::query::processor::QueryProcessor;

use super::error::QueryUseCaseError;

/// ユースケースのインターフェイス
pub trait QueryUseCaseExt {
    /// カートを取得する
    fn get(
        &self,
        id: Id<kernel::query::model::Cart>,
    ) -> impl Future<
        Output = Result<
            Result<Option<kernel::query::model::Cart>, QueryUseCaseError>,
            anyhow::Error,
        >,
    > + Send;
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
    #[tracing::instrument(skip(self), err, ret)]
    async fn get(
        &self,
        id: Id<kernel::query::model::Cart>,
    ) -> Result<Result<Option<kernel::query::model::Cart>, QueryUseCaseError>, anyhow::Error> {
        Ok(Ok(self.processor.get(id).await??))
    }
}
