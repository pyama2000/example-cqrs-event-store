use std::future::Future;

use crate::id::Id;

use super::error::QueryKernelError;
use super::model::Cart;

pub trait QueryProcessor {
    // カートを取得する
    fn get(
        &self,
        id: Id<Cart>,
    ) -> impl Future<Output = Result<Result<Option<Cart>, QueryKernelError>, anyhow::Error>> + Send;
}
