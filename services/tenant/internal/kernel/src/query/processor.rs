use std::future::Future;

use crate::{Aggregate, Id};

use super::{Item, Tenant};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub trait QueryProcessor {
    /// テナントの一覧を取得する
    fn list_tenants(&self) -> impl Future<Output = Result<Vec<Tenant>, Error>> + Send;

    /// テナント商品の一覧を取得する
    fn list_items(
        &self,
        tenant_id: Id<Aggregate>,
    ) -> impl Future<Output = Result<Option<Vec<Item>>, Error>> + Send;
}
