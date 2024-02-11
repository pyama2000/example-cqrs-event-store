use std::sync::Arc;

use app::WidgetService;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use lib::Error;
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
            .route(
                "/create",
                post(|State(service): State<Arc<S>>| async move {
                    match service.create_widget(String::new(), String::new()).await {
                        Ok(id) => Ok((
                            StatusCode::CREATED,
                            Json(serde_json::json!({ "widget_id": id })),
                        )),
                        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
                    }
                }),
            )
            .route(
                "/name",
                post(|State(service): State<Arc<S>>| async move {
                    match service
                        .change_widget_name(String::new(), String::new())
                        .await
                    {
                        Ok(_) => Ok(StatusCode::ACCEPTED),
                        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
                    }
                }),
            )
            .route(
                "/description",
                post(|State(service): State<Arc<S>>| async move {
                    match service
                        .change_widget_description(String::new(), String::new())
                        .await
                    {
                        Ok(_) => Ok(StatusCode::ACCEPTED),
                        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
                    }
                }),
            )
            .with_state(self.service);
        let listener = TcpListener::bind(&self.addr).await?;
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        Ok(())
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
