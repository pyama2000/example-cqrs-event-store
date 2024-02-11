use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use lib::Result;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::signal;

#[derive(Debug, Clone)]
pub struct Server<T: ToSocketAddrs> {
    addr: T,
    router: Router,
}

impl<T: ToSocketAddrs> Server<T> {
    pub fn new(addr: T) -> Self {
        Self {
            addr,
            router: Router::new(),
        }
    }

    pub async fn run(self) -> Result<()> {
        let app = self
            .router
            .route("/healthz", get(|| async { StatusCode::OK }));
        let listener = TcpListener::bind(&self.addr).await?;
        axum::serve(listener, app)
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
