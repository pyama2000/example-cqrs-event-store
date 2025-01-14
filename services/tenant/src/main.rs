use adapter::{dynamodb, CommandRepository};
use app::CommandUseCase;
use aws_config::BehaviorVersion;
use driver::server::{Server, Service};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let addr = format!(
        "[::1]:{}",
        std::env::var("PORT").map_err(|e| format!("PORT must be set: {e:?}"))?
    );
    let server = Server::new(Service::new(CommandUseCase::new(CommandRepository::new(
        dynamodb(&aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await),
    ))));
    println!("listing on: {addr}");
    server.run(addr.parse()?).await?;
    Ok(())
}
