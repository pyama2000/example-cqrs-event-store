/// 部品 (Widget) に対するコマンド
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WidgetCommand {
    /// 部品を新しく作成する
    CreateWidget,
    /// 部品の名前を変更する
    ChangeWidgetName,
    /// 部品の説明を変更する
    ChangeWidgetDescription,
}
