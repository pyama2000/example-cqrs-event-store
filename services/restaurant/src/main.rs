use adapter::{test_credential_dynamodb, CommandRepository};
use app::CommandUseCase;
use driver::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let dynamodb = test_credential_dynamodb(format!(
        "http://127.0.0.1:{}",
        option_env!("DYNAMODB_PORT").unwrap_or_else(|| "8000")
    ))
    .await;

    create_table(&dynamodb).await?;

    let repository = CommandRepository::new(dynamodb);
    let usecase = CommandUseCase::new(repository);
    let server = Server::new("0.0.0.0:8080", usecase.into());
    server.run().await?;
    Ok(())
}

async fn create_table(
    dynamodb: &aws_sdk_dynamodb::Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    const AGGREGATE_TABLE_NAME: &str = "restaurant-aggregate";
    const EVENT_TABLE_NAME: &str = "restaurant-event";
    use aws_sdk_dynamodb::types::{
        AttributeDefinition, BillingMode, KeySchemaElement, KeyType, ProvisionedThroughput,
    };

    let _ = dynamodb
        .create_table()
        .table_name(AGGREGATE_TABLE_NAME)
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("id")
                .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
                .build()?,
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("id")
                .key_type(KeyType::Hash)
                .build()?,
        )
        .billing_mode(BillingMode::Provisioned)
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(20)
                .write_capacity_units(20)
                .build()?,
        )
        .send()
        .await;
    let _ = dynamodb
        .create_table()
        .table_name(EVENT_TABLE_NAME)
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("aggregate_id")
                .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
                .build()?,
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("aggregate_id")
                .key_type(KeyType::Hash)
                .build()?,
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("id")
                .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
                .build()?,
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("id")
                .key_type(KeyType::Range)
                .build()?,
        )
        .billing_mode(BillingMode::Provisioned)
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(20)
                .write_capacity_units(20)
                .build()?,
        )
        .send()
        .await;
    Ok(())
}
