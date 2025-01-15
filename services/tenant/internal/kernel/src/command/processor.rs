use std::future::Future;

use crate::Id;

use super::{Aggregate, CommandKernelError, Event};

#[cfg_attr(feature = "test", mockall::automock)]
pub trait CommandProcessor {
    /// 集約とイベントを作成する
    fn create(
        &self,
        aggregate: Aggregate,
        event: Event,
    ) -> impl Future<Output = Result<(), CommandKernelError>> + Send;

    /// 集約を取得する
    fn get(
        &self,
        id: Id<Aggregate>,
    ) -> impl Future<Output = Result<Option<Aggregate>, CommandKernelError>> + Send;

    /// 集約を更新しイベントを追加する
    fn update(
        &self,
        aggregate: Aggregate,
        events: Vec<Event>,
    ) -> impl Future<Output = Result<(), CommandKernelError>> + Send;
}
