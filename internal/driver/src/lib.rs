use std::sync::Arc;

use app::WidgetService;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use lib::Error;
use serde::Deserialize;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::signal;

#[derive(Debug, Clone)]
pub struct Server<T: ToSocketAddrs, S: WidgetService> {
    addr: T,
    service: Arc<S>,
}

impl<T: ToSocketAddrs, S: WidgetService + Send + Sync + 'static> Server<T, S> {
    pub fn new(addr: T, service: Arc<S>) -> Self {
        Self { addr, service }
    }

    pub async fn run(self) -> Result<(), Error> {
        let router = Router::new()
            .route("/healthz", get(|| async { StatusCode::OK }))
            .nest(
                "/widgets",
                Router::new()
                    .route(
                        "/",
                        post(
                            |State(service): State<Arc<S>>,
                             Json(CreateWidget {
                                 widget_name,
                                 widget_description,
                             }): Json<CreateWidget>| async move {
                                match service.create_widget(widget_name, widget_description).await {
                                    Ok(id) => Ok((
                                        StatusCode::CREATED,
                                        Json(serde_json::json!({ "widget_id": id })),
                                    )),
                                    Err(e) => {
                                        Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
                                    }
                                }
                            },
                        ),
                    )
                    .nest(
                        "/:widget_id",
                        Router::new()
                            .route(
                                "/name",
                                post(
                                    |Path(widget_id): Path<String>,
                                     State(service): State<Arc<S>>,
                                     Json(ChangeWidgetName { widget_name }): Json<
                                        ChangeWidgetName,
                                    >| async move {
                                        match service
                                            .change_widget_name(widget_id, widget_name)
                                            .await
                                        {
                                            Ok(_) => Ok(StatusCode::ACCEPTED),
                                            Err(e) => Err((
                                                StatusCode::INTERNAL_SERVER_ERROR,
                                                e.to_string(),
                                            )),
                                        }
                                    },
                                ),
                            )
                            .route(
                                "/description",
                                post(
                                    |Path(widget_id): Path<String>,
                                     State(service): State<Arc<S>>,
                                     Json(ChangeWidgetDescription { widget_description }): Json<
                                        ChangeWidgetDescription,
                                    >| async move {
                                        match service
                                            .change_widget_description(
                                                widget_id,
                                                widget_description,
                                            )
                                            .await
                                        {
                                            Ok(_) => Ok(StatusCode::ACCEPTED),
                                            Err(e) => Err((
                                                StatusCode::INTERNAL_SERVER_ERROR,
                                                e.to_string(),
                                            )),
                                        }
                                    },
                                ),
                            ),
                    ),
            )
            .with_state(self.service);
        let listener = TcpListener::bind(&self.addr).await?;
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
