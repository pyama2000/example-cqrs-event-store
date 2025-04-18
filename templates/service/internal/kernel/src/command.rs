/// 集約を操作するコマンドを定義するモジュール
#[allow(clippy::module_inception)]
pub mod command;
/// エラーを定義したモジュール
pub mod error;
/// コマンドで発生したイベントを定義するモジュール
pub mod event;
/// 集約やエンティティを定義するモジュール
pub mod model;
/// リポジトリ操作のインターフェイスを定義するモジュール
pub mod processor;
