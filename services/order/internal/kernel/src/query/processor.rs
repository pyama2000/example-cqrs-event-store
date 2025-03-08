use std::future::Future;

use crate::id::Id;

use super::error::QueryKernelError;
use super::model::{Cart, Order, Tenant};

pub trait QueryProcessor {
    /// 注文を取得する
    fn get_by_order_id(
        &self,
        id: Id<Order>,
    ) -> impl Future<Output = Result<Result<Option<Order>, QueryKernelError>, anyhow::Error>> + Send;

    /// 注文を取得する
    fn get_by_cart_id(
        &self,
        id: Id<Cart>,
    ) -> impl Future<Output = Result<Result<Option<Order>, QueryKernelError>, anyhow::Error>> + Send;

    /// テナントに入った注文のID一覧を取得する
    fn list_tenant_received_order_ids(
        &self,
        tenant_id: Id<Tenant>,
    ) -> impl Future<Output = Result<Result<Vec<Id<Order>>, QueryKernelError>, anyhow::Error>> + Send;

    /// 準備完了状態になった注文のID一覧を取得する
    fn list_prepared_order_ids(
        &self,
    ) -> impl Future<Output = Result<Result<Vec<Id<Order>>, QueryKernelError>, anyhow::Error>> + Send;
}
