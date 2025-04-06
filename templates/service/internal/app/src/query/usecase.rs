use kernel::query::processor::QueryProcessor;

/// ユースケースのインターフェイス
pub trait QueryUseCaseExt {}

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

impl<P> QueryUseCaseExt for QueryUseCase<P> where P: QueryProcessor + Send + Sync + 'static {}
