use crate::command::command::Command;
use crate::command::error::CommandKernelError;
use crate::command::event::Event;
use crate::id::Id;

use super::entity::{Cart, Item, OrderStatus};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Aggregate {
    id: Id<Aggregate>,
    cart_id: Id<Cart>,
    items: Vec<Item>,
    status: OrderStatus,
    /// 集約のバージョン
    version: u64,
}

impl Aggregate {
    #[must_use]
    pub fn new(
        id: Id<Aggregate>,
        cart_id: Id<Cart>,
        items: Vec<Item>,
        status: OrderStatus,
        version: u64,
    ) -> Self {
        Self {
            id,
            cart_id,
            items,
            status,
            version,
        }
    }

    /// 集約のID
    #[must_use]
    pub fn id(&self) -> &Id<Aggregate> {
        &self.id
    }

    #[must_use]
    pub fn cart_id(&self) -> Id<Cart> {
        self.cart_id
    }

    #[must_use]
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    #[must_use]
    pub fn status(&self) -> OrderStatus {
        self.status
    }

    /// 集約のバージョン
    #[must_use]
    pub fn version(&self) -> u64 {
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
    pub fn apply_command(&mut self, _command: Command) -> Result<Vec<Event>, CommandKernelError> {
        todo!()
    }
}
