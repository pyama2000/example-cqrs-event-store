use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
}
