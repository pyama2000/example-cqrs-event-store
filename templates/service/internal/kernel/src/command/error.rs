use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandKernelError {
    #[error(transparent)]
    Unknown(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}
