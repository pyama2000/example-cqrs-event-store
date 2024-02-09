use crate::event::WidgetEvent;
use crate::Id;

/// 部品 (Widget) に対するコマンド
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WidgetCommand {
    /// 部品を新しく作成する
    CreateWidget(WidgetEvent),
    /// 部品の名前を変更する
    ChangeWidgetName(WidgetEvent),
    /// 部品の説明を変更する
    ChangeWidgetDescription(WidgetEvent),
}

impl WidgetCommand {
    /// 部品を新しく作成する
    pub fn create_widget(widget_name: String, widget_description: String) -> Self {
        Self::CreateWidget(WidgetEvent::WidgetCreated {
            id: Id::generate(),
            widget_name,
            widget_description,
        })
    }

    /// 部品の名前を変更する
    pub fn change_widget_name(widget_name: String) -> Self {
        Self::ChangeWidgetName(WidgetEvent::WidgetNameChanged {
            id: Id::generate(),
            widget_name,
        })
    }
}
