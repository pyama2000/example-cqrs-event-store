#[must_use]
pub fn dynamodb(config: &aws_config::SdkConfig) -> aws_sdk_dynamodb::Client {
    aws_sdk_dynamodb::Client::new(config)
}
