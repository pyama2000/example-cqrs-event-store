use kernel::aggregate::WidgetCommandState;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Aggregate テーブルのモデル
#[derive(FromRow, Debug, Clone, PartialEq, Eq)]
pub(crate) struct WidgetAggregateModel {
    widget_id: String,
    last_events: serde_json::Value,
    aggregate_version: u64,
}

impl WidgetAggregateModel {
    pub(crate) fn widget_id(&self) -> &str {
        &self.widget_id
    }

    pub(crate) fn last_events(&self) -> &serde_json::Value {
        &self.last_events
    }

    pub(crate) fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
}

impl TryFrom<WidgetCommandState> for WidgetAggregateModel {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn try_from(value: WidgetCommandState) -> Result<Self, Self::Error> {
        Ok(Self {
            widget_id: value.widget_id().to_string(),
            last_events: serde_json::to_value(
                value
                    .events()
                    .iter()
                    .map(|x| x.clone().into())
                    .collect::<Vec<WidgetEvent>>(),
            )?,
            aggregate_version: value.aggregate_version(),
        })
    }
}

/// Event テーブルのモデル
#[derive(FromRow, Debug, Clone, PartialEq, Eq)]
pub(crate) struct WidgetEventModel {
    event_id: String,
    event_name: String,
    payload: serde_json::Value,
}

impl WidgetEventModel {
    pub(crate) fn event_id(&self) -> &str {
        &self.event_id
    }

    pub(crate) fn event_name(&self) -> &str {
        &self.event_name
    }

    pub(crate) fn payload(&self) -> &serde_json::Value {
        &self.payload
    }
}

/// `WidgetEventModel` の配列を NewType パターンで表現する
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WidgetEventModels(pub(crate) Vec<WidgetEventModel>);

/// `WidgetAggregateModel` の last_events から `WidgetEventModel` の配列に変換する
impl TryFrom<WidgetAggregateModel> for WidgetEventModels {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn try_from(value: WidgetAggregateModel) -> Result<Self, Self::Error> {
        let events: Vec<WidgetEvent> = serde_json::from_value(value.last_events)?;
        let mut models = Vec::new();
        for event in events {
            let model = WidgetEventModel {
                event_id: event.event_id().to_string(),
                event_name: event.event_name(),
                payload: event.to_payload_json_value()?,
            };
            models.push(model);
        }
        Ok(Self(models))
    }
}

#[derive(
    Serialize, Deserialize, strum_macros::Display, Debug, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(tag = "event_name")]
pub(crate) enum WidgetEvent {
    WidgetCreated {
        event_id: String,
        payload: WidgetCreatedPayload,
    },
    WidgetNameChanged {
        event_id: String,
        payload: WidgetNameChangedPayload,
    },
    WidgetDescriptionChanged {
        event_id: String,
        payload: WidgetDescriptionChangedPayload,
    },
}

impl WidgetEvent {
    /// イベントの名前
    fn event_name(&self) -> String {
        self.to_string()
    }

    /// イベントの ID
    fn event_id(&self) -> &str {
        match &self {
            WidgetEvent::WidgetCreated { event_id, .. } => event_id,
            WidgetEvent::WidgetNameChanged { event_id, .. } => event_id,
            WidgetEvent::WidgetDescriptionChanged { event_id, .. } => event_id,
        }
    }

    fn to_payload_json_value(
        &self,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let payload = match &self {
            WidgetEvent::WidgetCreated { payload, .. } => serde_json::to_value(payload)?,
            WidgetEvent::WidgetNameChanged { payload, .. } => serde_json::to_value(payload)?,
            WidgetEvent::WidgetDescriptionChanged { payload, .. } => serde_json::to_value(payload)?,
        };
        Ok(payload)
    }
}

impl From<kernel::event::WidgetEvent> for WidgetEvent {
    fn from(value: kernel::event::WidgetEvent) -> Self {
        match value {
            kernel::event::WidgetEvent::WidgetCreated {
                id,
                widget_name,
                widget_description,
            } => Self::WidgetCreated {
                event_id: id.to_string(),
                payload: WidgetCreatedPayload::V1 {
                    widget_name,
                    widget_description,
                },
            },
            kernel::event::WidgetEvent::WidgetNameChanged { id, widget_name } => {
                Self::WidgetNameChanged {
                    event_id: id.to_string(),
                    payload: WidgetNameChangedPayload::V1 { widget_name },
                }
            }
            kernel::event::WidgetEvent::WidgetDescriptionChanged {
                id,
                widget_description,
            } => Self::WidgetDescriptionChanged {
                event_id: id.to_string(),
                payload: WidgetDescriptionChangedPayload::V1 { widget_description },
            },
        }
    }
}

impl TryFrom<WidgetEventModel> for WidgetEvent {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn try_from(value: WidgetEventModel) -> Result<Self, Self::Error> {
        let value = serde_json::json!({
            "event_id": value.event_id,
            "event_name": value.event_name,
            "payload": value.payload,
        });
        Ok(serde_json::from_value(value)?)
    }
}

impl TryInto<kernel::event::WidgetEvent> for WidgetEvent {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn try_into(self) -> Result<kernel::event::WidgetEvent, Self::Error> {
        let event = match self {
            WidgetEvent::WidgetCreated { event_id, payload } => match payload {
                WidgetCreatedPayload::V1 {
                    widget_name,
                    widget_description,
                } => kernel::event::WidgetEvent::WidgetCreated {
                    id: event_id.parse()?,
                    widget_name,
                    widget_description,
                },
            },
            WidgetEvent::WidgetNameChanged { event_id, payload } => match payload {
                WidgetNameChangedPayload::V1 { widget_name } => {
                    kernel::event::WidgetEvent::WidgetNameChanged {
                        id: event_id.parse()?,
                        widget_name,
                    }
                }
            },
            WidgetEvent::WidgetDescriptionChanged { event_id, payload } => match payload {
                WidgetDescriptionChangedPayload::V1 { widget_description } => {
                    kernel::event::WidgetEvent::WidgetDescriptionChanged {
                        id: event_id.parse()?,
                        widget_description,
                    }
                }
            },
        };
        Ok(event)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "version")]
pub(crate) enum WidgetCreatedPayload {
    V1 {
        widget_name: String,
        widget_description: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "version")]
pub(crate) enum WidgetNameChangedPayload {
    V1 { widget_name: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "version")]
pub(crate) enum WidgetDescriptionChangedPayload {
    V1 { widget_description: String },
}
