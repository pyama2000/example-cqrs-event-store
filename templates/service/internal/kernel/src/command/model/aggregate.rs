use crate::command::command::Command;
use crate::command::error::CommandKernelError;
use crate::command::event::Event;
use crate::id::Id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Aggregate {
    id: Id<Aggregate>,
    /// 集約のバージョン
    version: u128,
}

impl Aggregate {
    #[must_use]
    pub fn new(id: Id<Aggregate>, version: u128) -> Self {
        Self { id, version }
    }

    /// 集約のID
    #[must_use]
    pub fn id(&self) -> &Id<Aggregate> {
        &self.id
    }

    /// 集約のバージョン
    #[must_use]
    pub fn version(&self) -> u128 {
        self.version
    }

    /// 集約にコマンドを実行する
    ///
    /// 集約にコマンドを実行すると、コマンドに応じて集約の状態を変更し、集約の状態を変更したイベントを返す
    ///
    /// # Errors
    ///
    /// コマンド実行時にドメインエラーが発生したら [`CommandKernelError`] を成功状態で返し、例外エラーが発生したら [`anyhow::Error`] を返す
    ///
    /// [`anyhow::Error`]: https://docs.rs/anyhow/latest/anyhow/struct.Error.html
    pub fn apply_command(
        &mut self,
        _command: Command,
    ) -> Result<Result<Vec<Event>, CommandKernelError>, anyhow::Error> {
        todo!()
    }
}
