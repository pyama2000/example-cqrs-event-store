use std::sync::Arc;

use app::{AppError, CommandService, QueryService};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::server::ServiceState;

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

pub(crate) async fn create_restaurant<C, Q>(
    State(service): State<Arc<ServiceState<C, Q>>>,
    Json(CreateRestaurantRequest { name }): Json<CreateRestaurantRequest>,
) -> impl IntoResponse
where
    C: CommandService,
    Q: QueryService,
{
    match service
        .command
        .create_restaurant(app::Restaurant::new(name))
        .await
    {
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
}

impl From<Item> for app::Item {
    fn from(
        Item {
            name,
            price,
        }: Item,
    ) -> Self {
        Self::new(name, price.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl From<app::Price> for Price {
    fn from(value: app::Price) -> Self {
        match value {
            app::Price::Yen(v) => Self::Yen(v),
        }
    }
}

pub(crate) async fn add_items<C, Q>(
    Path(aggregate_id): Path<String>,
    State(service): State<Arc<ServiceState<C, Q>>>,
    Json(AddItemsRequest {
        aggregate_version,
        items,
    }): Json<AddItemsRequest>,
) -> impl IntoResponse
where
    C: CommandService,
    Q: QueryService,
{
    let items: Vec<app::Item> = items.into_iter().map(Into::into).collect();
    let Ok(aggregate_id) = aggregate_id.parse() else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    match service
        .command
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

pub(crate) async fn remove_items<C, Q>(
    Path(aggregate_id): Path<String>,
    State(service): State<Arc<ServiceState<C, Q>>>,
    Json(RemoveItemsRequest {
        aggregate_version,
        item_ids,
    }): Json<RemoveItemsRequest>,
) -> impl IntoResponse
where
    C: CommandService,
    Q: QueryService,
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
        .command
        .remove_items(aggregate_id, aggregate_version, item_ids)
        .await
    {
        Ok(()) => StatusCode::ACCEPTED,
        Err(e) => handling_app_error(e),
    }
    .into_response()
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListRestaurantsResponse {
    restaurants: Vec<Restaurant>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Restaurant {
    aggregate_id: String,
    name: String,
}

pub(crate) async fn list_restaurants<C, Q>(
    State(service): State<Arc<ServiceState<C, Q>>>,
) -> impl IntoResponse
where
    C: CommandService,
    Q: QueryService,
{
    match service.query.list_restaurants().await {
        Ok(restaurant_by_id) => {
            let restaurants: Vec<_> = restaurant_by_id
                .into_iter()
                .map(|(id, restaurant)| Restaurant {
                    aggregate_id: id.to_string(),
                    name: restaurant.name().to_string(),
                })
                .collect();
            (
                StatusCode::OK,
                Json(ListRestaurantsResponse { restaurants }),
            )
                .into_response()
        }
        Err(e) => handling_app_error(e).into_response(),
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListItemsResponse {
    items: Vec<ListItemsItem>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListItemsItem {
    id: String,
    name: String,
    price: Price,
}

pub(crate) async fn list_items<C, Q>(
    Path(aggregate_id): Path<String>,
    State(service): State<Arc<ServiceState<C, Q>>>,
) -> impl IntoResponse
where
    C: CommandService,
    Q: QueryService,
{
    let Ok(aggregate_id) = aggregate_id.parse() else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match service.query.list_items(aggregate_id).await {
        Ok(item_by_id) => {
            let items: Vec<_> = item_by_id
                .into_iter()
                .map(|(id, item)| ListItemsItem {
                    id: id.to_string(),
                    name: item.name().to_string(),
                    price: item.price().clone().into(),
                })
                .collect();
            (StatusCode::OK, Json(ListItemsResponse { items })).into_response()
        }
        Err(e) => handling_app_error(e).into_response(),
    }
}
