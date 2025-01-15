#![allow(clippy::module_name_repetitions)]

/// コマンド操作関連のモジュール
pub mod command;
/// IDに関連するモジュール
pub mod id;
/// クエリ操作関連のモジュール
pub mod query;

pub use command::{Aggregate, Command, CommandKernelError, CommandProcessor, Event, Item};
pub use id::Id;
pub use query::QueryProcessor;
