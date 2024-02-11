use lib::Result;

use crate::command::WidgetCommand;
use crate::error::{AggregateError, CommandError};
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
    pub fn apply_command(self, command: WidgetCommand) -> Result<WidgetCommandState> {
        WidgetCommandExecutor::new(self, command)
            .validate()
            .execute()
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

/// 集約 (Aggregate) に対する `WidgetCommand` が有効か確認して `WidgetCommandState` を作成するビルダー
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct WidgetCommandExecutor<IsEventsValid, IsNameValid, IsDescriptionValid> {
    aggregate: WidgetAggregate,
    command: WidgetCommand,
    is_events_valid: IsEventsValid,
    is_widget_name_valid: IsNameValid,
    is_widget_description_valid: IsDescriptionValid,
}

impl WidgetCommandExecutor<(), (), ()> {
    fn new(aggregate: WidgetAggregate, command: WidgetCommand) -> Self {
        Self {
            aggregate,
            command,
            is_events_valid: (),
            is_widget_name_valid: (),
            is_widget_description_valid: (),
        }
    }

    /// Aggregate に対するコマンドが有効か確認する
    fn validate(self) -> WidgetCommandExecutor<bool, bool, bool> {
        self.validate_events()
            .validate_widget_name()
            .validate_widget_description()
    }
}

impl WidgetCommandExecutor<(), (), ()> {
    /// コマンドに含まれるイベントが有効か確認する
    fn validate_events(self) -> WidgetCommandExecutor<bool, (), ()> {
        let is_events_valid = match &self.command {
            WidgetCommand::CreateWidget(event) => {
                matches!(event, WidgetEvent::WidgetCreated { .. })
            }
            WidgetCommand::ChangeWidgetName(event) => {
                matches!(event, WidgetEvent::WidgetNameChanged { .. })
            }
            WidgetCommand::ChangeWidgetDescription(event) => {
                matches!(event, WidgetEvent::WidgetDescriptionChanged { .. })
            }
        };
        WidgetCommandExecutor {
            aggregate: self.aggregate,
            command: self.command,
            is_events_valid,
            is_widget_name_valid: self.is_widget_name_valid,
            is_widget_description_valid: self.is_widget_description_valid,
        }
    }
}

impl<IsNameValid, IsDescriptionValid> WidgetCommandExecutor<bool, IsNameValid, IsDescriptionValid> {
    /// イベントの部品の名前が有効か確認する
    fn validate_widget_name(self) -> WidgetCommandExecutor<bool, bool, IsDescriptionValid> {
        let is_widget_name_valid = match &self.command {
            WidgetCommand::CreateWidget(event) | WidgetCommand::ChangeWidgetName(event) => {
                match event {
                    WidgetEvent::WidgetCreated { widget_name, .. }
                    | WidgetEvent::WidgetNameChanged { widget_name, .. } => !widget_name.is_empty(),
                    WidgetEvent::WidgetDescriptionChanged { .. } => true,
                }
            }
            WidgetCommand::ChangeWidgetDescription(_) => true,
        };
        WidgetCommandExecutor {
            aggregate: self.aggregate,
            command: self.command,
            is_events_valid: self.is_events_valid,
            is_widget_name_valid,
            is_widget_description_valid: self.is_widget_description_valid,
        }
    }

    /// イベントの部品の説明が有効か確認する
    fn validate_widget_description(self) -> WidgetCommandExecutor<bool, IsNameValid, bool> {
        let is_widget_description_valid = match &self.command {
            WidgetCommand::CreateWidget(event) | WidgetCommand::ChangeWidgetDescription(event) => {
                match event {
                    WidgetEvent::WidgetCreated {
                        widget_description, ..
                    }
                    | WidgetEvent::WidgetDescriptionChanged {
                        widget_description, ..
                    } => !widget_description.is_empty(),
                    WidgetEvent::WidgetNameChanged { .. } => true,
                }
            }
            WidgetCommand::ChangeWidgetName(_) => true,
        };
        WidgetCommandExecutor {
            aggregate: self.aggregate,
            command: self.command,
            is_events_valid: self.is_events_valid,
            is_widget_name_valid: self.is_widget_name_valid,
            is_widget_description_valid,
        }
    }
}

impl WidgetCommandExecutor<bool, bool, bool> {
    /// コマンドの実行結果を返す
    fn execute(self) -> Result<WidgetCommandState> {
        if !self.is_events_valid {
            return Err(Box::new(CommandError::InvalidEvent));
        }
        if !self.is_widget_name_valid {
            return Err(Box::new(CommandError::InvalidWidgetName));
        }
        if !self.is_widget_description_valid {
            return Err(Box::new(CommandError::InvalidWidgetDescription));
        }
        let aggregate_version = match self.command {
            WidgetCommand::CreateWidget(_) => 0,
            _ => self
                .aggregate
                .version
                .checked_add(1)
                .ok_or(Box::new(CommandError::VersionUpdateLimitReached))?,
        };
        let events = match self.command {
            WidgetCommand::CreateWidget(event)
            | WidgetCommand::ChangeWidgetName(event)
            | WidgetCommand::ChangeWidgetDescription(event) => vec![event],
        };
        Ok(WidgetCommandState {
            widget_id: self.aggregate.id,
            events,
            aggregate_version,
        })
    }
}

pub struct WidgetAggregateState {
    aggregate: WidgetAggregate,
    events: Vec<WidgetEvent>,
    widget_id: Id<WidgetAggregate>,
    aggregate_version: u64,
}

impl WidgetAggregateState {
    pub fn new(
        aggregate: WidgetAggregate,
        events: Vec<WidgetEvent>,
        widget_id: Id<WidgetAggregate>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            aggregate,
            events,
            widget_id,
            aggregate_version,
        }
    }

    pub fn restore(mut self) -> Result<WidgetAggregate> {
        self.aggregate.id = self.widget_id;
        for event in &self.events {
            match event {
                WidgetEvent::WidgetCreated {
                    widget_name,
                    widget_description,
                    ..
                } => {
                    self.aggregate.name = widget_name.to_string();
                    self.aggregate.description = widget_description.to_string();
                }
                WidgetEvent::WidgetNameChanged { widget_name, .. } => {
                    self.aggregate.name = widget_name.to_string();
                    self.aggregate.version += 1;
                }
                WidgetEvent::WidgetDescriptionChanged {
                    widget_description, ..
                } => {
                    self.aggregate.description = widget_description.to_string();
                    self.aggregate.version += 1;
                }
            }
        }
        if self.aggregate.version != self.aggregate_version {
            return Err(Box::new(AggregateError::NotMatchVersion));
        }
        Ok(self.aggregate)
    }
}
