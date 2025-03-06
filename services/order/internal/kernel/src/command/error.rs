use thiserror::Error;

use super::event::Event;
use super::model::entity::OrderStatus;

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommandKernelError {
    #[error("Aggregate not created")]
    AggregateNotCreated,
    #[error("Aggregate already created")]
    AggregateAlreadyCreated,
    #[error("Cannot update Aggregate version")]
    AggregateVersionOverflowed,
    #[error("Items is empty")]
    ItemsIsEmpty,
    #[error("Invalid operation: current_status =  {current_status:?}")]
    InvalidOperation { current_status: OrderStatus },
    #[error("Invalid events: {events:?}")]
    InvalidEvents { events: Vec<Event> },
}
