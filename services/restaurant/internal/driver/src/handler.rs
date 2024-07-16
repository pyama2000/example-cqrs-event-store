use std::sync::Arc;

use app::{AppError, CommandService, Restaurant};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

fn handling_app_error(err: AppError) -> StatusCode {
    match err {
        AppError::AggregateConflicted => StatusCode::CONFLICT,
        AppError::KernelError(err) => match err {
            app::KernelError::InvalidRestaurantName
            | app::KernelError::InvalidItemName
            | app::KernelError::EntitiesIsEmpty => StatusCode::BAD_REQUEST,
            app::KernelError::AggregateNotFound => StatusCode::NOT_FOUND,
            app::KernelError::AggregateAlreadyCreated => StatusCode::CONFLICT,
            app::KernelError::AggregateNotCreated
            | app::KernelError::AggregateVersionOverflow
            | app::KernelError::EmptyEvent
            | app::KernelError::InvalidEvents
            | app::KernelError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        },
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CreateRestaurantRequest {
    name: String,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CreateRestaurantResponse {
    id: String,
}

pub(crate) async fn create_restaurant<C>(
    State(service): State<Arc<C>>,
    Json(CreateRestaurantRequest { name }): Json<CreateRestaurantRequest>,
) -> impl IntoResponse
where
    C: CommandService,
{
    match service.create_restaurant(Restaurant::new(name)).await {
        Ok(id) => {
            let res = CreateRestaurantResponse { id: id.to_string() };

            (StatusCode::ACCEPTED, Json(res)).into_response()
        }
        Err(e) => handling_app_error(e).into_response(),
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct AddItemsRequest {
    aggregate_version: u64,
    items: Vec<Item>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Item {
    name: String,
    price: Price,
    category: ItemCategory,
}

impl From<Item> for app::Item {
    fn from(
        Item {
            name,
            price,
            category,
        }: Item,
    ) -> Self {
        Self::new(name, price.into(), category.into())
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Price {
    Yen(u64),
}

impl From<Price> for app::Price {
    fn from(value: Price) -> Self {
        match value {
            Price::Yen(v) => Self::Yen(v),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ItemCategory {
    Food,
    Drink,
    Other(String),
}

impl From<ItemCategory> for app::ItemCategory {
    fn from(value: ItemCategory) -> Self {
        match value {
            ItemCategory::Food => Self::Food,
            ItemCategory::Drink => Self::Drink,
            ItemCategory::Other(v) => Self::Other(v),
        }
    }
}

pub(crate) async fn add_items<C>(
    Path(aggregate_id): Path<String>,
    State(service): State<Arc<C>>,
    Json(AddItemsRequest {
        aggregate_version,
        items,
    }): Json<AddItemsRequest>,
) -> impl IntoResponse
where
    C: CommandService,
{
    let items: Vec<app::Item> = items.into_iter().map(Into::into).collect();
    let Ok(aggregate_id) = aggregate_id.parse() else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    match service
        .add_items(aggregate_id, aggregate_version, items)
        .await
    {
        Ok(()) => StatusCode::ACCEPTED,
        Err(e) => handling_app_error(e),
    }
    .into_response()
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RemoveItemsRequest {
    aggregate_version: u64,
    item_ids: Vec<String>,
}

pub(crate) async fn remove_items<C>(
    Path(aggregate_id): Path<String>,
    State(service): State<Arc<C>>,
    Json(RemoveItemsRequest {
        aggregate_version,
        item_ids,
    }): Json<RemoveItemsRequest>,
) -> impl IntoResponse
where
    C: CommandService,
{
    let Ok(aggregate_id) = aggregate_id.parse() else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let (oks, errs): (Vec<_>, Vec<_>) = item_ids
        .into_iter()
        .map(|x| x.parse())
        .partition(Result::is_ok);
    if !errs.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let item_ids: Vec<_> = oks.into_iter().flatten().collect();

    match service
        .remove_items(aggregate_id, aggregate_version, item_ids)
        .await
    {
        Ok(()) => StatusCode::ACCEPTED,
        Err(e) => handling_app_error(e),
    }
    .into_response()
}
