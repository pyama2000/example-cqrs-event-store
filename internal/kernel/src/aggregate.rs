use lib::Result;

use crate::command::WidgetCommand;
use crate::error::CommandError;
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
    pub fn apply_command(self, command: WidgetCommand) -> Result<WidgetCommandState> {
        WidgetCommandExecutor::new(self, command)
            .validate()?
            .execute()
    }

    /// イベントを読み込んで状態を復元する
    pub fn load_events(mut self, events: Vec<WidgetEvent>, version: u64) -> Result<Self> {
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
            // イベントから Aggregate を復元時のバージョンが合わないときのエラー
            return Err("Not match aggregate version".into());
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
struct WidgetCommandExecutor<EventValid, FormatValid> {
    aggregate: WidgetAggregate,
    command: WidgetCommand,
    event_valid: EventValid,
    format_valid: FormatValid,
}

impl WidgetCommandExecutor<(), ()> {
    fn new(aggregate: WidgetAggregate, command: WidgetCommand) -> Self {
        Self {
            aggregate,
            command,
            event_valid: (),
            format_valid: (),
        }
    }

    /// Aggregate に対するコマンドが有効か確認する
    fn validate(self) -> Result<WidgetCommandExecutor<Valid, Valid>> {
        self.validate_events()?.validate_format()
    }
}

impl WidgetCommandExecutor<(), ()> {
    /// コマンドに含まれるイベントが有効か確認する
    fn validate_events(self) -> Result<WidgetCommandExecutor<Valid, ()>> {
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
        if !is_events_valid {
            // コマンドに不正なイベントが含まれるときのエラー
            return Err("Invalid event found".into());
        }
        Ok(WidgetCommandExecutor {
            aggregate: self.aggregate,
            command: self.command,
            event_valid: Valid,
            format_valid: self.format_valid,
        })
    }
}

impl WidgetCommandExecutor<Valid, ()> {
    /// イベントの部品の名前が有効か確認する
    fn validate_format(self) -> Result<WidgetCommandExecutor<Valid, Valid>> {
        let is_widget_name_format_valid = match &self.command {
            WidgetCommand::CreateWidget(event) | WidgetCommand::ChangeWidgetName(event) => {
                match event {
                    WidgetEvent::WidgetCreated { widget_name, .. }
                    | WidgetEvent::WidgetNameChanged { widget_name, .. } => !widget_name.is_empty(),
                    WidgetEvent::WidgetDescriptionChanged { .. } => true,
                }
            }
            WidgetCommand::ChangeWidgetDescription(_) => true,
        };
        if !is_widget_name_format_valid {
            return Err(CommandError::InvalidWidgetName.into());
        }
        let is_widget_description_format_valid = match &self.command {
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
        if !is_widget_description_format_valid {
            return Err(CommandError::InvalidWidgetDescription.into());
        }
        Ok(WidgetCommandExecutor {
            aggregate: self.aggregate,
            command: self.command,
            event_valid: self.event_valid,
            format_valid: Valid,
        })
    }
}

impl WidgetCommandExecutor<Valid, Valid> {
    /// コマンドの実行結果を返す
    fn execute(self) -> Result<WidgetCommandState> {
        let aggregate_version = match self.command {
            WidgetCommand::CreateWidget(_) => 0,
            _ => self
                .aggregate
                .version
                .checked_add(1)
                .ok_or("Cannot update Aggregate version")?,
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
