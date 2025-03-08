#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let shutdown_providers =
        observability::provider::init_providers(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))?;
    let addr = format!(
        "[::1]:{}",
        std::env::var("PORT").map_err(|e| format!("PORT must be set: {e:?}"))?
    );
    let config = aws_config::defaults(aws_config::BehaviorVersion::v2024_03_28())
        .endpoint_url(format!(
            "http://localhost:{}",
            std::env::var("LOCALSTACK_GATEWAY_PORT").unwrap_or("4566".to_string())
        ))
        .test_credentials()
        .load()
        .await;
    let dynamodb = adapter::command::persistence::dynamodb(&config);
    let server = driver::server::Server::new(driver::server::Service::new(
        app::command::usecase::CommandUseCase::new(
            adapter::command::repository::CommandRepository::new(dynamodb.clone()),
        ),
        app::query::usecase::QueryUseCase::new(adapter::query::repository::QueryRepository::new(
            dynamodb,
        )),
    ));
    tracing::info!("listing on: {addr}");
    server.run(addr.parse()?).await?;
    shutdown_providers()?;
    Ok(())
}
