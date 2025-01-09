use std::net::SocketAddr;

use app::{CommandUseCaseExt, Item, Tenant};
use proto::tenant::v1::tenant_service_server::{TenantService, TenantServiceServer};
use proto::tenant::v1::{
    AddItemsRequest, AddItemsResponse, CreateRequest, CreateResponse, ListItemsRequest,
    ListItemsResponse, ListTenantsRequest, ListTenantsResponse, RemoveItemsRequest,
    RemoveItemsResponse, FILE_DESCRIPTOR_SET,
};
use tokio::signal;
use tonic::{Code, Request, Response, Status};
use tonic_types::{ErrorDetails, StatusExt};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct Service<C: CommandUseCaseExt> {
    command: C,
}

impl<C: CommandUseCaseExt> Service<C> {
    pub fn new(command: C) -> Self {
        Self { command }
    }
}

#[tonic::async_trait]
impl<C> TenantService for Service<C>
where
    C: CommandUseCaseExt + Send + Sync + 'static,
{
    async fn create(
        &self,
        req: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        let req = req.into_inner();
        self.command
            .create(Tenant::new(req.name))
            .await
            .map(|id| Response::new(CreateResponse { id: id.to_string() }))
            // TODO: エラー内容によって返すステータスコードを変更する / Richer Error Model を使う
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn list_tenants(
        &self,
        _: Request<ListTenantsRequest>,
    ) -> Result<Response<ListTenantsResponse>, Status> {
        todo!()
    }

    async fn add_items(
        &self,
        req: Request<AddItemsRequest>,
    ) -> Result<Response<AddItemsResponse>, Status> {
        let AddItemsRequest { tenant_id, items } = req.into_inner();
        let tenant_id = tenant_id.parse().map_err(|e: Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid tenant id: {tenant_id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("tenant_id", e.to_string())
                    .to_owned(),
            )
        })?;
        let items = items
            .into_iter()
            .map(|x| Item::new(x.name, x.price))
            .collect();
        self.command
            .add_items(tenant_id, items)
            .await
            .map(|ids| {
                Response::new(AddItemsResponse {
                    ids: ids.into_iter().map(|x| x.to_string()).collect(),
                })
            })
            // TODO: エラー内容によって返すステータスコードを変更する / Richer Error Model を使う
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn remove_items(
        &self,
        req: Request<RemoveItemsRequest>,
    ) -> Result<Response<RemoveItemsResponse>, Status> {
        let RemoveItemsRequest {
            tenant_id,
            item_ids,
        } = req.into_inner();
        let tenant_id = tenant_id.parse().map_err(|e: Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid tenant id: {tenant_id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("tenant_id", e.to_string())
                    .to_owned(),
            )
        })?;
        let item_ids = item_ids
            .iter()
            .map(|x| x.parse())
            .collect::<Result<_, _>>()
            .map_err(|e: Error| {
                Status::with_error_details(
                    Code::InvalidArgument,
                    format!("invalid item ids: {}", item_ids.join(",")),
                    ErrorDetails::new()
                        .add_bad_request_violation("item_ids", e.to_string())
                        .to_owned(),
                )
            })?;
        self.command
            .remove_items(tenant_id, item_ids)
            .await
            .map(|()| Response::new(RemoveItemsResponse {}))
            // TODO: エラー内容によって返すステータスコードを変更する / Richer Error Model を使う
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn list_items(
        &self,
        _: Request<ListItemsRequest>,
    ) -> Result<Response<ListItemsResponse>, Status> {
        todo!()
    }
}

pub struct Server<C: CommandUseCaseExt> {
    service: Service<C>,
}

impl<C> Server<C>
where
    C: CommandUseCaseExt + Send + Sync + 'static,
{
    #[must_use]
    pub fn new(service: Service<C>) -> Self {
        Self { service }
    }

    /// サーバーを起動する
    ///
    /// # Errors
    /// サーバー起動時に何らかの問題が発生したらエラーが発生する
    pub async fn run(
        self,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let tenant_service = TenantServiceServer::new(self.service);
        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()?;
        let (_, health_service) = tonic_health::server::health_reporter();
        tonic::transport::Server::builder()
            .add_service(tenant_service)
            .add_service(reflection_service)
            .add_service(health_service)
            .serve_with_shutdown(addr, shutdown_signal())
            .await?;
        Ok(())
    }
}

/// サーバーを安全に終了するための仕組み(Graceful shutdown)
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .unwrap_or_else(|e| panic!("failed to install Ctrl+C handler: {e}"));
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .unwrap_or_else(|e| panic!("failed to install signal handler: {e}"))
            .recv()
            .await;
    };
    tokio::select! {
        () = ctrl_c => tracing::trace!("receive ctrl_c signal"),
        () = terminate => tracing::trace!("receive terminate"),
    }
}
