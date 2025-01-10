use kernel::{CommandProcessor, Id};

/// コマンド操作を行うリポジトリ
#[derive(Debug, Clone)]
pub struct CommandRepository;

impl CommandProcessor for CommandRepository {
    async fn create(
        &self,
        _aggregate: kernel::Aggregate,
        _events: Vec<kernel::Event>,
    ) -> Result<(), kernel::CommandKernelError> {
        todo!()
    }

    async fn get(
        &self,
        _id: Id<kernel::Aggregate>,
    ) -> Result<Option<kernel::Aggregate>, kernel::CommandKernelError> {
        todo!()
    }

    async fn update(
        &self,
        _aggregate: kernel::Aggregate,
        _events: Vec<kernel::Event>,
    ) -> Result<(), kernel::CommandKernelError> {
        todo!()
    }
}
