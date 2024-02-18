use crate::command::WidgetCommand;
use crate::error::{ApplyCommandError, LoadEventError};
use crate::event::WidgetEvent;
use crate::Id;

/// 部品 (Widget) の集約
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct WidgetAggregate {
    id: Id<WidgetAggregate>,
    name: String,
    description: String,
    version: u64,
}

impl WidgetAggregate {
    pub fn new(id: Id<WidgetAggregate>) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    /// 部品の id
    pub fn id(&self) -> &Id<WidgetAggregate> {
        &self.id
    }

    /// 部品の名前
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 部品の説明
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 集約のバージョン (= 更新回数)
    pub fn version(&self) -> u64 {
        self.version
    }

    /// 集約にコマンドを実行する
    pub fn apply_command(
        self,
        command: WidgetCommand,
    ) -> Result<WidgetCommandState, ApplyCommandError> {
        WidgetCommandExecutor::new(self, command)
            .validate()?
            .execute()
    }

    /// イベントを読み込んで状態を復元する
    pub fn load_events(
        mut self,
        events: Vec<WidgetEvent>,
        version: u64,
    ) -> Result<Self, LoadEventError> {
        if events.is_empty() {
            return Err(LoadEventError::EventsIsEmpty);
        }

        for event in events {
            match event {
                WidgetEvent::WidgetCreated {
                    widget_name,
                    widget_description,
                    ..
                } => {
                    self.name = widget_name;
                    self.description = widget_description;
                }
                WidgetEvent::WidgetNameChanged { widget_name, .. } => {
                    self.name = widget_name;
                    self.version += 1;
                }
                WidgetEvent::WidgetDescriptionChanged {
                    widget_description, ..
                } => {
                    self.description = widget_description;
                    self.version += 1;
                }
            }
        }
        if self.version != version {
            return Err(LoadEventError::NotMatchVersion);
        }
        Ok(self)
    }
}

/// 集約 (Aggregate) に対するコマンドの処理を成功して保存可能になった状態
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WidgetCommandState {
    widget_id: Id<WidgetAggregate>,
    events: Vec<WidgetEvent>,
    aggregate_version: u64,
}

impl WidgetCommandState {
    /// 部品の id
    pub fn widget_id(&self) -> &Id<WidgetAggregate> {
        &self.widget_id
    }

    /// 部品に対するコマンド
    pub fn events(&self) -> &[WidgetEvent] {
        &self.events
    }

    /// 集約のバージョン
    pub fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Valid;

/// 集約 (Aggregate) に対する `WidgetCommand` が有効か確認して `WidgetCommandState` を作成するビルダー
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct WidgetCommandExecutor<ValidCreation, FormatValid> {
    aggregate: WidgetAggregate,
    command: WidgetCommand,
    valid_creation: ValidCreation,
    format_valid: FormatValid,
}

impl WidgetCommandExecutor<(), ()> {
    fn new(aggregate: WidgetAggregate, command: WidgetCommand) -> Self {
        Self {
            aggregate,
            command,
            valid_creation: (),
            format_valid: (),
        }
    }

    /// Aggregate に対するコマンドが有効か確認する
    fn validate(self) -> Result<WidgetCommandExecutor<Valid, Valid>, ApplyCommandError> {
        self.validate_unique_aggregation_creation()?
            .validate_format()
    }
}

impl WidgetCommandExecutor<(), ()> {
    /// 作成済みの Aggregate に対して再度作成するコマンドが実行できないことを確認する
    fn validate_unique_aggregation_creation(
        self,
    ) -> Result<WidgetCommandExecutor<Valid, ()>, ApplyCommandError> {
        if matches!(self.command, WidgetCommand::CreateWidget { .. })
            && self.aggregate.version() != 0
        {
            return Err(ApplyCommandError::AggregationAlreadyCreated);
        }
        Ok(WidgetCommandExecutor {
            aggregate: self.aggregate,
            command: self.command,
            valid_creation: Valid,
            format_valid: self.format_valid,
        })
    }
}

impl WidgetCommandExecutor<Valid, ()> {
    /// イベントの部品の名前が有効か確認する
    fn validate_format(self) -> Result<WidgetCommandExecutor<Valid, Valid>, ApplyCommandError> {
        let events: Vec<WidgetEvent> = self.command.clone().into();
        for event in &events {
            let is_widget_name_format_valid = match event {
                WidgetEvent::WidgetCreated { widget_name, .. }
                | WidgetEvent::WidgetNameChanged { widget_name, .. } => !widget_name.is_empty(),
                WidgetEvent::WidgetDescriptionChanged { .. } => true,
            };
            if !is_widget_name_format_valid {
                return Err(ApplyCommandError::InvalidWidgetName);
            }
            let is_widget_description_format_valid = match event {
                WidgetEvent::WidgetCreated {
                    widget_description, ..
                }
                | WidgetEvent::WidgetDescriptionChanged {
                    widget_description, ..
                } => !widget_description.is_empty(),
                WidgetEvent::WidgetNameChanged { .. } => true,
            };
            if !is_widget_description_format_valid {
                return Err(ApplyCommandError::InvalidWidgetDescription);
            }
        }
        Ok(WidgetCommandExecutor {
            aggregate: self.aggregate,
            command: self.command,
            valid_creation: self.valid_creation,
            format_valid: Valid,
        })
    }
}

impl WidgetCommandExecutor<Valid, Valid> {
    /// コマンドの実行結果を返す
    fn execute(self) -> Result<WidgetCommandState, ApplyCommandError> {
        let aggregate_version = match self.command {
            WidgetCommand::CreateWidget { .. } => 0,
            _ => self
                .aggregate
                .version
                .checked_add(1)
                .ok_or(ApplyCommandError::VersionOverflow)?,
        };
        let events = self.command.into();
        Ok(WidgetCommandState {
            widget_id: self.aggregate.id,
            events,
            aggregate_version,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::{WidgetAggregate, WidgetCommandState};
    use crate::command::WidgetCommand;
    use crate::error::ApplyCommandError;
    use crate::event::WidgetEvent;
    use crate::Id;

    const WIDGET_NAME: &str = "部品名";
    const WIDGET_DESCRIPTION: &str = "部品説明";

    #[test]
    /// 集約へのコマンド実行のテスト
    fn test_apply_command() {
        struct TestCase {
            name: &'static str,
            aggregate: WidgetAggregate,
            arg: WidgetCommand,
            assert: fn(name: &'static str, result: Result<WidgetCommandState, ApplyCommandError>),
        }
        let tests = vec![
            TestCase {
                name: "部品作成コマンドを実行した場合、イベントが部品作成イベントのみ、かつバージョンが0の CommandState が返る",
                aggregate: WidgetAggregate::default(),
                arg: WidgetCommand::CreateWidget {
                    widget_name: WIDGET_NAME.to_string(),
                    widget_description: WIDGET_DESCRIPTION.to_string(),
                },
                assert: |name, result| {
                    assert!(result.is_ok(), "{name}");
                    let state = result.unwrap();
                    assert_eq!(state.events.len(), 1);
                    assert!(
                        matches!(state.events[0], WidgetEvent::WidgetCreated { .. }),
                        "{name}"
                    );
                    assert_eq!(state.aggregate_version, 0);
                },
            },
            TestCase {
                name: "バージョンが1の集約に部品名変更コマンドを実行した場合、イベントが部品名変更イベントのみ、かつバージョンが2の CommandState が返る",
                aggregate: WidgetAggregate {
                    id: Id::generate(),
                    name: WIDGET_NAME.to_string(),
                    description: WIDGET_DESCRIPTION.to_string(),
                    version: 1,
                },
                arg: WidgetCommand::ChangeWidgetName {
                    widget_name: "変更後の部品名".to_string()
                },
                assert: |name, result| {
                    assert!(result.is_ok(), "{name}");
                    let state = result.unwrap();
                    assert_eq!(state.events.len(), 1, "{name}");
                    assert!(
                        matches!(
                            &state.events[0],
                            WidgetEvent::WidgetNameChanged {
                                widget_name, ..
                            } if widget_name == "変更後の部品名",
                        ),
                        "{name}"
                    );
                    assert_eq!(state.aggregate_version, 2);
                },
            },
            TestCase {
                name: "バージョンが1の集約に部品の説明変更コマンドを実行した場合、イベントが部品の説明変更イベントのみ、かつバージョンが2の CommandState が返る",
                aggregate: WidgetAggregate {
                    id: Id::generate(),
                    name: WIDGET_NAME.to_string(),
                    description: WIDGET_DESCRIPTION.to_string(),
                    version: 1,
                },
                arg: WidgetCommand::ChangeWidgetDescription {
                    widget_description: "変更後の部品の説明".to_string(),
                },
                assert: |name, result| {
                    assert!(result.is_ok(), "{name}");
                    let state = result.unwrap();
                    assert_eq!(state.events.len(), 1, "{name}");
                    assert!(
                        matches!(
                            &state.events[0],
                            WidgetEvent::WidgetDescriptionChanged {
                                widget_description, ..
                            } if widget_description == "変更後の部品の説明",
                        ),
                        "{name}"
                    );
                    assert_eq!(state.aggregate_version, 2);
                },
            },
            TestCase {
                name: "すでに集約が作られている状態で部品作成コマンドを実行した場合、ApplyCommandError::AggregateAlreadyCreated が返る",
                aggregate: WidgetAggregate {
                    id: Id::generate(),
                    name: WIDGET_NAME.to_string(),
                    description: WIDGET_DESCRIPTION.to_string(),
                    version: 1,
                },
                arg: WidgetCommand::CreateWidget {
                    widget_name: WIDGET_NAME.to_string(),
                    widget_description: WIDGET_DESCRIPTION.to_string(),
                },
                assert: |name, result| {
                    assert!(
                        matches!(result, Err(ApplyCommandError::AggregationAlreadyCreated)),
                        "{name}",
                    );
                },
            },
            TestCase {
                name: "部品作成コマンド実行時に部品名が空文字の場合、ApplyCommandError::InvalideWidetName が返る",
                aggregate: WidgetAggregate::default(),
                arg: WidgetCommand::CreateWidget {
                    widget_name: "".to_string(),
                    widget_description: WIDGET_DESCRIPTION.to_string(),
                },
                assert: |name, result| {
                    assert!(
                        matches!(result, Err(ApplyCommandError::InvalidWidgetName)),
                        "{name}",
                    );
                },
            },
            TestCase {
                name: "部品作成コマンド実行時に部品の説明が空文字の場合、ApplyCommandError::InvalideWidetName が返る",
                aggregate: WidgetAggregate::default(),
                arg: WidgetCommand::CreateWidget {
                    widget_name: WIDGET_NAME.to_string(),
                    widget_description: "".to_string(),
                },
                assert: |name, result| {
                    assert!(
                        matches!(result, Err(ApplyCommandError::InvalidWidgetDescription)),
                        "{name}",
                    );
                },
            },
            TestCase {
                name: "部品名変更コマンド実行時に部品名が空文字の場合、ApplyCommandError::InvalideWidetName が返る",
                aggregate: WidgetAggregate {
                    id: Id::generate(),
                    name: WIDGET_NAME.to_string(),
                    description: WIDGET_DESCRIPTION.to_string(),
                    version: 1,
                },
                arg: WidgetCommand::ChangeWidgetName { widget_name: "".to_string() },
                assert: |name, result| {
                    assert!(
                        matches!(result, Err(ApplyCommandError::InvalidWidgetName)),
                        "{name}",
                    );
                },
            },
            TestCase {
                name: "部品の説明変更コマンド実行時に部品の説明が空文字の場合、ApplyCommandError::InvalideWidetName が返る",
                aggregate: WidgetAggregate {
                    id: Id::generate(),
                    name: WIDGET_NAME.to_string(),
                    description: WIDGET_DESCRIPTION.to_string(),
                    version: 1,
                },
                arg: WidgetCommand::ChangeWidgetDescription { widget_description: "".to_string() },
                assert: |name, result| {
                    assert!(
                        matches!(result, Err(ApplyCommandError::InvalidWidgetDescription)),
                        "{name}",
                    );
                },
            },
            TestCase {
                name: "バージョンが最大値の集約に部品名変更コマンドを実行した場合、ApplyCommandError::VersionOverflow が返る",
                aggregate: WidgetAggregate {
                    id: Id::generate(),
                    name: WIDGET_NAME.to_string(),
                    description: WIDGET_DESCRIPTION.to_string(),
                    version: u64::MAX,
                },
                arg: WidgetCommand::ChangeWidgetName {
                    widget_name: WIDGET_NAME.to_string(),
                },
                assert: |name, result| {
                    assert!(
                        matches!(result, Err(ApplyCommandError::VersionOverflow)),
                        "{name}",
                    );
                },
            },
            TestCase {
                name: "バージョンが最大値の集約に部品の説明変更コマンドを実行時した場合、ApplyCommandError::VersionOverflow が返る",
                aggregate: WidgetAggregate {
                    id: Id::generate(),
                    name: WIDGET_NAME.to_string(),
                    description: WIDGET_DESCRIPTION.to_string(),
                    version: u64::MAX,
                },
                arg: WidgetCommand::ChangeWidgetDescription {
                    widget_description: WIDGET_DESCRIPTION.to_string(),
                },
                assert: |name, result| {
                    assert!(
                        matches!(result, Err(ApplyCommandError::VersionOverflow)),
                        "{name}",
                    );
                },
            },
        ];
        for test in tests {
            let result = test.aggregate.apply_command(test.arg);
            (test.assert)(test.name, result);
        }
    }
}
