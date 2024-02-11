use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use lib::Result;
use tokio::net::{TcpListener, ToSocketAddrs};

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
        axum::serve(listener, app).await?;
        Ok(())
    }
}
