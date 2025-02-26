use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandKernelError {
    #[error("Aggregate not created")]
    AggregateNotCreated,
    #[error("Aggregate already created")]
    AggregateAlreadyCreated,
    #[error("Order already placed")]
    OrderAlreadyPlaced,
    #[error("Item not found")]
    ItemNotFound,
    #[error("PlaceOrder: {message}")]
    PlaceOrder { message: String },
}
