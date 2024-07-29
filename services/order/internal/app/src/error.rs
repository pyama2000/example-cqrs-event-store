use kernel::KernelError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Aggregate conflicted")]
    AggregateConflicted,
    #[error(transparent)]
    KernelError(#[from] KernelError),
    #[error(transparent)]
    Unknown(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}
