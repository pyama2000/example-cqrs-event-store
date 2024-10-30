use kernel::CommandKernelError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandUseCaseError {
    #[error(transparent)]
    KernelError(#[from] CommandKernelError),
    #[error(transparent)]
    Unknown(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}
