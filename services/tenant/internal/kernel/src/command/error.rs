use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandKernelError {
    #[error("Aggregate not created")]
    AggregateNotCreated,
    #[error("Aggregate already created")]
    AggregateAlreadyCreated,
    #[error("Cannot update Aggregate version")]
    AggregateVersionOverflowed,
    #[error("Invalid tenant name")]
    InvalidTenantName,
    #[error("Invalid item name")]
    InvalidItemName,
    #[error("Items are empty")]
    EmptyItems,
    #[error("Item ids are empty")]
    EmptyItemIds,
    #[error(transparent)]
    ProcessorError(#[from] CommandProcessorError),
    #[error(transparent)]
    Unknown(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[derive(Debug, Error)]
pub enum CommandProcessorError {
    #[error("Empty event")]
    EmptyEvent,
    #[error("Invalid event")]
    InvalidEvent,
}
