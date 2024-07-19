use adapter::{test_credential_dynamodb, CommandRepository};
use app::CommandUseCase;
use driver::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let dynamodb = test_credential_dynamodb(format!(
        "http://127.0.0.1:{}",
        option_env!("LOCALSTACK_GATEWAY_PORT").unwrap_or_else(|| "4566")
    ))
    .await;

    let repository = CommandRepository::new(dynamodb);
    let usecase = CommandUseCase::new(repository);
    let server = Server::new("0.0.0.0:8080", usecase.into());
    server.run().await?;
    Ok(())
}
