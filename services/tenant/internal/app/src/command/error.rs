use kernel::CommandKernelError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandUseCaseError {
    #[error("Invalid argument")]
    InvalidArgument,
    #[error("Aggregate not found")]
    NotFound,
    #[error("Cannot update aggregate")]
    Overflowed,
    #[error(transparent)]
    KernelError(#[from] CommandKernelError),
    #[error(transparent)]
    Unknown(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}
