use thiserror::Error;

/// コマンドの実行に失敗したことを表す
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandError {
    /// Aggregate のバージョンが上限を超えたときのエラー
    #[error("Cannot update Aggregate version")]
    VersionUpdateLimitReached,
    /// コマンドに不正なイベントが含まれるときのエラー
    #[error("Invalid event found")]
    InvalidEvent,
    /// イベントに含まれる部品の名前が不正なフォーマットのときのエラー
    #[error("Invalid name for the widget")]
    InvalidWidgetName,
    /// イベントに含まれる部品の説明が不正なフォーマットのときのエラー
    #[error("Invalid description for the widget")]
    InvalidWidgetDescription,
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AggregateError {
    /// Aggregate が既に更新さているときのエラー
    #[error("Aggregate is already updated")]
    Conflict,
}
