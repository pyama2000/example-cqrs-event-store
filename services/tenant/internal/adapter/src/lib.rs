#![allow(clippy::module_name_repetitions)]

/// コマンド操作関連のモジュール
pub mod command;
/// クエリ操作関連のモジュール
pub mod query;

pub const EVENT_SEQUENCE_TABLE_NAME: &str = "tenant-event-version";
pub const EVENT_STORE_TABLE_NAME: &str = "tenant-event-store";
pub const AGGREGATE_TABLE_NAME: &str = "tenant-aggregate";

pub use command::{dynamodb, CommandRepository};
pub use query::QueryRepository;
