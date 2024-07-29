use std::future::Future;

use kernel::{Aggregate, Command, CommandProcessor, Id};

use crate::{AppError, Order, OrderItem};

use super::DeliveryPerson;

pub trait CommandService {
    fn create_order(
        &self,
        order: Order,
        order_items: Vec<OrderItem>,
    ) -> impl Future<Output = Result<Id<Aggregate>, AppError>> + Send;

    fn preparing(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn assign_delievery_person(
        &self,
        aggregate_id: Id<Aggregate>,
        delivery_person_id: Id<DeliveryPerson>,
        current_aggregate_version: u64,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn ready_for_pick(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn delivery_person_picking_up(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn delivered(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn cancel(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
    ) -> impl Future<Output = Result<(), AppError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandUseCase<P: CommandProcessor> {
    processor: P,
}

impl<P: CommandProcessor> CommandUseCase<P> {
    pub fn new(processor: P) -> Self {
        Self { processor }
    }
}

impl<P> CommandService for CommandUseCase<P>
where
    P: CommandProcessor + Send + Sync + 'static,
{
    async fn create_order(
        &self,
        order: Order,
        order_items: Vec<OrderItem>,
    ) -> Result<Id<Aggregate>, AppError> {
        let (aggregate, events) = Aggregate::default().apply_command(Command::Receive {
            order: kernel::Order::new(
                order.restaurant_id.to_string().parse()?,
                order.user_id.to_string().parse()?,
                order.delivery_address,
                kernel::OrderStatus::Received,
            ),
            items: order_items
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })?;
        let aggregate_id = aggregate.id().clone();
        self.processor.create(aggregate, events).await?;
        Ok(aggregate_id)
    }

    async fn preparing(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
    ) -> Result<(), AppError> {
        let aggregate = self.processor.get(aggregate_id).await?;
        if aggregate.is_conflicted(current_aggregate_version) {
            return Err(AppError::AggregateConflicted);
        }
        let (aggregate, events) = aggregate.apply_command(Command::Prepare)?;
        Ok(self.processor.update(aggregate, events).await?)
    }

    async fn assign_delievery_person(
        &self,
        aggregate_id: Id<Aggregate>,
        delivery_person_id: Id<DeliveryPerson>,
        current_aggregate_version: u64,
    ) -> Result<(), AppError> {
        let aggregate = self.processor.get(aggregate_id).await?;
        if aggregate.is_conflicted(current_aggregate_version) {
            return Err(AppError::AggregateConflicted);
        }
        let (aggregate, events) = aggregate.apply_command(Command::AssigningDeliveryPerson {
            delivery_person_id: delivery_person_id.to_string().parse()?,
        })?;
        Ok(self.processor.update(aggregate, events).await?)
    }

    async fn ready_for_pick(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
    ) -> Result<(), AppError> {
        let aggregate = self.processor.get(aggregate_id).await?;
        if aggregate.is_conflicted(current_aggregate_version) {
            return Err(AppError::AggregateConflicted);
        }
        let (aggregate, events) = aggregate.apply_command(Command::ReadyForPickup)?;
        Ok(self.processor.update(aggregate, events).await?)
    }

    async fn delivery_person_picking_up(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
    ) -> Result<(), AppError> {
        let aggregate = self.processor.get(aggregate_id).await?;
        if aggregate.is_conflicted(current_aggregate_version) {
            return Err(AppError::AggregateConflicted);
        }
        let (aggregate, events) = aggregate.apply_command(Command::DeliveryPersonPickingUp)?;
        Ok(self.processor.update(aggregate, events).await?)
    }

    async fn delivered(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
    ) -> Result<(), AppError> {
        let aggregate = self.processor.get(aggregate_id).await?;
        if aggregate.is_conflicted(current_aggregate_version) {
            return Err(AppError::AggregateConflicted);
        }
        let (aggregate, events) = aggregate.apply_command(Command::Delivered)?;
        Ok(self.processor.update(aggregate, events).await?)
    }

    async fn cancel(
        &self,
        aggregate_id: Id<Aggregate>,
        current_aggregate_version: u64,
    ) -> Result<(), AppError> {
        let aggregate = self.processor.get(aggregate_id).await?;
        if aggregate.is_conflicted(current_aggregate_version) {
            return Err(AppError::AggregateConflicted);
        }
        let (aggregate, events) = aggregate.apply_command(Command::Cancel)?;
        Ok(self.processor.update(aggregate, events).await?)
    }
}
