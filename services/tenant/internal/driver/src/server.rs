use std::net::SocketAddr;

use proto::tenant::v1::tenant_service_server::{TenantService, TenantServiceServer};
use proto::tenant::v1::{
    AddItemsRequest, AddItemsResponse, CreateRequest, CreateResponse, ListItemsRequest,
    ListItemsResponse, ListTenantsRequest, ListTenantsResponse, RemoveItemsRequest,
    RemoveItemsResponse, FILE_DESCRIPTOR_SET,
};
use tokio::signal;
use tonic::{Request, Response, Status};

pub struct Service;

#[tonic::async_trait]
impl TenantService for Service {
    async fn create(&self, _: Request<CreateRequest>) -> Result<Response<CreateResponse>, Status> {
        todo!()
    }

    async fn list_tenants(
        &self,
        _: Request<ListTenantsRequest>,
    ) -> Result<Response<ListTenantsResponse>, Status> {
        todo!()
    }

    async fn add_items(
        &self,
        _: Request<AddItemsRequest>,
    ) -> Result<Response<AddItemsResponse>, Status> {
        todo!()
    }

    async fn remove_items(
        &self,
        _: Request<RemoveItemsRequest>,
    ) -> Result<Response<RemoveItemsResponse>, Status> {
        todo!()
    }

    async fn list_items(
        &self,
        _: Request<ListItemsRequest>,
    ) -> Result<Response<ListItemsResponse>, Status> {
        todo!()
    }
}

pub struct Server {
    service: Service,
}

impl Server {
    #[must_use]
    pub fn new(service: Service) -> Self {
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
