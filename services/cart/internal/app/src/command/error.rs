use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandUseCaseError {
    #[error("Aggregate not found")]
    AggregateNotFound,
}
