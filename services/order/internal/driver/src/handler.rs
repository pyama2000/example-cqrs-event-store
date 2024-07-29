use std::sync::Arc;

use app::{AppError, CommandService};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::ServiceState;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

fn handling_app_error(err: &AppError) -> StatusCode {
    match err {
        AppError::AggregateConflicted => StatusCode::CONFLICT,
        AppError::KernelError(_) | AppError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ReceiveRequest {
    order: Order,
    order_items: Vec<OrderItem>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Order {
    restaurant_id: String,
    user_id: String,
    delivery_address: String,
}

impl TryFrom<Order> for app::Order {
    type Error = Error;

    fn try_from(
        Order {
            restaurant_id,
            user_id,
            delivery_address,
        }: Order,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(
            restaurant_id.parse()?,
            user_id.parse()?,
            delivery_address,
        ))
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct OrderItem {
    item_id: String,
    price: u64,
    quantity: u64,
}

impl TryFrom<OrderItem> for app::OrderItem {
    type Error = Error;

    fn try_from(
        OrderItem {
            item_id,
            price,
            quantity,
        }: OrderItem,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(item_id.parse()?, price, quantity))
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ReceiveResponse {
    id: String,
}

pub(crate) async fn create_order<C: CommandService>(
    State(service): State<Arc<ServiceState<C>>>,
    Json(ReceiveRequest { order, order_items }): Json<ReceiveRequest>,
) -> impl IntoResponse {
    let Ok(order): Result<app::Order, _> = order.try_into() else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Ok(order_items): Result<Vec<app::OrderItem>, _> =
        order_items.into_iter().map(TryInto::try_into).collect()
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match service.command.create_order(order, order_items).await {
        Ok(id) => {
            let res = ReceiveResponse { id: id.to_string() };
            (StatusCode::ACCEPTED, Json(res)).into_response()
        }
        Err(e) => handling_app_error(&e).into_response(),
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UpdateStatusRequest {
    command: Command,
    current_aggregate_version: u64,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Command {
    Prepare,
    AssigningDeliveryPerson { delivery_person_id: String },
    ReadyForPickup,
    DeliveryPersonPickingUp,
    Delivered,
    Cancel,
}

pub(crate) async fn update_status<C: CommandService>(
    Path(aggregate_id): Path<String>,
    State(service): State<Arc<ServiceState<C>>>,
    Json(UpdateStatusRequest {
        command,
        current_aggregate_version,
    }): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    let Ok(aggregate_id) = aggregate_id.parse() else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match command {
        Command::Prepare => match service
            .command
            .preparing(aggregate_id, current_aggregate_version)
            .await
        {
            Ok(()) => StatusCode::ACCEPTED,
            Err(e) => handling_app_error(&e),
        }
        .into_response(),
        Command::AssigningDeliveryPerson { delivery_person_id } => {
            let Ok(delivery_person_id) = delivery_person_id.parse() else {
                return StatusCode::BAD_REQUEST.into_response();
            };
            match service
                .command
                .assign_delievery_person(
                    aggregate_id,
                    delivery_person_id,
                    current_aggregate_version,
                )
                .await
            {
                Ok(()) => StatusCode::ACCEPTED,
                Err(e) => handling_app_error(&e),
            }
            .into_response()
        }
        Command::ReadyForPickup => match service
            .command
            .ready_for_pick(aggregate_id, current_aggregate_version)
            .await
        {
            Ok(()) => StatusCode::ACCEPTED,
            Err(e) => handling_app_error(&e),
        }
        .into_response(),
        Command::DeliveryPersonPickingUp => match service
            .command
            .delivery_person_picking_up(aggregate_id, current_aggregate_version)
            .await
        {
            Ok(()) => StatusCode::ACCEPTED,
            Err(e) => handling_app_error(&e),
        }
        .into_response(),
        Command::Delivered => match service
            .command
            .delivered(aggregate_id, current_aggregate_version)
            .await
        {
            Ok(()) => StatusCode::ACCEPTED,
            Err(e) => handling_app_error(&e),
        }
        .into_response(),
        Command::Cancel => match service
            .command
            .cancel(aggregate_id, current_aggregate_version)
            .await
        {
            Ok(()) => StatusCode::ACCEPTED,
            Err(e) => handling_app_error(&e),
        }
        .into_response(),
    }
}
