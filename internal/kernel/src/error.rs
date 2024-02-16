use lib::Error;
use thiserror::Error;

/// コマンドの実行に失敗したことを表す
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ApplyCommandError {
    /// イベントに含まれる部品の名前が不正なフォーマットのときのエラー
    #[error("Invalid name for the widget")]
    InvalidWidgetName,
    /// イベントに含まれる部品の説明が不正なフォーマットのときのエラー
    #[error("Invalid description for the widget")]
    InvalidWidgetDescription,
    /// イベントに含まれる部品の説明が不正なフォーマットのときのエラー
    #[error("Aggregation already created")]
    AggregationAlreadyCreated,
    /// イベントに含まれる部品の説明が不正なフォーマットのときのエラー
    #[error("Cannot update Aggregate version")]
    VersionOverflow,
}

/// イベントから復元時のエラー
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadEventError {
    /// Aggregate 復元時に version が実体と一致しないときのエラー
    #[error("Not match aggregate version")]
    NotMatchVersion,
}

#[derive(Error, Debug)]
pub enum AggregateError {
    /// Aggregate が既に更新さているときのエラー
    #[error("Aggregate is already updated")]
    Conflict,
    /// Aggregate が存在しないときのエラー
    #[error("Aggregate not found")]
    NotFound,
    /// その他のエラー
    #[error(transparent)]
    Unknow(#[from] Error),
}
