use crate::query::model::AggregateModel;
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

    /// 集約テーブルから全てのレコードを取得する
    #[tracing::instrument(skip(self), err(Debug), ret)]
    async fn list_aggregate_models(&self) -> Result<Vec<AggregateModel>, anyhow::Error> {
        use anyhow::Context as _;

        let result = self
            .dynamodb
            .scan()
            .table_name(AGGREGATE_TABLE_NAME)
            .send()
            .await
            .with_context(|| "scan Aggregate table")?;
        serde_dynamo::from_items(result.items().to_vec())
            .with_context(|| "from DynamoDB items to aggregate models")
    }
}

impl kernel::query::processor::QueryProcessor for QueryRepository {
    #[tracing::instrument(skip(self), err(Debug), ret)]
    async fn get_by_order_id(
        &self,
        id: kernel::id::Id<kernel::query::model::Order>,
    ) -> Result<
        Result<Option<kernel::query::model::Order>, kernel::query::error::QueryKernelError>,
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
        let model: AggregateModel = serde_dynamo::from_item(item)
            .with_context(|| "from DynamoDB item to AggregateModel")?;
        Ok(Ok(Some(
            model
                .try_into()
                .with_context(|| "try from aggregate model to cart")?,
        )))
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
    async fn get_by_cart_id(
        &self,
        id: kernel::id::Id<kernel::query::model::Cart>,
    ) -> Result<
        Result<Option<kernel::query::model::Order>, kernel::query::error::QueryKernelError>,
        anyhow::Error,
    > {
        use anyhow::Context as _;

        let models = self
            .list_aggregate_models()
            .await
            .with_context(|| "list aggregate model")?;
        let Some(model) = models.into_iter().find(|model| match model.payload() {
            crate::command::model::AggregatePayload::V1 { cart_id, .. } => {
                cart_id == &id.to_string()
            }
        }) else {
            return Ok(Ok(None));
        };
        Ok(Ok(Some(
            model
                .try_into()
                .with_context(|| "try from aggregate model to cart")?,
        )))
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
    async fn list_tenant_received_order_ids(
        &self,
        tenant_id: kernel::id::Id<kernel::query::model::Tenant>,
    ) -> Result<
        Result<
            Vec<kernel::id::Id<kernel::query::model::Order>>,
            kernel::query::error::QueryKernelError,
        >,
        anyhow::Error,
    > {
        use crate::query::model::OrderStatus;
        use anyhow::Context as _;
        use kernel::id::Id;
        use std::str::FromStr as _;

        let models = self
            .list_aggregate_models()
            .await
            .with_context(|| "list aggregate model")?;
        let order_ids: Vec<_> = models
            .into_iter()
            .filter(|model| match model.payload() {
                crate::command::model::AggregatePayload::V1 {
                    items,
                    order_status,
                    ..
                } => {
                    if *order_status != OrderStatus::Created {
                        return false;
                    }
                    items.iter().any(|item| match item {
                        crate::command::model::Item::V1 { tenant_id: id, .. } => {
                            id == &tenant_id.to_string()
                        }
                    })
                }
            })
            .map(|model| model.id().to_string())
            .collect();
        let order_ids: Vec<_> = order_ids
            .iter()
            .map(|id| Id::from_str(id))
            .collect::<Result<_, _>>()
            .with_context(|| format!("parse order ids: {order_ids:?}"))?;
        Ok(Ok(order_ids))
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
    async fn list_prepared_order_ids(
        &self,
    ) -> Result<
        Result<
            Vec<kernel::id::Id<kernel::query::model::Order>>,
            kernel::query::error::QueryKernelError,
        >,
        anyhow::Error,
    > {
        use crate::query::model::OrderStatus;
        use anyhow::Context as _;
        use kernel::id::Id;
        use std::str::FromStr as _;

        let models = self
            .list_aggregate_models()
            .await
            .with_context(|| "list aggregate model")?;
        let order_ids: Vec<_> = models
            .into_iter()
            .filter(|model| match model.payload() {
                crate::command::model::AggregatePayload::V1 { order_status, .. } => {
                    *order_status == OrderStatus::Prepared
                }
            })
            .map(|model| model.id().to_string())
            .collect();
        let order_ids: Vec<_> = order_ids
            .iter()
            .map(|id| Id::from_str(id))
            .collect::<Result<_, _>>()
            .with_context(|| format!("parse order ids: {order_ids:?}"))?;
        Ok(Ok(order_ids))
    }
}
