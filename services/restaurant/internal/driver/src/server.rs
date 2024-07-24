use std::sync::Arc;

use app::{CommandService, QueryService};
use axum::routing::post;
use axum::Router;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::signal;

use crate::create_restaurant;
use crate::handler::{add_items, list_items, list_restaurants, remove_items};

pub struct ServiceState<C: CommandService, Q: QueryService> {
    pub command: C,
    pub query: Q,
}

pub struct Server<T: ToSocketAddrs> {
    addr: T,
    router: Router,
}

impl<T: ToSocketAddrs + std::fmt::Display> Server<T> {
    pub fn new<C, Q>(addr: T, command_service: C, query_service: Q) -> Self
    where
        C: CommandService + Send + Sync + 'static,
        Q: QueryService + Send + Sync + 'static,
    {
        let state = ServiceState {
            command: command_service,
            query: query_service,
        };
        let router = Router::new()
            .nest(
                "/restaurants",
                Router::new()
                    .route("/", post(create_restaurant).get(list_restaurants))
                    .nest(
                        "/:aggregate_id",
                        Router::new().route(
                            "/items",
                            post(add_items).get(list_items).delete(remove_items),
                        ),
                    ),
            )
            .with_state(Arc::new(state));
        Self { addr, router }
    }

    /// サーバーを起動する
    ///
    /// # Errors
    ///
    /// 起動に失敗した際にエラーが発生する
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let listener = TcpListener::bind(&self.addr).await?;
        axum::serve(listener, self.router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        Ok(())
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .unwrap_or_else(|e| panic!("failed to install Ctrl+C handler: {e}"));
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .unwrap_or_else(|e| panic!("failed to install signal handler: {e}"))
            .recv()
            .await;
    };
    tokio::select! {
        () = ctrl_c => println!("receive ctrl_c signal"),
        () = terminate => println!("receive terminate"),
    }
}
