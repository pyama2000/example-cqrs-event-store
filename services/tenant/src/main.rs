use adapter::CommandRepository;
use app::CommandUseCase;
use driver::server::{Server, Service};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let addr = format!(
        "0.0.0.0:{}",
        std::env::var("PORT").map_err(|e| format!("PORT must be set: {e:?}"))?
    );
    let server = Server::new(Service::new(CommandUseCase::new(CommandRepository)));
    println!("listing on: {addr}");
    server.run(addr.parse()?).await?;
    Ok(())
}
