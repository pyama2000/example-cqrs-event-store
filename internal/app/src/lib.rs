use std::future::Future;

use kernel::aggregate::WidgetAggregate;
use kernel::command::WidgetCommand;
use kernel::error::{AggregateError, CommandError};
use kernel::processor::CommandProcessor;
use lib::{Error, Result};
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WidgetServiceError {
    /// Aggregate が存在しないときのエラー
    #[error("Aggregate not found")]
    AggregateNotFound,
    /// Aggregate が既に更新さているときのエラー
    #[error("Aggregate is already updated")]
    AggregateConfilict,
    /// 要求された値が不正なときのエラー
    #[error("Invalid value")]
    InvalidValue,
}

/// 部品 (Widget) のユースケース処理のインターフェイス
pub trait WidgetService {
    /// 部品を新しく作成する
    fn create_widget(
        &self,
        widget_name: String,
        widget_description: String,
    ) -> impl Future<Output = Result<String>> + Send;
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

impl<C: CommandProcessor> WidgetServiceImpl<C> {
    pub fn new(command: C) -> Self {
        Self { command }
    }

    fn handling_command_error(&self, err: Error) -> Error {
        match err.downcast_ref::<CommandError>() {
            Some(e) => match e {
                CommandError::InvalidWidgetName | CommandError::InvalidWidgetDescription => {
                    WidgetServiceError::InvalidValue.into()
                }
            },
            None => err,
        }
    }

    fn handling_aggregate_error(&self, err: Error) -> Result<()> {
        match err.downcast_ref::<AggregateError>() {
            Some(AggregateError::Conflict) => Ok(()),
            _ => Err(err),
        }
    }
}

impl<C: CommandProcessor + Send + Sync + 'static> WidgetService for WidgetServiceImpl<C> {
    async fn create_widget(
        &self,
        widget_name: String,
        widget_description: String,
    ) -> Result<String> {
        let aggregate = WidgetAggregate::default();
        let widget_id = aggregate.id().to_string();
        let command = WidgetCommand::CreateWidget {
            widget_name,
            widget_description,
        };
        let command_state = aggregate
            .apply_command(command)
            .map_err(|e| self.handling_command_error(e))?;
        self.command.create_widget_aggregate(command_state).await?;
        Ok(widget_id)
    }

    async fn change_widget_name(&self, widget_id: String, widget_name: String) -> Result<()> {
        const MAX_RETRY_COUNT: u32 = 3;
        let mut retry_count = 0;
        loop {
            if retry_count > MAX_RETRY_COUNT {
                return Err(WidgetServiceError::AggregateConfilict.into());
            }
            let aggregate = self
                .command
                .get_widget_aggregate(widget_id.parse()?)
                .await?
                .ok_or(WidgetServiceError::AggregateNotFound)?;
            let command = WidgetCommand::ChangeWidgetName {
                widget_name: widget_name.clone(),
            };
            let command_state = aggregate
                .apply_command(command)
                .map_err(|e| self.handling_command_error(e))?;
            let result = self.command.update_widget_aggregate(command_state).await;
            if result.is_ok() {
                break;
            }
            self.handling_aggregate_error(result.err().unwrap())?;
            retry_count += 1;
        }
        Ok(())
    }

    async fn change_widget_description(
        &self,
        widget_id: String,
        widget_description: String,
    ) -> Result<()> {
        const MAX_RETRY_COUNT: u32 = 3;
        let mut retry_count = 0;
        loop {
            if retry_count > MAX_RETRY_COUNT {
                return Err(WidgetServiceError::AggregateConfilict.into());
            }
            let aggregate = self
                .command
                .get_widget_aggregate(widget_id.parse()?)
                .await?
                .ok_or(WidgetServiceError::AggregateNotFound)?;
            let command = WidgetCommand::ChangeWidgetDescription {
                widget_description: widget_description.clone(),
            };
            let command_state = aggregate
                .apply_command(command)
                .map_err(|e| self.handling_command_error(e))?;
            let result = self.command.update_widget_aggregate(command_state).await;
            if result.is_ok() {
                break;
            }
            self.handling_aggregate_error(result.err().unwrap())?;
            retry_count += 1;
        }
        Ok(())
    }
}
