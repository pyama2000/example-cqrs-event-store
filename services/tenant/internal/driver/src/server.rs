use tokio::signal;

pub struct Server;

/// サーバーを安全に終了するための仕組み(Graceful shutdown)
async fn _shutdown_signal() {
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
        () = ctrl_c => tracing::trace!("receive ctrl_c signal"),
        () = terminate => tracing::trace!("receive terminate"),
    }
}
