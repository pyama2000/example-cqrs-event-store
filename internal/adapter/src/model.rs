use kernel::aggregate::WidgetCommandState;
use serde::Serialize;

/// Aggregate テーブルのモデル
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WidgetAggregateModel {
    widget_id: String,
    last_events: serde_json::Value,
    aggregate_version: u64,
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

#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "version")]
pub(crate) enum WidgetCreatedPayload {
    V1 {
        widget_name: String,
        widget_description: String,
    },
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "version")]
pub(crate) enum WidgetNameChangedPayload {
    V1 { widget_name: String },
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "version")]
pub(crate) enum WidgetDescriptionChangedPayload {
    V1 { widget_description: String },
}
