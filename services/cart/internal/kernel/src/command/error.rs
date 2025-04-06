use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandKernelError {
    #[error("Aggregate not created")]
    AggregateNotCreated,
    #[error("Aggregate already created")]
    AggregateAlreadyCreated,
    #[error("Cannot update Aggregate version")]
    AggregateVersionOverflowed,
    #[error("Order already placed")]
    OrderAlreadyPlaced,
    #[error("Tenant not found")]
    TenantNotFound,
    #[error("Item not found")]
    ItemNotFound,
    #[error("PlaceOrder: {message}")]
    PlaceOrder { message: String },
}
