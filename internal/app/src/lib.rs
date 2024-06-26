use std::fmt::Debug;
use std::future::Future;

use kernel::aggregate::WidgetAggregate;
use kernel::command::WidgetCommand;
use kernel::error::{AggregateError, ApplyCommandError};
use kernel::processor::CommandProcessor;
use lib::Error;
use thiserror::Error;

const MAX_RETRY_COUNT: u32 = 3;

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
    Unknown(#[from] Error),
}

impl From<ApplyCommandError> for WidgetServiceError {
    fn from(value: ApplyCommandError) -> Self {
        match value {
            ApplyCommandError::InvalidWidgetName | ApplyCommandError::InvalidWidgetDescription => {
                WidgetServiceError::InvalidValue
            }
            ApplyCommandError::VersionOverflow | ApplyCommandError::AggregationAlreadyCreated => {
                WidgetServiceError::Unknown(value.into())
            }
        }
    }
}

impl From<AggregateError> for WidgetServiceError {
    fn from(value: AggregateError) -> Self {
        match value {
            AggregateError::Conflict => Self::AggregateConfilict,
            AggregateError::NotFound => Self::AggregateNotFound,
            AggregateError::Unknow(e) => Self::Unknown(e),
        }
    }
}

/// 部品 (Widget) のユースケース処理のインターフェイス
#[cfg_attr(feature = "mockall", mockall::automock)]
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
}

impl<C: CommandProcessor + Debug + Send + Sync + 'static> WidgetService for WidgetServiceImpl<C> {
    #[tracing::instrument(ret, err)]
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
        let command_state = aggregate.apply_command(command)?;
        self.command.create_widget_aggregate(command_state).await?;
        Ok(widget_id)
    }

    #[tracing::instrument(ret, err)]
    async fn change_widget_name(
        &self,
        widget_id: String,
        widget_name: String,
    ) -> Result<(), WidgetServiceError> {
        let mut retry_count = 0;
        loop {
            if retry_count > MAX_RETRY_COUNT {
                return Err(WidgetServiceError::AggregateConfilict);
            }
            let aggregate = self
                .command
                .get_widget_aggregate(widget_id.parse()?)
                .await?;
            let command = WidgetCommand::ChangeWidgetName {
                widget_name: widget_name.clone(),
            };
            let command_state = aggregate.apply_command(command)?;
            match self.command.update_widget_aggregate(command_state).await {
                Ok(_) => break,
                Err(e) => match e {
                    AggregateError::Conflict if retry_count.lt(&MAX_RETRY_COUNT) => {
                        retry_count += 1
                    }
                    _ => return Err(e.into()),
                },
            }
        }
        Ok(())
    }

    #[tracing::instrument(ret, err)]
    async fn change_widget_description(
        &self,
        widget_id: String,
        widget_description: String,
    ) -> Result<(), WidgetServiceError> {
        let mut retry_count = 0;
        loop {
            if retry_count > MAX_RETRY_COUNT {
                return Err(WidgetServiceError::AggregateConfilict);
            }
            let aggregate = self
                .command
                .get_widget_aggregate(widget_id.parse()?)
                .await?;
            let command = WidgetCommand::ChangeWidgetDescription {
                widget_description: widget_description.clone(),
            };
            let command_state = aggregate.apply_command(command)?;
            match self.command.update_widget_aggregate(command_state).await {
                Ok(_) => break,
                Err(e) => match e {
                    AggregateError::Conflict if retry_count.lt(&MAX_RETRY_COUNT) => {
                        retry_count += 1
                    }
                    _ => return Err(e.into()),
                },
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use kernel::aggregate::WidgetAggregate;
    use kernel::error::{AggregateError, ApplyCommandError};
    use kernel::event::WidgetEvent;
    use kernel::processor::{CommandProcessor, MockCommandProcessor};
    use lib::DateTime;

    use crate::{WidgetService, WidgetServiceError, WidgetServiceImpl, MAX_RETRY_COUNT};

    const WIDGET_NAME: &str = "部品名";
    const WIDGET_DESCRIPTION: &str = "部品の説明";

    /// ApplyCommandError から WidgetServiceError に変換するテスト
    #[test]
    fn test_convert_apply_command_error_to_service_error() {
        struct TestCase {
            error: ApplyCommandError,
            assert: fn(error: WidgetServiceError),
        }
        let tests = vec![
            TestCase {
                error: ApplyCommandError::InvalidWidgetName,
                assert: |error| assert!(matches!(error, WidgetServiceError::InvalidValue)),
            },
            TestCase {
                error: ApplyCommandError::InvalidWidgetDescription,
                assert: |error| assert!(matches!(error, WidgetServiceError::InvalidValue)),
            },
            TestCase {
                error: ApplyCommandError::VersionOverflow,
                assert: |error| assert!(matches!(error, WidgetServiceError::Unknown(_))),
            },
            TestCase {
                error: ApplyCommandError::AggregationAlreadyCreated,
                assert: |error| assert!(matches!(error, WidgetServiceError::Unknown(_))),
            },
        ];
        for test in tests {
            (test.assert)(test.error.into());
        }
    }

    /// AggregateError から WidgetServiceError に変換するテスト
    #[test]
    fn test_convert_aggregate_error_to_service_error() {
        struct TestCase {
            error: AggregateError,
            assert: fn(error: WidgetServiceError),
        }
        let tests = vec![
            TestCase {
                error: AggregateError::Conflict,
                assert: |error| assert!(matches!(error, WidgetServiceError::AggregateConfilict)),
            },
            TestCase {
                error: AggregateError::NotFound,
                assert: |error| assert!(matches!(error, WidgetServiceError::AggregateNotFound)),
            },
            TestCase {
                error: AggregateError::Unknow("".into()),
                assert: |error| assert!(matches!(error, WidgetServiceError::Unknown(_))),
            },
        ];
        for test in tests {
            (test.assert)(test.error.into());
        }
    }

    /// 部品を作成するテスト
    #[tokio::test]
    async fn test_create_widget() {
        struct TestCase<T: CommandProcessor> {
            name: &'static str,
            widget_name: String,
            widget_description: String,
            command: T,
            assert: fn(name: &str, result: Result<String, WidgetServiceError>),
        }
        let tests = vec![
            TestCase {
                name: "部品名・部品の説明の形式が正しく永続化時にエラーがない場合、処理に成功する",
                widget_name: WIDGET_NAME.to_string(),
                widget_description: WIDGET_DESCRIPTION.to_string(),
                command: {
                    let mut command = MockCommandProcessor::new();
                    command
                        .expect_create_widget_aggregate()
                        .withf(|x| {
                            matches!(
                                x.events().first().unwrap(),
                                WidgetEvent::WidgetCreated {
                                    widget_name,
                                    widget_description,
                                    ..
                                } if widget_name == WIDGET_NAME
                                    && widget_description == WIDGET_DESCRIPTION
                            )
                        })
                        .returning(|_| Box::pin(async { Ok(()) }));
                    command
                },
                assert: |name, result| {
                    assert!(result.is_ok(), "{name}");
                },
            },
            TestCase {
                name:
                    "部品名・部品の説明の形式が不正な場合、WidgetServiceError::InvalidValue が返る",
                widget_name: String::new(),
                widget_description: String::new(),
                command: {
                    let mut command = MockCommandProcessor::new();
                    command
                        .expect_create_widget_aggregate()
                        .returning(|_| Box::pin(async { Ok(()) }));
                    command
                },
                assert: |name, result| {
                    assert!(
                        result.is_err_and(|e| matches!(e, WidgetServiceError::InvalidValue)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "永続化時にエラーが発生した場合は、WidgetServiceError::Unknow が返る",
                widget_name: WIDGET_NAME.to_string(),
                widget_description: WIDGET_DESCRIPTION.to_string(),
                command: {
                    let mut command = MockCommandProcessor::new();
                    command.expect_create_widget_aggregate().returning(|_| {
                        Box::pin(async { Err(AggregateError::Unknow("unknown".into())) })
                    });
                    command
                },
                assert: |name, result| {
                    assert!(
                        result.is_err_and(|e| matches!(e, WidgetServiceError::Unknown(_))),
                        "{name}"
                    );
                },
            },
        ];
        for test in tests {
            let service = WidgetServiceImpl::new(test.command);
            let result = service
                .create_widget(test.widget_name, test.widget_description)
                .await;
            (test.assert)(test.name, result);
        }
    }

    /// 部品名を変更するテスト
    #[tokio::test]
    async fn test_change_widget_name() {
        struct TestCase<T: CommandProcessor> {
            name: &'static str,
            widget_id: String,
            widget_name: String,
            command: T,
            assert: fn(name: &str, result: Result<(), WidgetServiceError>),
        }
        let tests = vec![
            TestCase {
                name: "部品名の形式が正しく永続化時にエラーがない場合は、処理に成功する",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id(),
                widget_name: "部品名v2".to_string(),
                command: {
                    let mut command = MockCommandProcessor::new();
                    command.expect_get_widget_aggregate().returning(|_| {
                        Box::pin(async {
                            Ok(WidgetAggregate::new(
                                DateTime::DT2023_01_01_00_00_00_00.id().parse().unwrap(),
                            )
                            .set_name(WIDGET_NAME.to_string())
                            .set_description(WIDGET_DESCRIPTION.to_string())
                            .set_version(1))
                        })
                    });
                    command
                        .expect_update_widget_aggregate()
                        .withf(|x| {
                            matches!(
                                x.events().first().unwrap(),
                                WidgetEvent::WidgetNameChanged {
                                    widget_name,
                                    ..
                                } if widget_name == "部品名v2"
                            )
                        })
                        .times(1)
                        .returning(|_| Box::pin(async { Ok(()) }));
                    command
                },
                assert: |name, result| {
                    assert!(result.is_ok(), "{name}");
                },
            },
            TestCase {
                name: "部品名の形式が不正な場合、WidgetServiceError::InvalidValue が返る",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse().unwrap(),
                widget_name: String::new(),
                command: {
                    let mut command = MockCommandProcessor::new();
                    command
                        .expect_get_widget_aggregate()
                        .returning(|_| Box::pin(async { Ok(WidgetAggregate::default()) }));
                    command
                        .expect_update_widget_aggregate()
                        .returning(|_| Box::pin(async { Ok(()) }));
                    command
                },
                assert: |name, result| {
                    assert!(
                        result.is_err_and(|e| matches!(e, WidgetServiceError::InvalidValue)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "永続化時に既に集約が更新されている場合、一定数再試行する",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse().unwrap(),
                widget_name: WIDGET_NAME.to_string(),
                command: {
                    let mut command = MockCommandProcessor::new();
                    command.expect_get_widget_aggregate().returning(|_| {
                        Box::pin(async {
                            Ok(WidgetAggregate::new(
                                DateTime::DT2023_01_01_00_00_00_00.id().parse().unwrap(),
                            )
                            .set_name(WIDGET_NAME.to_string())
                            .set_description(WIDGET_DESCRIPTION.to_string())
                            .set_version(1))
                        })
                    });
                    command
                        .expect_update_widget_aggregate()
                        .times((MAX_RETRY_COUNT + 1) as usize) // NOTE: 最初の1回 + 再試行回数
                        .returning(|_| Box::pin(async { Err(AggregateError::Conflict) }));
                    command
                },
                assert: |name, result| {
                    assert!(
                        result.is_err_and(|e| matches!(e, WidgetServiceError::AggregateConfilict)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "永続化時に不明なエラーが発生した場合は、WidgetServiceError::Unknow が返る",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse().unwrap(),
                widget_name: WIDGET_NAME.to_string(),
                command: {
                    let mut command = MockCommandProcessor::new();
                    command
                        .expect_get_widget_aggregate()
                        .returning(|_| Box::pin(async { Ok(WidgetAggregate::default()) }));
                    command.expect_update_widget_aggregate().returning(|_| {
                        Box::pin(async { Err(AggregateError::Unknow("unknown".into())) })
                    });
                    command
                },
                assert: |name, result| {
                    assert!(
                        result.is_err_and(|e| matches!(e, WidgetServiceError::Unknown(_))),
                        "{name}"
                    );
                },
            },
        ];
        for test in tests {
            let service = WidgetServiceImpl::new(test.command);
            let result = service
                .change_widget_name(test.widget_id, test.widget_name)
                .await;
            (test.assert)(test.name, result);
        }
    }

    /// 部品の説明を変更するテスト
    #[tokio::test]
    async fn test_change_widget_description() {
        struct TestCase<T: CommandProcessor> {
            name: &'static str,
            widget_id: String,
            widget_description: String,
            command: T,
            assert: fn(name: &str, result: Result<(), WidgetServiceError>),
        }
        let tests = vec![
            TestCase {
                name: "部品の説明の形式が正しく永続化時にエラーがない場合は、処理に成功する",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id(),
                widget_description: "部品の説明v2".to_string(),
                command: {
                    let mut command = MockCommandProcessor::new();
                    command.expect_get_widget_aggregate().returning(|_| {
                        Box::pin(async {
                            Ok(WidgetAggregate::new(
                                DateTime::DT2023_01_01_00_00_00_00.id().parse().unwrap(),
                            )
                            .set_name(WIDGET_NAME.to_string())
                            .set_description(WIDGET_DESCRIPTION.to_string())
                            .set_version(1))
                        })
                    });
                    command
                        .expect_update_widget_aggregate()
                        .withf(|x| {
                            matches!(
                                x.events().first().unwrap(),
                                WidgetEvent::WidgetDescriptionChanged {
                                    widget_description,
                                    ..
                                } if widget_description == "部品の説明v2"
                            )
                        })
                        .times(1)
                        .returning(|_| Box::pin(async { Ok(()) }));
                    command
                },
                assert: |name, result| {
                    assert!(result.is_ok(), "{name}");
                },
            },
            TestCase {
                name: "部品の説明の形式が不正な場合、WidgetServiceError::InvalidValue が返る",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse().unwrap(),
                widget_description: String::new(),
                command: {
                    let mut command = MockCommandProcessor::new();
                    command
                        .expect_get_widget_aggregate()
                        .returning(|_| Box::pin(async { Ok(WidgetAggregate::default()) }));
                    command
                        .expect_update_widget_aggregate()
                        .returning(|_| Box::pin(async { Ok(()) }));
                    command
                },
                assert: |name, result| {
                    assert!(
                        result.is_err_and(|e| matches!(e, WidgetServiceError::InvalidValue)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "永続化時に既に集約が更新されている場合、一定数再試行する",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse().unwrap(),
                widget_description: WIDGET_DESCRIPTION.to_string(),
                command: {
                    let mut command = MockCommandProcessor::new();
                    command.expect_get_widget_aggregate().returning(|_| {
                        Box::pin(async {
                            Ok(WidgetAggregate::new(
                                DateTime::DT2023_01_01_00_00_00_00.id().parse().unwrap(),
                            )
                            .set_name(WIDGET_NAME.to_string())
                            .set_description(WIDGET_DESCRIPTION.to_string())
                            .set_version(1))
                        })
                    });
                    command
                        .expect_update_widget_aggregate()
                        .times((MAX_RETRY_COUNT + 1) as usize) // NOTE: 最初の1回 + 再試行回数
                        .returning(|_| Box::pin(async { Err(AggregateError::Conflict) }));
                    command
                },
                assert: |name, result| {
                    assert!(
                        result.is_err_and(|e| matches!(e, WidgetServiceError::AggregateConfilict)),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "永続化時に不明なエラーが発生した場合は、WidgetServiceError::Unknow が返る",
                widget_id: DateTime::DT2023_01_01_00_00_00_00.id().parse().unwrap(),
                widget_description: WIDGET_DESCRIPTION.to_string(),
                command: {
                    let mut command = MockCommandProcessor::new();
                    command
                        .expect_get_widget_aggregate()
                        .returning(|_| Box::pin(async { Ok(WidgetAggregate::default()) }));
                    command.expect_update_widget_aggregate().returning(|_| {
                        Box::pin(async { Err(AggregateError::Unknow("unknown".into())) })
                    });
                    command
                },
                assert: |name, result| {
                    assert!(
                        result.is_err_and(|e| matches!(e, WidgetServiceError::Unknown(_))),
                        "{name}"
                    );
                },
            },
        ];
        for test in tests {
            let service = WidgetServiceImpl::new(test.command);
            let result = service
                .change_widget_description(test.widget_id, test.widget_description)
                .await;
            (test.assert)(test.name, result);
        }
    }
}
