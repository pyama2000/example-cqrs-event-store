use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AggregateModel {
    id: String,
    version: u64,
    payload: AggregatePayload,
}

impl AggregateModel {
    pub(crate) fn payload(&self) -> &AggregatePayload {
        &self.payload
    }
}

impl From<AggregateModel> for kernel::query::Tenant {
    fn from(value: AggregateModel) -> Self {
        let name = match value.payload {
            AggregatePayload::V1 { name, .. } => name,
        };
        Self::new(value.id, name)
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AggregatePayload {
    V1 { name: String, items: Vec<Item> },
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Item {
    V1 {
        id: String,
        name: String,
        price: u32,
    },
}

impl From<Item> for kernel::query::Item {
    fn from(value: Item) -> Self {
        match value {
            Item::V1 { id, name, price } => Self::new(id, name, price),
        }
    }
}
