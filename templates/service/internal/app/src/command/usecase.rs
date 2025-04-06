use kernel::command::processor::CommandProcessor;

/// ユースケースのインターフェイス
pub trait CommandUseCaseExt {}

/// ユースケースの実態
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandUseCase<P: CommandProcessor> {
    processor: P,
}

impl<P: CommandProcessor> CommandUseCase<P> {
    pub fn new(processor: P) -> Self {
        Self { processor }
    }
}

impl<P> CommandUseCaseExt for CommandUseCase<P> where P: CommandProcessor + Send + Sync + 'static {}
