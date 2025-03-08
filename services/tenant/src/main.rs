use adapter::{dynamodb, CommandRepository, QueryRepository};
use app::{CommandUseCase, QueryUseCase};
use aws_config::BehaviorVersion;
use driver::server::{Server, Service};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let shutdown_providers =
        observability::provider::init_providers(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))?;
    let addr = format!(
        "{}:{}",
        std::env::var("HOST").unwrap_or("0.0.0.0".to_string()),
        std::env::var("PORT").unwrap_or(50051.to_string()),
    );
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .endpoint_url(format!(
            "http://{}:{}",
            std::env::var("LOCALSTACK_GATEWAY_HOST").unwrap_or("localhost".to_string()),
            std::env::var("LOCALSTACK_GATEWAY_PORT").unwrap_or("4566".to_string()),
        ))
        .test_credentials()
        .load()
        .await;
    let dynamodb = dynamodb(&config);
    let server = Server::new(Service::new(
        CommandUseCase::new(CommandRepository::new(dynamodb.clone())),
        QueryUseCase::new(QueryRepository::new(dynamodb)),
    ));
    tracing::info!("listing on: {addr}");
    server.run(addr.parse()?).await?;
    shutdown_providers()?;
    Ok(())
}
