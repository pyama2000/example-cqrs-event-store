/// コマンド操作を行うリポジトリ
#[derive(Debug, Clone)]
pub struct CommandRepository;

impl kernel::command::processor::CommandProcessor for CommandRepository {
    async fn create(
        &self,
        _aggregate: kernel::command::model::aggregate::Aggregate,
        _event: kernel::command::event::Event,
    ) -> Result<Result<(), kernel::command::error::CommandKernelError>, anyhow::Error> {
        todo!()
    }

    async fn get(
        &self,
        _id: kernel::id::Id<kernel::command::model::aggregate::Aggregate>,
    ) -> Result<
        Result<
            Option<kernel::command::model::aggregate::Aggregate>,
            kernel::command::error::CommandKernelError,
        >,
        anyhow::Error,
    > {
        todo!()
    }

    async fn update(
        &self,
        _aggregate: kernel::command::model::aggregate::Aggregate,
        _events: Vec<kernel::command::event::Event>,
    ) -> Result<Result<(), kernel::command::error::CommandKernelError>, anyhow::Error> {
        todo!()
    }
}
