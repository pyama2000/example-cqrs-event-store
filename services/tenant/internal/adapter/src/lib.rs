#![allow(clippy::module_name_repetitions)]

/// コマンド操作関連のモジュール
pub mod command;

pub use command::{
    dynamodb, CommandRepository, AGGREGATE_TABLE_NAME, EVENT_SEQUENCE_TABLE_NAME,
    EVENT_STORE_TABLE_NAME,
};
