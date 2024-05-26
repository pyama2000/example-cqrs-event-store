use lib::Error;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};

pub type ConnectionPool = Pool<MySql>;

pub async fn connect(url: &str) -> Result<ConnectionPool, Error> {
    Ok(MySqlPoolOptions::new()
        .max_connections(20)
        .connect(url)
        .await?)
}

pub type DbClient = aws_sdk_dynamodb::Client;

pub async fn client() -> DbClient {
    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::v2024_03_28()).await;
    let retry_config = aws_config::retry::RetryConfig::standard()
        .with_max_attempts(3)
        .with_initial_backoff(std::time::Duration::from_secs(5));
    let config = aws_sdk_dynamodb::config::Builder::from(&sdk_config)
        .retry_config(retry_config)
        .build();
    aws_sdk_dynamodb::Client::from_conf(config)
}
