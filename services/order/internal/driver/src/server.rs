use app::command::usecase::CommandUseCaseExt;
use proto::order::v1::order_service_server::OrderService;
use proto::order::v1::{
    CancelRequest, CancelResponse, CreateRequest, CreateResponse, DeliveredRequest,
    DeliveredResponse, GetRequest, GetResponse, Item, ListPreparedOrdersRequest,
    ListPreparedOrdersResponse, ListTenantReceivedOrdersRequest, ListTenantReceivedOrdersResponse,
    PickedUpRequest, PickedUpResponse, PreparedRequest, PreparedResponse,
};
use tonic::{Code, Request, Response, Status};
use tonic_types::{ErrorDetails, StatusExt as _};

pub struct Service<C: CommandUseCaseExt> {
    command: C,
}

impl<C: CommandUseCaseExt> Service<C> {
    pub fn new(command: C) -> Self {
        Self { command }
    }
}

#[tonic::async_trait]
impl<C> OrderService for Service<C>
where
    C: CommandUseCaseExt + Send + Sync + 'static,
{
    async fn create(
        &self,
        req: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        let CreateRequest { cart_id, items } = req.into_inner();
        let cart_id = cart_id.parse().map_err(|e: anyhow::Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid cart id: {cart_id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("cart_id", e.to_string())
                    .to_owned(),
            )
        })?;
        let items: Vec<_> = items
            .into_iter()
            .map(|item| {
                let Item {
                    tenant_id,
                    item_id,
                    quantity,
                } = item;
                let tenant_id = tenant_id.parse().map_err(|e: anyhow::Error| {
                    Status::with_error_details(
                        Code::InvalidArgument,
                        format!("invalid tenant id: {tenant_id}"),
                        ErrorDetails::new()
                            .add_bad_request_violation("tenant_id", e.to_string())
                            .to_owned(),
                    )
                })?;
                let item_id = item_id.parse().map_err(|e: anyhow::Error| {
                    Status::with_error_details(
                        Code::InvalidArgument,
                        format!("invalid item id: {item_id}"),
                        ErrorDetails::new()
                            .add_bad_request_violation("item_id", e.to_string())
                            .to_owned(),
                    )
                })?;
                Ok::<app::command::model::Item, Status>(app::command::model::Item::new(
                    item_id, tenant_id, quantity,
                ))
            })
            .collect::<Result<_, _>>()?;
        match self.command.create(cart_id, items).await {
            Ok(result) => match result {
                Ok(id) => return Ok(Response::new(CreateResponse { id: id.to_string() })),
                Err(e) => return Err(Status::unknown(e.to_string())),
            },
            Err(e) => return Err(Status::unknown(e.to_string())),
        }
    }

    async fn prepared(
        &self,
        req: Request<PreparedRequest>,
    ) -> Result<Response<PreparedResponse>, Status> {
        let PreparedRequest { id } = req.into_inner();
        let id = id.parse().map_err(|e: anyhow::Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid order id: {id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("id", e.to_string())
                    .to_owned(),
            )
        })?;
        match self.command.prepared(id).await {
            Ok(result) => match result {
                Ok(()) => return Ok(Response::new(PreparedResponse {})),
                Err(e) => return Err(Status::unknown(e.to_string())),
            },
            Err(e) => return Err(Status::unknown(e.to_string())),
        }
    }

    async fn picked_up(
        &self,
        req: Request<PickedUpRequest>,
    ) -> Result<Response<PickedUpResponse>, Status> {
        let PickedUpRequest { id } = req.into_inner();
        let id = id.parse().map_err(|e: anyhow::Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid order id: {id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("id", e.to_string())
                    .to_owned(),
            )
        })?;
        match self.command.picked_up(id).await {
            Ok(result) => match result {
                Ok(()) => return Ok(Response::new(PickedUpResponse {})),
                Err(e) => return Err(Status::unknown(e.to_string())),
            },
            Err(e) => return Err(Status::unknown(e.to_string())),
        }
    }

    async fn delivered(
        &self,
        req: Request<DeliveredRequest>,
    ) -> Result<Response<DeliveredResponse>, Status> {
        let DeliveredRequest { id } = req.into_inner();
        let id = id.parse().map_err(|e: anyhow::Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid order id: {id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("id", e.to_string())
                    .to_owned(),
            )
        })?;
        match self.command.delivered(id).await {
            Ok(result) => match result {
                Ok(()) => return Ok(Response::new(DeliveredResponse {})),
                Err(e) => return Err(Status::unknown(e.to_string())),
            },
            Err(e) => return Err(Status::unknown(e.to_string())),
        }
    }

    async fn cancel(
        &self,
        req: Request<CancelRequest>,
    ) -> Result<Response<CancelResponse>, Status> {
        let CancelRequest { id } = req.into_inner();
        let id = id.parse().map_err(|e: anyhow::Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid order id: {id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("id", e.to_string())
                    .to_owned(),
            )
        })?;
        match self.command.cancel(id).await {
            Ok(result) => match result {
                Ok(()) => return Ok(Response::new(CancelResponse {})),
                Err(e) => return Err(Status::unknown(e.to_string())),
            },
            Err(e) => return Err(Status::unknown(e.to_string())),
        }
    }

    async fn get(&self, _: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        todo!()
    }

    async fn list_tenant_received_orders(
        &self,
        _: Request<ListTenantReceivedOrdersRequest>,
    ) -> Result<Response<ListTenantReceivedOrdersResponse>, Status> {
        todo!()
    }

    async fn list_prepared_orders(
        &self,
        _: Request<ListPreparedOrdersRequest>,
    ) -> Result<Response<ListPreparedOrdersResponse>, Status> {
        todo!()
    }
}

pub struct Server<C: CommandUseCaseExt> {
    service: Service<C>,
}

impl<C: CommandUseCaseExt> Server<C>
where
    C: CommandUseCaseExt + Send + Sync + 'static,
{
    pub fn new(service: Service<C>) -> Self {
        Self { service }
    }

    /// # Errors
    pub async fn run(self, addr: std::net::SocketAddr) -> anyhow::Result<()> {
        use anyhow::Context as _;
        use proto::order::v1::order_service_server::OrderServiceServer;
        use proto::order::v1::FILE_DESCRIPTOR_SET;
        use tower_http::catch_panic::CatchPanicLayer;

        let order = OrderServiceServer::new(self.service);
        let refrection = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .with_context(|| "build reflection service")?;
        let (_, health) = tonic_health::server::health_reporter();
        tonic::transport::Server::builder()
            .layer(CatchPanicLayer::custom(
                |any: Box<dyn std::any::Any + Send>| {
                    let message = if let Some(s) = any.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = any.downcast_ref::<&str>() {
                        (*s).to_string()
                    } else {
                        "unknown panic occured".to_string()
                    };
                    let err = format!("panic: {message}");
                    Status::unknown(err).into_http()
                },
            ))
            .add_service(order)
            .add_service(refrection)
            .add_service(health)
            .serve_with_shutdown(addr, shutdown_signal())
            .await
            .with_context(|| "execute the server")
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
