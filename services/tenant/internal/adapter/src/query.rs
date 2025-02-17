/// クエリモデルを定義するモジュール
pub mod model;
/// リポジトリ関連のモジュール
pub mod repository;

pub(crate) use model::{AggregateModel, AggregatePayload};
pub use repository::QueryRepository;
