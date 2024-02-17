use kernel::aggregate::WidgetCommandState;
use kernel::event::WidgetEvent;
use lib::Error;
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
    type Error = Error;

    fn try_from(value: WidgetCommandState) -> Result<Self, Self::Error> {
        Ok(Self {
            widget_id: value.widget_id().to_string(),
            last_events: serde_json::to_value(
                value
                    .events()
                    .iter()
                    .map(|x| x.clone().into())
                    .collect::<Vec<WidgetEventMapper>>(),
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

impl TryFrom<WidgetAggregateModel> for Vec<WidgetEventModel> {
    type Error = Error;

    fn try_from(value: WidgetAggregateModel) -> Result<Self, Self::Error> {
        let mappers: Vec<WidgetEventMapper> = serde_json::from_value(value.last_events)?;
        let mut models = Vec::new();
        for mapper in mappers {
            let model = WidgetEventModel {
                event_id: mapper.event_id().to_string(),
                event_name: mapper.event_name(),
                payload: mapper.to_payload_json_value()?,
            };
            models.push(model);
        }
        Ok(models)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(
    Serialize, Deserialize, strum_macros::Display, Debug, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(tag = "event_name")]
pub(crate) enum WidgetEventMapper {
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

impl WidgetEventMapper {
    /// イベントの名前
    fn event_name(&self) -> String {
        self.to_string()
    }

    /// イベントの ID
    fn event_id(&self) -> &str {
        match &self {
            WidgetEventMapper::WidgetCreated { event_id, .. } => event_id,
            WidgetEventMapper::WidgetNameChanged { event_id, .. } => event_id,
            WidgetEventMapper::WidgetDescriptionChanged { event_id, .. } => event_id,
        }
    }

    fn to_payload_json_value(&self) -> Result<serde_json::Value, Error> {
        let payload = match &self {
            WidgetEventMapper::WidgetCreated { payload, .. } => serde_json::to_value(payload)?,
            WidgetEventMapper::WidgetNameChanged { payload, .. } => serde_json::to_value(payload)?,
            WidgetEventMapper::WidgetDescriptionChanged { payload, .. } => {
                serde_json::to_value(payload)?
            }
        };
        Ok(payload)
    }
}

impl From<WidgetEvent> for WidgetEventMapper {
    fn from(value: WidgetEvent) -> Self {
        match value {
            WidgetEvent::WidgetCreated {
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
            WidgetEvent::WidgetNameChanged { id, widget_name } => Self::WidgetNameChanged {
                event_id: id.to_string(),
                payload: WidgetNameChangedPayload::V1 { widget_name },
            },
            WidgetEvent::WidgetDescriptionChanged {
                id,
                widget_description,
            } => Self::WidgetDescriptionChanged {
                event_id: id.to_string(),
                payload: WidgetDescriptionChangedPayload::V1 { widget_description },
            },
        }
    }
}

impl TryFrom<WidgetEventModel> for WidgetEventMapper {
    type Error = Error;

    fn try_from(value: WidgetEventModel) -> Result<Self, Self::Error> {
        let value = serde_json::json!({
            "event_id": value.event_id,
            "event_name": value.event_name,
            "payload": value.payload,
        });
        Ok(serde_json::from_value(value)?)
    }
}

impl TryInto<WidgetEvent> for WidgetEventMapper {
    type Error = Error;

    fn try_into(self) -> Result<WidgetEvent, Self::Error> {
        let event = match self {
            WidgetEventMapper::WidgetCreated { event_id, payload } => match payload {
                WidgetCreatedPayload::V1 {
                    widget_name,
                    widget_description,
                } => WidgetEvent::WidgetCreated {
                    id: event_id.parse()?,
                    widget_name,
                    widget_description,
                },
            },
            WidgetEventMapper::WidgetNameChanged { event_id, payload } => match payload {
                WidgetNameChangedPayload::V1 { widget_name } => WidgetEvent::WidgetNameChanged {
                    id: event_id.parse()?,
                    widget_name,
                },
            },
            WidgetEventMapper::WidgetDescriptionChanged { event_id, payload } => match payload {
                WidgetDescriptionChangedPayload::V1 { widget_description } => {
                    WidgetEvent::WidgetDescriptionChanged {
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use kernel::aggregate::WidgetAggregate;
    use lib::Error;
    use ulid::Ulid;

    use crate::model::{
        WidgetCreatedPayload, WidgetDescriptionChangedPayload, WidgetEventMapper,
        WidgetNameChangedPayload,
    };

    use super::{WidgetAggregateModel, WidgetEventModel};

    const WIDGET_NAME: &str = "部品名";
    const WIDGET_DESCRIPTION: &str = "部品説明";
    const EVENT_ID: &str = "01HPS5PP8444SAZ4XPCB407D0R";

    /// CommandState から Aggregate テーブルのモデルに変換するテスト
    #[test]
    fn test_convert_command_state_to_aggregate_table_model() {
        let command_state = WidgetAggregate::default()
            .apply_command(kernel::command::WidgetCommand::CreateWidget {
                widget_name: WIDGET_NAME.to_string(),
                widget_description: WIDGET_DESCRIPTION.to_string(),
            })
            .unwrap();
        let model: Result<WidgetAggregateModel, _> = command_state.try_into();
        assert!(model.is_ok());
    }

    /// Aggregate テーブルのモデルから複数の Event テーブルのモデルに変換するテスト
    #[test]
    fn test_convert_aggregate_table_model_to_event_table_model() {
        struct TestCase {
            name: &'static str,
            aggregate_model: WidgetAggregateModel,
            assert: fn(name: &str, result: Result<Vec<WidgetEventModel>, Error>),
        }
        let tests = vec![
            TestCase {
                name: "部品作成コマンドを実行した集約のAggregate テーブルのモデルから変換する",
                aggregate_model: WidgetAggregateModel {
                    widget_id: String::new(),
                    last_events: serde_json::to_value(vec![WidgetEventMapper::WidgetCreated {
                        event_id: EVENT_ID.to_string(),
                        payload: WidgetCreatedPayload::V1 {
                            widget_name: WIDGET_NAME.to_string(),
                            widget_description: WIDGET_DESCRIPTION.to_string(),
                        },
                    }])
                    .unwrap(),
                    aggregate_version: 0,
                },
                assert: |name: _, result: Result<_, _>| {
                    assert!(result.is_ok(), "{name}");
                    let models = result.unwrap();
                    assert_eq!(models.len(), 1, "{name}");
                    let WidgetEventModel {
                        event_id,
                        event_name,
                        payload,
                    } = models.first().unwrap();
                    assert!(Ulid::from_str(event_id).is_ok(), "{name}");
                    assert_eq!(event_name, "WidgetCreated", "{name}");
                    assert_eq!(
                        payload,
                        &serde_json::json!({
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION
                        }),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "部品名変更コマンドを実行した集約のAggregate テーブルのモデルから変換する",
                aggregate_model: WidgetAggregateModel {
                    widget_id: String::new(),
                    last_events: serde_json::to_value(vec![WidgetEventMapper::WidgetNameChanged {
                        event_id: EVENT_ID.to_string(),
                        payload: WidgetNameChangedPayload::V1 {
                            widget_name: WIDGET_NAME.to_string(),
                        },
                    }])
                    .unwrap(),
                    aggregate_version: 1,
                },
                assert: |name: _, result: Result<_, _>| {
                    assert!(result.is_ok(), "{name}");
                    let models = result.unwrap();
                    assert_eq!(models.len(), 1, "{name}");
                    let WidgetEventModel {
                        event_id,
                        event_name,
                        payload,
                    } = models.first().unwrap();
                    assert!(Ulid::from_str(event_id).is_ok(), "{name}");
                    assert_eq!(event_name, "WidgetNameChanged", "{name}");
                    assert_eq!(
                        payload,
                        &serde_json::json!({"version": "V1", "widget_name": WIDGET_NAME}),
                        "{name}"
                    );
                },
            },
            TestCase {
                name:
                    "部品の説明変更コマンドを実行した集約のAggregate テーブルのモデルから変換する",
                aggregate_model: WidgetAggregateModel {
                    widget_id: String::new(),
                    last_events: serde_json::to_value(vec![
                        WidgetEventMapper::WidgetDescriptionChanged {
                            event_id: EVENT_ID.to_string(),
                            payload: WidgetDescriptionChangedPayload::V1 {
                                widget_description: WIDGET_DESCRIPTION.to_string(),
                            },
                        },
                    ])
                    .unwrap(),
                    aggregate_version: 1,
                },
                assert: |name: _, result: Result<_, _>| {
                    assert!(result.is_ok(), "{name}");
                    let models = result.unwrap();
                    assert_eq!(models.len(), 1, "{name}");
                    let WidgetEventModel {
                        event_id,
                        event_name,
                        payload,
                    } = models.first().unwrap();
                    assert!(Ulid::from_str(event_id).is_ok(), "{name}");
                    assert_eq!(event_name, "WidgetDescriptionChanged", "{name}");
                    assert_eq!(
                        payload,
                        &serde_json::json!({
                            "version": "V1",
                            "widget_description": WIDGET_DESCRIPTION
                        }),
                        "{name}"
                    );
                },
            },
        ];
        for test in tests {
            (test.assert)(test.name, test.aggregate_model.try_into())
        }
    }

    /// Event テーブルのモデルから WidgetEventMapper に変換するテスト
    #[test]
    fn test_convert_event_table_model_to_event_mapper() {
        struct TestCase {
            name: &'static str,
            event_model: WidgetEventModel,
            assert: fn(name: &str, result: Result<WidgetEventMapper, Error>),
        }
        let tests = vec![
            TestCase {
                name: "V1 の部品作成イベントの Event テーブルのモデルから変換する",
                event_model: WidgetEventModel {
                    event_id: EVENT_ID.to_string(),
                    event_name: "WidgetCreated".to_string(),
                    payload: serde_json::to_value(WidgetCreatedPayload::V1 {
                        widget_name: WIDGET_NAME.to_string(),
                        widget_description: WIDGET_DESCRIPTION.to_string(),
                    })
                    .unwrap(),
                },
                assert: |name: _, result: _| {
                    assert!(result.is_ok(), "{name}");
                    let mapper = result.unwrap();
                    assert!(
                        matches!(mapper, WidgetEventMapper::WidgetCreated { .. }),
                        "{name}"
                    );
                    assert_eq!(mapper.event_id(), EVENT_ID, "{name}");
                    assert_eq!(mapper.event_name(), "WidgetCreated", "{name}");
                    assert_eq!(
                        mapper.to_payload_json_value().unwrap(),
                        serde_json::json!({
                            "version": "V1",
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION,
                        }),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "V1 の部品名変更イベントの Event テーブルのモデルから変換する",
                event_model: WidgetEventModel {
                    event_id: EVENT_ID.to_string(),
                    event_name: "WidgetNameChanged".to_string(),
                    payload: serde_json::to_value(WidgetNameChangedPayload::V1 {
                        widget_name: WIDGET_NAME.to_string(),
                    })
                    .unwrap(),
                },
                assert: |name: _, result: _| {
                    assert!(result.is_ok(), "{name}");
                    let mapper = result.unwrap();
                    assert!(
                        matches!(mapper, WidgetEventMapper::WidgetNameChanged { .. }),
                        "{name}"
                    );
                    assert_eq!(mapper.event_id(), EVENT_ID, "{name}");
                    assert_eq!(mapper.event_name(), "WidgetNameChanged", "{name}");
                    assert_eq!(
                        mapper.to_payload_json_value().unwrap(),
                        serde_json::json!({ "version": "V1", "widget_name": WIDGET_NAME }),
                        "{name}",
                    );
                },
            },
            TestCase {
                name: "V1 の部品の説明変更イベントの Event テーブルのモデルから変換する",
                event_model: WidgetEventModel {
                    event_id: EVENT_ID.to_string(),
                    event_name: "WidgetDescriptionChanged".to_string(),
                    payload: serde_json::to_value(WidgetDescriptionChangedPayload::V1 {
                        widget_description: WIDGET_DESCRIPTION.to_string(),
                    })
                    .unwrap(),
                },
                assert: |name: _, result: _| {
                    assert!(result.is_ok(), "{name}");
                    let mapper = result.unwrap();
                    assert!(
                        matches!(mapper, WidgetEventMapper::WidgetDescriptionChanged { .. }),
                        "{name}"
                    );
                    assert_eq!(mapper.event_id(), EVENT_ID, "{name}");
                    assert_eq!(mapper.event_name(), "WidgetDescriptionChanged", "{name}");
                    assert_eq!(
                        mapper.to_payload_json_value().unwrap(),
                        serde_json::json!({
                            "version": "V1",
                            "widget_description": WIDGET_DESCRIPTION,
                        }),
                        "{name}"
                    );
                },
            },
        ];
        for test in tests {
            (test.assert)(test.name, test.event_model.try_into());
        }
    }
}
