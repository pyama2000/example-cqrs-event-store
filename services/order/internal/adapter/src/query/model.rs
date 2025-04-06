// NOTE: Query領域だがデータベースを分けないのでCommandで定義したテーブルモデルを利用する
pub(crate) use crate::command::model::AggregateModel;

impl TryFrom<AggregateModel> for kernel::query::model::Order {
    type Error = anyhow::Error;

    fn try_from(value: AggregateModel) -> Result<Self, Self::Error> {
        use anyhow::Context as _;
        use kernel::id::Id;
        use kernel::query::model::{Item, OrderStatus, Tenant};

        let (items, status): (Vec<Item>, kernel::query::model::OrderStatus) = match value.payload()
        {
            crate::command::model::AggregatePayload::V1 {
                items,
                order_status,
                ..
            } => {
                let items: Vec<_> = items
                    .iter()
                    .map(|item| {
                        let (id, tenant_id, quantity): (Id<Item>, Id<Tenant>, u32) = match item {
                            crate::command::model::Item::V1 {
                                id,
                                tenant_id,
                                quantity,
                            } => {
                                let id = id.parse().with_context(|| "parse item id")?;
                                let tenant_id =
                                    tenant_id.parse().with_context(|| "parse tenant id")?;
                                (id, tenant_id, *quantity)
                            }
                        };
                        Ok::<_, anyhow::Error>(Item::new(id, tenant_id, quantity))
                    })
                    .collect::<Result<_, _>>()
                    .with_context(|| format!("transform items: {items:?}"))?;
                let status = match order_status {
                    crate::command::model::OrderStatus::Created => OrderStatus::Received,
                    crate::command::model::OrderStatus::Prepared => OrderStatus::Prepared,
                    crate::command::model::OrderStatus::PickedUp => OrderStatus::OnTheWay,
                    crate::command::model::OrderStatus::Delivered => OrderStatus::Delivered,
                    crate::command::model::OrderStatus::Canceled => OrderStatus::Canceled,
                };
                (items, status)
            }
        };
        Ok(Self::new(
            value.id().parse().with_context(|| "parse order id")?,
            items,
            status,
        ))
    }
}

// NOTE: Query領域だがデータベースを分けないのでCommandで定義したモデルを利用する
pub(crate) use crate::command::model::OrderStatus;
