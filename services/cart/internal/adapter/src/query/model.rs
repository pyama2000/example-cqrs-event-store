// NOTE: Query領域だがデータベースを分けないのでCommandで定義したテーブルモデルを利用する
impl TryFrom<crate::command::model::AggregateModel> for kernel::query::model::Cart {
    type Error = anyhow::Error;

    fn try_from(value: crate::command::model::AggregateModel) -> Result<Self, Self::Error> {
        use std::str::FromStr as _;

        use anyhow::Context as _;
        use kernel::id::Id;
        use kernel::query::model::{Cart, Item, Tenant};

        let items_list: Vec<_> = match value.payload() {
            crate::command::model::AggregatePayload::V1 {
                items,
                is_order_placed: _,
            } => items
                .iter()
                .map(|(tenant_id, quantity_by_item_id)| {
                    let tenant_id: Id<Tenant> = Id::from_str(tenant_id)
                        .with_context(|| format!("parse tenant id: {tenant_id}"))?;
                    let items: Vec<_> = quantity_by_item_id
                        .iter()
                        .map(|(item_id, quantity)| {
                            let item_id: Id<Item> = Id::from_str(item_id)
                                .with_context(|| format!("parse item id: {item_id}"))?;
                            Ok::<_, Self::Error>(Item::new(tenant_id.clone(), item_id, *quantity))
                        })
                        .collect::<Result<_, _>>()
                        .with_context(|| {
                            format!("transform quantity_by_item_id: {quantity_by_item_id:?}")
                        })?;
                    Ok::<_, Self::Error>(items)
                })
                .collect::<Result<_, _>>()
                .with_context(|| format!("transform items: {items:?}"))?,
        };
        let items: Vec<_> = items_list.into_iter().flatten().collect();
        Ok(Cart::new(
            Id::from_str(value.id()).with_context(|| format!("parse cart id: {}", value.id()))?,
            items,
        ))
    }
}
