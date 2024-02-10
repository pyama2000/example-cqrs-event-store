use std::future::Future;

use kernel::aggregate::WidgetAggregate;
use kernel::command::WidgetCommand;
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
    async fn create_widget(&self, widget_name: String, widget_description: String) -> Result<()> {
        let aggregate = WidgetAggregate::default();
        let command = WidgetCommand::create_widget(widget_name, widget_description);
        let command_state = aggregate.apply_command(command)?;
        self.command.create_widget_aggregate(command_state).await
    }

    async fn change_widget_name(&self, widget_id: String, widget_name: String) -> Result<()> {
        let aggregate = self
            .command
            .get_widget_aggregate(widget_id.parse()?)
            .await?
            .ok_or("Aggregate not found")?;
        let command = WidgetCommand::change_widget_name(widget_name);
        let command_state = aggregate.apply_command(command)?;
        self.command.update_widget_aggregate(command_state).await
    }

    async fn change_widget_description(
        &self,
        widget_id: String,
        widget_description: String,
    ) -> Result<()> {
        let aggregate = self
            .command
            .get_widget_aggregate(widget_id.parse()?)
            .await?
            .ok_or("Aggregate not found")?;
        let command = WidgetCommand::change_widget_description(widget_description);
        let command_state = aggregate.apply_command(command)?;
        self.command.update_widget_aggregate(command_state).await
    }
}
