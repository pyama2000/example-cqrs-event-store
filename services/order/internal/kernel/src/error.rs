use thiserror::Error;

#[derive(Error, Debug)]
pub enum KernelError {
    #[error("Aggregate already created")]
    AggregateAlreadyCreated,
    #[error("Aggregate not created")]
    AggregateNotCreated,
    #[error("Cannot update Aggregate version")]
    AggregateVersionOverflow,
    #[error("Invalid delivery address")]
    InvalidDeliveryAddress,
    #[error("Invalid status change")]
    InvalidStatusChange,
    #[error("Invalid order item quantity")]
    InvalidOrderItemQuantity,
    #[error("Order items is empty")]
    OrderItemsIsEmpty,
    #[error("Empty events")]
    EmptyEvents,
    #[error("Invalid event")]
    InvalidEvents,
    #[error("Aggregate not found")]
    AggregateNotFound,
    #[error(transparent)]
    Unknown(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}
