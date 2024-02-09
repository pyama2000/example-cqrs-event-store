use crate::event::WidgetEvent;

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
