use thiserror::Error;

#[derive(Error, Debug)]
pub enum KernelError {
    #[error("Aggregate already created")]
    AggregateAlreadyCreated,
    #[error("Aggregate not created")]
    AggregateNotCreated,
    #[error("Cannot update Aggregate version")]
    AggregateVersionOverflow,
    #[error("Invalid restaurant name")]
    InvalidRestaurantName,
    #[error("Invalid item name")]
    InvalidItemName,
    #[error("Entities is empty")]
    EntitiesIsEmpty,
    #[error("Empty event")]
    EmptyEvent,
    #[error("Invalid events")]
    InvalidEvents,
    #[error("Aggregate not found")]
    AggregateNotFound,
    #[error(transparent)]
    Unknown(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}
