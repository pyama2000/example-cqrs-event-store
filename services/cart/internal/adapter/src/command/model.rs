use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Context as _;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};

/// 集約のテーブルモデル
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct AggregateModel {
    id: String,
    version: u64,
    payload: AggregatePayload,
}

impl AggregateModel {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn version(&self) -> u64 {
        self.version
    }

    pub(crate) fn payload(&self) -> &AggregatePayload {
        &self.payload
    }

    pub(crate) fn version_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, anyhow::Error> {
        Ok(serde_dynamo::to_attribute_value(self.version)?)
    }

    pub(crate) fn payload_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, anyhow::Error> {
        Ok(serde_dynamo::to_attribute_value(&self.payload)?)
    }
}

impl From<kernel::command::model::aggregate::Aggregate> for AggregateModel {
    fn from(value: kernel::command::model::aggregate::Aggregate) -> Self {
        Self {
            id: value.id().to_string(),
            version: value.version(),
            payload: value.into(),
        }
    }
}

impl TryFrom<AggregateModel> for kernel::command::model::aggregate::Aggregate {
    type Error = anyhow::Error;

    fn try_from(
        AggregateModel {
            id,
            version,
            payload,
        }: AggregateModel,
    ) -> Result<Self, Self::Error> {
        use kernel::command::model::aggregate::Aggregate;
        use kernel::command::model::entity::{Item, Tenant};
        use kernel::id::Id;

        let (items, is_order_placed) = match payload {
            AggregatePayload::V1 {
                items,
                is_order_placed,
            } => {
                let items: HashMap<Id<Tenant>, HashMap<Id<Item>, u32>> = items
                    .iter()
                    .map(|(tenant_id, quantity_by_item_id)| {
                        let tenant_id: Id<Tenant> = Id::from_str(tenant_id)
                            .with_context(|| format!("parse tenant id: {tenant_id}"))?;
                        let quantity_by_item_id: HashMap<Id<Item>, u32> = quantity_by_item_id
                            .iter()
                            .map(|(item_id, quantity)| {
                                let item_id: Id<Item> = Id::from_str(item_id)
                                    .with_context(|| format!("parse item id: {item_id}"))?;
                                Ok::<_, Self::Error>((item_id, *quantity))
                            })
                            .collect::<Result<_, _>>()
                            .with_context(|| {
                                format!("transform quantity_by_item_id: {quantity_by_item_id:?}")
                            })?;
                        Ok::<_, Self::Error>((tenant_id, quantity_by_item_id))
                    })
                    .collect::<Result<_, _>>()
                    .with_context(|| format!("transform items: {items:?}"))?;
                (items, is_order_placed)
            }
        };
        let aggregate = Aggregate::new(
            Id::from_str(&id).with_context(|| format!("parse aggregate id: {id}"))?,
            items,
            is_order_placed,
            version,
        );
        Ok(aggregate)
    }
}
impl<S> TryFrom<AggregateModel> for HashMap<String, AttributeValue, S>
where
    S: std::hash::BuildHasher,
    HashMap<String, AttributeValue, S>: From<serde_dynamo::Item>,
{
    type Error = anyhow::Error;

    fn try_from(value: AggregateModel) -> Result<Self, Self::Error> {
        Ok(serde_dynamo::to_item(value)?)
    }
}

/// 集約テーブルのペイロード
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub(crate) enum AggregatePayload {
    V1 {
        items: HashMap<String, HashMap<String, u32>>,
        is_order_placed: bool,
    },
}

impl From<kernel::command::model::aggregate::Aggregate> for AggregatePayload {
    fn from(value: kernel::command::model::aggregate::Aggregate) -> Self {
        let items: HashMap<String, HashMap<String, u32>> = value
            .items()
            .iter()
            .map(|(tenant_id, quantity_by_item_id)| {
                let quantity_by_item_id: HashMap<String, u32> = quantity_by_item_id
                    .iter()
                    .map(|(item_id, quantity)| (item_id.to_string(), *quantity))
                    .collect();
                (tenant_id.to_string(), quantity_by_item_id)
            })
            .collect();
        Self::V1 {
            items,
            is_order_placed: value.is_order_placed(),
        }
    }
}

/// イベントストアのテーブルモデル
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EventStoreModel {
    id: u64,
    aggregate_id: String,
    payload: EventPayload,
    metadata: std::collections::HashMap<String, String>,
}

impl EventStoreModel {
    #[must_use]
    pub(crate) fn new(id: u64, aggregate_id: String, payload: EventPayload) -> Self {
        Self {
            id,
            aggregate_id,
            payload,
            metadata: std::collections::HashMap::new(),
        }
    }

    #[must_use]
    pub fn aggregate_id(&self) -> &str {
        &self.aggregate_id
    }

    #[must_use]
    pub fn payload(&self) -> &EventPayload {
        &self.payload
    }

    #[must_use]
    pub fn metadata(&self) -> &std::collections::HashMap<String, String> {
        &self.metadata
    }

    #[must_use]
    pub(crate) fn metadata_mut(&mut self) -> &mut std::collections::HashMap<String, String> {
        &mut self.metadata
    }
}

impl<S> TryFrom<EventStoreModel> for HashMap<String, AttributeValue, S>
where
    S: std::hash::BuildHasher,
    HashMap<String, AttributeValue, S>: From<serde_dynamo::Item>,
{
    type Error = anyhow::Error;

    fn try_from(value: EventStoreModel) -> Result<Self, Self::Error> {
        serde_dynamo::to_item(value).with_context(|| "try from EventStoreModel")
    }
}

/// イベントストアのペイロード
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventPayload {
    CreatedV1,
    ItemAddedV1 { tenant_id: String, item_id: String },
    ItemRemovedV1 { tenant_id: String, item_id: String },
    OrderPlacedV1,
}

impl From<kernel::command::event::Event> for EventPayload {
    fn from(value: kernel::command::event::Event) -> Self {
        match value {
            kernel::command::event::Event::Created => EventPayload::CreatedV1,
            kernel::command::event::Event::ItemAdded { tenant_id, item_id } => {
                EventPayload::ItemAddedV1 {
                    tenant_id: tenant_id.to_string(),
                    item_id: item_id.to_string(),
                }
            }
            kernel::command::event::Event::ItemRemoved { tenant_id, item_id } => {
                EventPayload::ItemRemovedV1 {
                    tenant_id: tenant_id.to_string(),
                    item_id: item_id.to_string(),
                }
            }
            kernel::command::event::Event::OrderPlaced => EventPayload::OrderPlacedV1,
        }
    }
}

/// イベントストアの最新のIDを記録するテーブルのモデル
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct EventSequenceModel {
    aggregate_id: String,
    latest_event_id: u64,
}

impl EventSequenceModel {
    #[must_use]
    pub(crate) fn new(aggregate_id: String, latest_event_id: u64) -> Self {
        Self {
            aggregate_id,
            latest_event_id,
        }
    }

    #[must_use]
    pub(crate) fn latest_event_id(&self) -> u64 {
        self.latest_event_id
    }

    pub(crate) fn latest_event_id_attribute_value<T: From<serde_dynamo::AttributeValue>>(
        &self,
    ) -> Result<T, anyhow::Error> {
        Ok(serde_dynamo::to_attribute_value(self.latest_event_id)?)
    }
}

impl<S> TryFrom<EventSequenceModel> for HashMap<String, AttributeValue, S>
where
    S: std::hash::BuildHasher,
    HashMap<String, AttributeValue, S>: From<serde_dynamo::Item>,
{
    type Error = anyhow::Error;

    fn try_from(value: EventSequenceModel) -> Result<Self, Self::Error> {
        serde_dynamo::to_item(value).with_context(|| "try from EventSequenceModel")
    }
}
