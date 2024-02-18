use crate::aggregate::{WidgetAggregate, WidgetCommandState};
use crate::error::AggregateError;
use crate::Id;

/// 集約を永続化する処理のインターフェイス
#[mockall::automock]
pub trait CommandProcessor {
    /// 部品の集約を新しく作成する
    fn create_widget_aggregate(
        &self,
        command_state: WidgetCommandState,
    ) -> impl std::future::Future<Output = Result<(), AggregateError>> + Send;
    /// 部品の集約を取得する
    fn get_widget_aggregate(
        &self,
        widget_id: Id<WidgetAggregate>,
    ) -> impl std::future::Future<Output = Result<WidgetAggregate, AggregateError>> + Send;
    /// 部品の集約を更新する
    fn update_widget_aggregate(
        &self,
        command_state: WidgetCommandState,
    ) -> impl std::future::Future<Output = Result<(), AggregateError>> + Send;
}
