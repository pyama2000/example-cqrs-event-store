use std::fmt::Debug;
use std::sync::Arc;

use app::{WidgetService, WidgetServiceError};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use opentelemetry_semantic_conventions::trace::{EXCEPTION_ESCAPED, EXCEPTION_MESSAGE, EXCEPTION_TYPE};
use serde::Deserialize;
use tracing::instrument;

#[instrument]
pub async fn healthz() -> StatusCode {
    StatusCode::OK
}

#[instrument]
pub async fn panic() -> StatusCode {
    panic!("panic handler");
}

#[derive(Deserialize)]
pub struct CreateWidget {
    widget_name: String,
    widget_description: String,
}

#[instrument]
pub async fn create_widget<S: WidgetService + Debug>(
    State(service): State<Arc<S>>,
    Json(CreateWidget {
        widget_name,
        widget_description,
    }): Json<CreateWidget>,
) -> impl IntoResponse {
    match service.create_widget(widget_name, widget_description).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "widget_id": id })),
        )
            .into_response(),
        Err(e) => handling_service_error(e).into_response(),
    }
}

#[derive(Deserialize)]
pub struct ChangeWidgetName {
    widget_name: String,
}

#[instrument]
pub async fn change_widget_name<S: WidgetService + Debug>(
    Path(widget_id): Path<String>,
    State(service): State<Arc<S>>,
    Json(ChangeWidgetName { widget_name }): Json<ChangeWidgetName>,
) -> impl IntoResponse {
    match service.change_widget_name(widget_id, widget_name).await {
        Ok(_) => StatusCode::ACCEPTED.into_response(),
        Err(e) => handling_service_error(e).into_response(),
    }
}

#[derive(Deserialize)]
pub struct ChangeWidgetDescription {
    widget_description: String,
}

#[instrument]
pub async fn change_widget_description<S: WidgetService + Debug>(
    Path(widget_id): Path<String>,
    State(service): State<Arc<S>>,
    Json(ChangeWidgetDescription { widget_description }): Json<ChangeWidgetDescription>,
) -> impl IntoResponse {
    match service
        .change_widget_description(widget_id, widget_description)
        .await
    {
        Ok(_) => StatusCode::ACCEPTED.into_response(),
        Err(e) => handling_service_error(e).into_response(),
    }
}

fn handling_service_error(err: WidgetServiceError) -> impl IntoResponse {
    match err {
        WidgetServiceError::AggregateNotFound => StatusCode::NOT_FOUND.into_response(),
        WidgetServiceError::AggregateConfilict => StatusCode::CONFLICT.into_response(),
        WidgetServiceError::InvalidValue => StatusCode::BAD_REQUEST.into_response(),
        WidgetServiceError::Unknown(ref e) => {
            tracing::event!(
                tracing::Level::ERROR,
                { EXCEPTION_ESCAPED } = true,
                { EXCEPTION_MESSAGE } = e,
                { EXCEPTION_TYPE } = ?err,
                "exception"
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
