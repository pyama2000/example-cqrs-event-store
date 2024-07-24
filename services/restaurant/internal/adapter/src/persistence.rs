use aws_config::BehaviorVersion;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};

pub async fn dynamodb(endpoint_url: impl Into<String>) -> aws_sdk_dynamodb::Client {
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .endpoint_url(endpoint_url)
        .test_credentials()
        .load()
        .await;
    aws_sdk_dynamodb::Client::new(&config)
}

/// `MySQL` クライアント
///
/// # Errors
///
/// コネクションプールの作成に失敗するとエラーが起きる
pub async fn mysql(
    url: impl Into<String>,
) -> Result<Pool<MySql>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    Ok(MySqlPoolOptions::new()
        .max_connections(20)
        .connect(&url.into())
        .await?)
}
