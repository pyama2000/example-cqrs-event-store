use adapter::{dynamodb, mysql, CommandRepository, QueryRepository};
use app::{CommandUseCase, QueryUseCase};
use driver::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let dynamodb = dynamodb(format!(
        "http://127.0.0.1:{}",
        option_env!("LOCALSTACK_GATEWAY_PORT").unwrap_or_else(|| "4566")
    ))
    .await;
    let mysql = mysql(format!(
        "mysql://root:root@127.0.0.1:{}/query_model",
        option_env!("MYSQL_PORT").unwrap_or_else(|| "3306")
    ))
    .await?;

    let server = Server::new(
        "0.0.0.0:8080",
        CommandUseCase::new(CommandRepository::new(dynamodb)),
        QueryUseCase::new(QueryRepository::new(mysql)),
    );
    server.run().await?;
    Ok(())
}
