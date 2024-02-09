use std::future::Future;

use kernel::processor::CommandProcessor;
use lib::Result;

/// 部品 (Widget) のユースケース処理のインターフェイス
pub trait WidgetService {
    /// 部品を新しく作成する
    fn create_widget(
        &self,
        widget_name: String,
        widget_description: String,
    ) -> impl Future<Output = Result<()>> + Send;
    /// 部品の名前を変更する
    fn change_widget_name(
        &self,
        widget_id: String,
        widget_name: String,
    ) -> impl Future<Output = Result<()>> + Send;
    /// 部品の説明を変更する
    fn change_widget_description(
        &self,
        widget_id: String,
        widget_description: String,
    ) -> impl Future<Output = Result<()>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WidgetServiceImpl<C: CommandProcessor> {
    command: C,
}

impl<C: CommandProcessor + Send + Sync + 'static> WidgetService for WidgetServiceImpl<C> {
    async fn create_widget(&self, _widget_name: String, _widget_description: String) -> Result<()> {
        todo!()
    }

    async fn change_widget_name(&self, _widget_id: String, _widget_name: String) -> Result<()> {
        todo!()
    }

    async fn change_widget_description(
        &self,
        _widget_id: String,
        _widget_description: String,
    ) -> Result<()> {
        todo!()
    }
}
