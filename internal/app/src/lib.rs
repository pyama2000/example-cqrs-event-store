use std::future::Future;

use kernel::aggregate::WidgetAggregate;
use kernel::command::WidgetCommand;
use kernel::error::{AggregateError, ApplyCommandError};
use kernel::processor::CommandProcessor;
use thiserror::Error;

#[derive(Error, Debug)]
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
    #[error("error")]
    Unknow(lib::Error),
}

impl From<ApplyCommandError> for WidgetServiceError {
    fn from(value: ApplyCommandError) -> Self {
        match value {
            ApplyCommandError::InvalidWidgetName | ApplyCommandError::InvalidWidgetDescription => {
                WidgetServiceError::InvalidValue
            }
            ApplyCommandError::VersionOverflow => WidgetServiceError::Unknow(value.into()),
        }
    }
}

impl From<lib::Error> for WidgetServiceError {
    fn from(value: lib::Error) -> Self {
        WidgetServiceError::Unknow(value)
    }
}

/// 部品 (Widget) のユースケース処理のインターフェイス
pub trait WidgetService {
    /// 部品を新しく作成する
    fn create_widget(
        &self,
        widget_name: String,
        widget_description: String,
    ) -> impl Future<Output = Result<String, WidgetServiceError>> + Send;
    /// 部品の名前を変更する
    fn change_widget_name(
        &self,
        widget_id: String,
        widget_name: String,
    ) -> impl Future<Output = Result<(), WidgetServiceError>> + Send;
    /// 部品の説明を変更する
    fn change_widget_description(
        &self,
        widget_id: String,
        widget_description: String,
    ) -> impl Future<Output = Result<(), WidgetServiceError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WidgetServiceImpl<C: CommandProcessor> {
    command: C,
}

impl<C: CommandProcessor> WidgetServiceImpl<C> {
    pub fn new(command: C) -> Self {
        Self { command }
    }

    fn handling_aggregate_error(&self, err: lib::Error) -> Result<(), WidgetServiceError> {
        match err.downcast_ref::<AggregateError>() {
            Some(AggregateError::Conflict) => Ok(()),
            _ => Err(WidgetServiceError::Unknow(err)),
        }
    }
}

impl<C: CommandProcessor + Send + Sync + 'static> WidgetService for WidgetServiceImpl<C> {
    async fn create_widget(
        &self,
        widget_name: String,
        widget_description: String,
    ) -> Result<String, WidgetServiceError> {
        let aggregate = WidgetAggregate::default();
        let widget_id = aggregate.id().to_string();
        let command = WidgetCommand::CreateWidget {
            widget_name,
            widget_description,
        };
        let command_state = aggregate
            .apply_command(command)
            .map_err(WidgetServiceError::from)?;
        self.command.create_widget_aggregate(command_state).await?;
        Ok(widget_id)
    }

    async fn change_widget_name(
        &self,
        widget_id: String,
        widget_name: String,
    ) -> Result<(), WidgetServiceError> {
        const MAX_RETRY_COUNT: u32 = 3;
        let mut retry_count = 0;
        loop {
            if retry_count > MAX_RETRY_COUNT {
                return Err(WidgetServiceError::AggregateConfilict);
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
                .map_err(WidgetServiceError::from)?;
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
    ) -> Result<(), WidgetServiceError> {
        const MAX_RETRY_COUNT: u32 = 3;
        let mut retry_count = 0;
        loop {
            if retry_count > MAX_RETRY_COUNT {
                return Err(WidgetServiceError::AggregateConfilict);
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
                .map_err(WidgetServiceError::from)?;
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
