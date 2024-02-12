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
