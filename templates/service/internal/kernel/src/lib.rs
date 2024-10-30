#![allow(clippy::module_name_repetitions)]

/// コマンド操作関連のモジュール
pub mod command;

pub use command::{Aggregate, Command, CommandKernelError, CommandProcessor, Event, EventPayload};
