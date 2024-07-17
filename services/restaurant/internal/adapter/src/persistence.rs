use aws_config::BehaviorVersion;

pub async fn dynamodb() -> aws_sdk_dynamodb::Client {
    let config = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
    aws_sdk_dynamodb::Client::new(&config)
}

pub async fn test_credential_dynamodb(endpoint_url: impl Into<String>) -> aws_sdk_dynamodb::Client {
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .endpoint_url(endpoint_url)
        .test_credentials()
        .load()
        .await;
    aws_sdk_dynamodb::Client::new(&config)
}
