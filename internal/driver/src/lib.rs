use std::sync::Arc;
use std::time::Duration;

use app::{WidgetService, WidgetServiceError};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use lib::Error;
use serde::Deserialize;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::signal;
use tower_http::timeout::TimeoutLayer;

#[derive(Debug, Clone)]
pub struct Server<T: ToSocketAddrs, S: WidgetService> {
    addr: T,
    service: Arc<S>,
}

impl<T: ToSocketAddrs + std::fmt::Display, S: WidgetService + Send + Sync + 'static> Server<T, S> {
    pub fn new(addr: T, service: Arc<S>) -> Self {
        Self { addr, service }
    }

    pub async fn run(self) -> Result<(), Error> {
        let router = Router::new()
            .route("/healthz", get(|| async { StatusCode::OK }))
            .nest(
                "/widgets",
                Router::new().route("/", post(create_widget)).nest(
                    "/:widget_id",
                    Router::new()
                        .route("/name", post(change_widget_name))
                        .route("/description", post(change_widget_description)),
                ),
            )
            .with_state(self.service)
            .layer(TimeoutLayer::new(Duration::from_millis(100)));
        let listener = TcpListener::bind(&self.addr).await?;
        println!("listening: {}", &self.addr);
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct CreateWidget {
    widget_name: String,
    widget_description: String,
}

#[derive(Deserialize)]
struct ChangeWidgetName {
    widget_name: String,
}

#[derive(Deserialize)]
struct ChangeWidgetDescription {
    widget_description: String,
}

async fn create_widget<S: WidgetService>(
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

async fn change_widget_name<S: WidgetService>(
    Path(widget_id): Path<String>,
    State(service): State<Arc<S>>,
    Json(ChangeWidgetName { widget_name }): Json<ChangeWidgetName>,
) -> impl IntoResponse {
    match service.change_widget_name(widget_id, widget_name).await {
        Ok(_) => StatusCode::ACCEPTED.into_response(),
        Err(e) => handling_service_error(e).into_response(),
    }
}

async fn change_widget_description<S: WidgetService>(
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
        WidgetServiceError::Unknow(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .unwrap_or_else(|e| panic!("failed to install Ctrl+C handler: {e}"))
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .unwrap_or_else(|e| panic!("failed to install signal handler: {e}"))
            .recv()
            .await;
    };
    tokio::select! {
        _ = ctrl_c => println!("receive ctrl_c signal"),
        _ = terminate => println!("receive terminate"),
    }
    println!("signal received, starting graceful shutdown");
}
