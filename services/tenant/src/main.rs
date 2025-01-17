use adapter::{dynamodb, CommandRepository, QueryRepository};
use app::{CommandUseCase, QueryUseCase};
use aws_config::BehaviorVersion;
use driver::server::{Server, Service};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let addr = format!(
        "[::1]:{}",
        std::env::var("PORT").map_err(|e| format!("PORT must be set: {e:?}"))?
    );
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .endpoint_url(format!(
            "http://localhost:{}",
            std::env::var("LOCALSTACK_GATEWAY_PORT").unwrap_or("4566".to_string())
        ))
        .test_credentials()
        .load()
        .await;
    let dynamodb = dynamodb(&config);
    let server = Server::new(Service::new(
        CommandUseCase::new(CommandRepository::new(dynamodb.clone())),
        QueryUseCase::new(QueryRepository::new(dynamodb)),
    ));
    println!("listing on: {addr}");
    server.run(addr.parse()?).await?;
    Ok(())
}
