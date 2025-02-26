use std::collections::HashMap;

use crate::command::command::Command;
use crate::command::error::CommandKernelError;
use crate::command::event::Event;
use crate::id::Id;

use super::entity::{Item, Tenant};

pub trait ApplyCommand {
    /// 集約にコマンドを実行する
    ///
    /// 集約にコマンドを実行すると、コマンドに応じて集約の状態を変更し、集約の状態を変更したイベントを返す
    ///
    /// # Errors
    ///
    /// コマンド実行時にドメインエラーが発生したら [`CommandKernelError`] を成功状態で返し、例外エラーが発生したら [`anyhow::Error`] を返す
    ///
    /// [`anyhow::Error`]: https://docs.rs/anyhow/latest/anyhow/struct.Error.html
    fn apply_command(
        &mut self,
        command: Command,
    ) -> Result<Result<Vec<Event>, CommandKernelError>, anyhow::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Aggregate {
    id: Id<Aggregate>,
    item_ids_by_tenant_id: HashMap<Id<Tenant>, Vec<Id<Item>>>,
    /// 集約のバージョン
    version: u128,
}

impl Aggregate {
    #[must_use]
    pub fn new(
        id: Id<Aggregate>,
        item_ids_by_tenant_id: HashMap<Id<Tenant>, Vec<Id<Item>>>,
        version: u128,
    ) -> Self {
        Self {
            id,
            item_ids_by_tenant_id,
            version,
        }
    }

    /// 集約のID
    #[must_use]
    pub fn id(&self) -> &Id<Aggregate> {
        &self.id
    }

    #[must_use]
    pub fn item_ids_by_tenant_id(&self) -> &HashMap<Id<Tenant>, Vec<Id<Item>>> {
        &self.item_ids_by_tenant_id
    }

    /// 集約のバージョン
    #[must_use]
    pub fn version(&self) -> u128 {
        self.version
    }
}
