use std::net::SocketAddr;

use app::{CommandUseCaseExt, QueryUseCaseExt};
use proto::tenant::v1::tenant_service_server::{TenantService, TenantServiceServer};
use proto::tenant::v1::{
    AddItemsRequest, AddItemsResponse, CreateRequest, CreateResponse, ListItemsRequest,
    ListItemsResponse, ListTenantsRequest, ListTenantsResponse, RemoveItemsRequest,
    RemoveItemsResponse, FILE_DESCRIPTOR_SET,
};
use tonic::{Code, Request, Response, Status};
use tonic_types::{ErrorDetails, StatusExt};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct Service<C: CommandUseCaseExt, Q: QueryUseCaseExt> {
    command: C,
    query: Q,
}

impl<C, Q> Service<C, Q>
where
    C: CommandUseCaseExt,
    Q: QueryUseCaseExt,
{
    pub fn new(command: C, query: Q) -> Self {
        Self { command, query }
    }
}

#[tonic::async_trait]
impl<C, Q> TenantService for Service<C, Q>
where
    C: CommandUseCaseExt + Send + Sync + 'static,
    Q: QueryUseCaseExt + Send + Sync + 'static,
{
    async fn create(
        &self,
        req: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        use app::Tenant;

        let req = req.into_inner();
        let name = req.name;
        self.command
            .create(Tenant::new(name.clone()))
            .await
            .map(|id| Response::new(CreateResponse { id: id.to_string() }))
            .map_err(|e| match e {
                app::CommandUseCaseError::InvalidArgument => Status::with_error_details(
                    Code::InvalidArgument,
                    e.to_string(),
                    ErrorDetails::new()
                        .add_bad_request_violation("name", format!("invalid tenant name: {name}"))
                        .to_owned(),
                ),
                e => Status::unknown(e.to_string()),
            })
    }

    async fn list_tenants(
        &self,
        _: Request<ListTenantsRequest>,
    ) -> Result<Response<ListTenantsResponse>, Status> {
        use proto::tenant::v1::list_tenants_response::Tenant;

        self.query
            .list_tenants()
            .await
            .map(|tenants| {
                let tenants: Vec<_> = tenants
                    .into_iter()
                    .map(|t| Tenant {
                        id: t.id().to_string(),
                        name: t.name().to_string(),
                    })
                    .collect();
                Response::new(ListTenantsResponse { tenants })
            })
            .map_err(|e| Status::unknown(e.to_string()))
    }

    async fn add_items(
        &self,
        req: Request<AddItemsRequest>,
    ) -> Result<Response<AddItemsResponse>, Status> {
        use app::Item;

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
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn list_items(
        &self,
        req: Request<ListItemsRequest>,
    ) -> Result<Response<ListItemsResponse>, Status> {
        use proto::tenant::v1::list_items_response::Item;

        let ListItemsRequest { tenant_id } = req.into_inner();
        let tenant_id = tenant_id.parse().map_err(|e: Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid tenant id: {tenant_id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("tenant_id", e.to_string())
                    .to_owned(),
            )
        })?;
        if let Some(items) = self
            .query
            .list_items(tenant_id)
            .await
            .map_err(|e| Status::unknown(e.to_string()))?
        {
            let items: Vec<_> = items
                .into_iter()
                .map(|i| Item {
                    id: i.id().to_string(),
                    name: i.name().to_string(),
                    price: i.price(),
                })
                .collect();
            Ok(Response::new(ListItemsResponse { items }))
        } else {
            Err(Status::not_found("tenant not found"))
        }
    }
}

pub struct Server<C: CommandUseCaseExt, Q: QueryUseCaseExt> {
    service: Service<C, Q>,
}

impl<C, Q> Server<C, Q>
where
    C: CommandUseCaseExt + Send + Sync + 'static,
    Q: QueryUseCaseExt + Send + Sync + 'static,
{
    #[must_use]
    pub fn new(service: Service<C, Q>) -> Self {
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
    use tokio::signal;

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
