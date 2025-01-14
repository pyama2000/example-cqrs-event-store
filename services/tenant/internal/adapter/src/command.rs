/// テーブルモデル関連のモジュール
pub(crate) mod model;
/// データの永続性とデータストアとのインタラクションを管理するモジュール
pub mod persistence;
/// リポジトリ関連のモジュール
pub mod repository;

pub(crate) use model::{AggregateModel, EventSequenceModel, EventStoreModel};
#[cfg(test)]
pub(crate) use model::{AggregatePayload, EventPayload, Item};
pub use persistence::dynamodb;
pub use repository::{
    CommandRepository, AGGREGATE_TABLE_NAME, EVENT_SEQUENCE_TABLE_NAME, EVENT_STORE_TABLE_NAME,
};
