use crate::command::WidgetCommand;
use crate::Id;

/// 部品 (Widget) に発生するイベント
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WidgetEvent {
    /// 部品を新しく作成する
    WidgetCreated {
        /// イベントの id
        id: Id<WidgetEvent>,
        /// 部品の名前
        widget_name: String,
        /// 部品の説明
        widget_description: String,
    },
    /// 部品の名前を変更する
    WidgetNameChanged {
        /// イベントの id
        id: Id<WidgetEvent>,
        /// 部品の名前
        widget_name: String,
    },
    /// 部品の説明を変更する
    WidgetDescriptionChanged {
        /// イベントの id
        id: Id<WidgetEvent>,
        /// 部品の名前
        widget_description: String,
    },
}

impl From<WidgetCommand> for Vec<WidgetEvent> {
    fn from(value: WidgetCommand) -> Self {
        let id = Id::generate();
        match value {
            WidgetCommand::CreateWidget {
                widget_name,
                widget_description,
            } => vec![WidgetEvent::WidgetCreated {
                id,
                widget_name,
                widget_description,
            }],
            WidgetCommand::ChangeWidgetName { widget_name } => {
                vec![WidgetEvent::WidgetNameChanged { id, widget_name }]
            }
            WidgetCommand::ChangeWidgetDescription { widget_description } => {
                vec![WidgetEvent::WidgetDescriptionChanged {
                    id,
                    widget_description,
                }]
            }
        }
    }
}
