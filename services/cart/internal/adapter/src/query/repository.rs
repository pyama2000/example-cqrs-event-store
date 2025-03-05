use crate::AGGREGATE_TABLE_NAME;

#[derive(Debug, Clone)]
pub struct QueryRepository {
    dynamodb: aws_sdk_dynamodb::Client,
}

impl QueryRepository {
    #[must_use]
    pub fn new(dynamodb: aws_sdk_dynamodb::Client) -> Self {
        Self { dynamodb }
    }
}

impl kernel::query::processor::QueryProcessor for QueryRepository {
    #[tracing::instrument(skip(self), err, ret)]
    async fn get(
        &self,
        id: kernel::id::Id<kernel::query::model::Cart>,
    ) -> Result<
        Result<Option<kernel::query::model::Cart>, kernel::query::error::QueryKernelError>,
        anyhow::Error,
    > {
        use anyhow::Context as _;
        use aws_sdk_dynamodb::operation::get_item::GetItemError::ResourceNotFoundException;
        use aws_sdk_dynamodb::types::AttributeValue;

        let result = self
            .dynamodb
            .get_item()
            .table_name(AGGREGATE_TABLE_NAME)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await;
        if let Err(e) = result {
            match e.into_service_error() {
                ResourceNotFoundException(_) => return Ok(Ok(None)),
                e => return Err(e.into()),
            }
        }
        let Some(item) = result?.item else {
            return Ok(Ok(None));
        };
        // NOTE: Query領域だがデータベースを分けないのでCommandで定義したテーブルモデルを利用する
        let aggregate: crate::command::model::AggregateModel = serde_dynamo::from_item(item)
            .with_context(|| "from DynamoDB item to AggregateModel")?;
        Ok(Ok(Some(aggregate.try_into()?)))
    }
}
