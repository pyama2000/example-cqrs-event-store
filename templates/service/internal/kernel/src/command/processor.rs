use std::future::Future;

use crate::id::Id;

use super::error::CommandKernelError;
use super::event::Event;
use super::model::aggregate::{Aggregate, ApplyCommand};

pub trait CommandProcessor {
    /// 集約とイベントを作成する
    fn create(
        &self,
        aggregate: Aggregate,
        event: Event,
    ) -> impl Future<Output = Result<Result<(), CommandKernelError>, anyhow::Error>> + Send;

    /// 集約を取得する
    fn get<T: ApplyCommand>(
        &self,
        id: Id<Aggregate>,
    ) -> impl Future<Output = Result<Result<Option<T>, CommandKernelError>, anyhow::Error>> + Send;

    /// 集約を更新しイベントを追加する
    fn update(
        &self,
        aggregate: Aggregate,
        events: Vec<Event>,
    ) -> impl Future<Output = Result<Result<(), CommandKernelError>, anyhow::Error>> + Send;
}
