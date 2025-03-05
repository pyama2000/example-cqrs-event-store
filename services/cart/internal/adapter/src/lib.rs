#![allow(clippy::module_name_repetitions)]

/// コマンド操作関連のモジュール
pub mod command;
/// クエリ操作関連のモジュール
pub mod query;

pub(crate) const AGGREGATE_TABLE_NAME: &str = "cart-aggregate";
pub(crate) const EVENT_SEQUENCE_TABLE_NAME: &str = "cart-event-sequence";
pub(crate) const EVENT_STORE_TABLE_NAME: &str = "cart-event-store";
