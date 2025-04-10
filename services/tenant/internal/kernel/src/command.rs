#[allow(clippy::module_inception)]
/// 集約を操作するコマンドを定義するモジュール
pub mod command;
/// エラーを定義したモジュール
pub mod error;
/// コマンドで発生したイベントを定義するモジュール
pub mod event;
/// 集約やエンティティを定義するモジュール
pub mod model;
/// リポジトリ操作のインターフェイスを定義するモジュール
pub mod processor;

pub use command::Command;
pub use error::CommandKernelError;
pub use event::Event;
pub use model::{Aggregate, Item};
pub use processor::CommandProcessor;
