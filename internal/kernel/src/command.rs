/// 部品 (Widget) に対するコマンド
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WidgetCommand {
    /// 部品を新しく作成する
    CreateWidget {
        widget_name: String,
        widget_description: String,
    },
    /// 部品の名前を変更する
    ChangeWidgetName { widget_name: String },
    /// 部品の説明を変更する
    ChangeWidgetDescription { widget_description: String },
}
