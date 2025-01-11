#![allow(clippy::module_name_repetitions)]

/// コマンド操作関連のモジュール
pub mod command;
/// IDに関連するモジュール
pub mod id;

pub use command::{Aggregate, Command, CommandKernelError, CommandProcessor, Event, Item};
pub use id::Id;
