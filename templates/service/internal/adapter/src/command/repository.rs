use kernel::CommandProcessor;

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

    async fn get(&self, _id: ()) -> Result<Option<kernel::Aggregate>, kernel::CommandKernelError> {
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
