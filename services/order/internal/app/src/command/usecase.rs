use std::future::Future;

use kernel::{Aggregate, Id};

use crate::{AppError, Order, OrderItem};

use super::DeliveryPerson;

pub trait CommandService {
    fn receive(
        &self,
        order: Order,
        order_items: Vec<OrderItem>,
    ) -> impl Future<Output = Result<Id<Aggregate>, AppError>> + Send;

    fn preparing(
        &self,
        aggregate_id: Id<Aggregate>,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn assign_delievery_person(
        &self,
        aggregate_id: Id<Aggregate>,
        delivery_person_id: Id<DeliveryPerson>,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn ready_for_pick(
        &self,
        aggregate_id: Id<Aggregate>,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn delivery_person_picking_up(
        &self,
        aggregate_id: Id<Aggregate>,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn delivered(
        &self,
        aggregate_id: Id<Aggregate>,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn cancel(
        &self,
        aggregate_id: Id<Aggregate>,
    ) -> impl Future<Output = Result<(), AppError>> + Send;
}
