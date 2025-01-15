use kernel::QueryProcessor;

use crate::AGGREGATE_TABLE_NAME;

use super::{AggregateModel, AggregatePayload};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct QueryRepository {
    dynamodb: aws_sdk_dynamodb::Client,
}

impl QueryRepository {
    /// Creates a new [`QueryRepository`].
    #[must_use]
    pub fn new(dynamodb: aws_sdk_dynamodb::Client) -> Self {
        Self { dynamodb }
    }
}

impl QueryProcessor for QueryRepository {
    async fn list_tenants(&self) -> Result<Vec<kernel::query::Tenant>, Error> {
        let models: Vec<AggregateModel> = serde_dynamo::from_items(
            self.dynamodb
                .scan()
                .table_name(AGGREGATE_TABLE_NAME)
                .send()
                .await?
                .items()
                .to_vec(),
        )?;
        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn list_items(
        &self,
        tenant_id: kernel::Id<kernel::Aggregate>,
    ) -> Result<Option<Vec<kernel::query::Item>>, Error> {
        use aws_sdk_dynamodb::operation::get_item::GetItemError::ResourceNotFoundException;
        use aws_sdk_dynamodb::types::AttributeValue;

        let output = match self
            .dynamodb
            .get_item()
            .table_name(AGGREGATE_TABLE_NAME)
            .key("id", AttributeValue::S(tenant_id.to_string()))
            .send()
            .await
        {
            Ok(o) => o,
            Err(e) => match e.into_service_error() {
                ResourceNotFoundException(_) => return Ok(None),
                e => return Err(e.into()),
            },
        };
        let Some(item) = output.item else {
            return Ok(None);
        };
        let model: AggregateModel = serde_dynamo::from_item(item)?;
        match model.payload() {
            AggregatePayload::V1 { items, .. } => {
                Ok(Some(items.iter().cloned().map(Into::into).collect()))
            }
        }
    }
}
