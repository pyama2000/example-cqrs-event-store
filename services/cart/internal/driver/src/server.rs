use app::command::error::CommandUseCaseError;
use app::command::usecase::CommandUseCaseExt;
use app::query::usecase::QueryUseCaseExt;
use proto::cart::v1::cart_service_server::CartService;
use proto::cart::v1::{
    AddItemRequest, AddItemResponse, CreateRequest, CreateResponse, GetRequest, GetResponse,
    PlaceOrderRequest, PlaceOrderResponse, RemoveItemRequest, RemoveItemResponse,
};
use tonic::{Code, Request, Response, Status};
use tonic_types::{ErrorDetails, StatusExt as _};

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
impl<C, Q> CartService for Service<C, Q>
where
    C: CommandUseCaseExt + Send + Sync + 'static,
    Q: QueryUseCaseExt + Send + Sync + 'static,
{
    async fn create(&self, _: Request<CreateRequest>) -> Result<Response<CreateResponse>, Status> {
        match self.command.create().await {
            Ok(result) => match result {
                Ok(id) => return Ok(Response::new(CreateResponse { id: id.to_string() })),
                Err(e) => return Err(Status::unknown(e.to_string())),
            },
            Err(e) => return Err(Status::unknown(e.to_string())),
        }
    }

    async fn add_item(
        &self,
        req: Request<AddItemRequest>,
    ) -> Result<Response<AddItemResponse>, Status> {
        let AddItemRequest {
            id,
            tenant_id,
            item_id,
        } = req.into_inner();
        let cart_id = id.parse().map_err(|e: anyhow::Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid cart id: {id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("id", e.to_string())
                    .to_owned(),
            )
        })?;
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
        match self.command.add_item(cart_id, tenant_id, item_id).await {
            Ok(result) => match result {
                Ok(()) => return Ok(Response::new(AddItemResponse {})),
                Err(e) => match e {
                    CommandUseCaseError::AggregateNotFound => {
                        return Err(Status::with_error_details(
                            Code::NotFound,
                            format!("cart not found: {id}"),
                            ErrorDetails::new()
                                .add_bad_request_violation("id", e.to_string())
                                .to_owned(),
                        ))
                    }
                },
            },
            Err(e) => return Err(Status::unknown(e.to_string())),
        }
    }

    async fn remove_item(
        &self,
        req: Request<RemoveItemRequest>,
    ) -> Result<Response<RemoveItemResponse>, Status> {
        let RemoveItemRequest {
            id,
            tenant_id,
            item_id,
        } = req.into_inner();
        let cart_id = id.parse().map_err(|e: anyhow::Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid cart id: {id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("id", e.to_string())
                    .to_owned(),
            )
        })?;
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
        match self.command.remove_item(cart_id, tenant_id, item_id).await {
            Ok(result) => match result {
                Ok(()) => return Ok(Response::new(RemoveItemResponse {})),
                Err(e) => match e {
                    CommandUseCaseError::AggregateNotFound => {
                        return Err(Status::with_error_details(
                            Code::NotFound,
                            format!("cart not found: {id}"),
                            ErrorDetails::new()
                                .add_bad_request_violation("id", e.to_string())
                                .to_owned(),
                        ))
                    }
                },
            },
            Err(e) => return Err(Status::unknown(e.to_string())),
        }
    }

    async fn place_order(
        &self,
        req: Request<PlaceOrderRequest>,
    ) -> Result<Response<PlaceOrderResponse>, Status> {
        let PlaceOrderRequest { id } = req.into_inner();
        let cart_id = id.parse().map_err(|e: anyhow::Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid cart id: {id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("id", e.to_string())
                    .to_owned(),
            )
        })?;
        match self.command.place_order(cart_id).await {
            Ok(result) => match result {
                Ok(()) => return Ok(Response::new(PlaceOrderResponse {})),
                Err(e) => match e {
                    CommandUseCaseError::AggregateNotFound => {
                        return Err(Status::with_error_details(
                            Code::NotFound,
                            format!("cart not found: {id}"),
                            ErrorDetails::new()
                                .add_bad_request_violation("id", e.to_string())
                                .to_owned(),
                        ))
                    }
                },
            },
            Err(e) => return Err(Status::unknown(e.to_string())),
        }
    }

    async fn get(&self, req: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        use proto::cart::v1::get_response::Item;

        let GetRequest { id } = req.into_inner();
        let cart_id = id.parse().map_err(|e: anyhow::Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid cart id: {id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("id", e.to_string())
                    .to_owned(),
            )
        })?;
        match self.query.get(cart_id).await {
            Ok(result) => match result {
                Ok(option) => match option {
                    Some(cart) => {
                        let items: Vec<_> = cart
                            .items()
                            .iter()
                            .map(|item| Item {
                                tenant_id: item.tenant_id().to_string(),
                                item_id: item.item_id().to_string(),
                                quantity: item.quantity(),
                            })
                            .collect();
                        return Ok(Response::new(GetResponse {
                            id: cart.id().to_string(),
                            items,
                        }));
                    }
                    None => return Err(Status::not_found(format!("cart not found: {id}"))),
                },
                Err(e) => return Err(Status::unknown(e.to_string())),
            },
            Err(e) => return Err(Status::unknown(e.to_string())),
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
    pub fn new(service: Service<C, Q>) -> Self {
        Self { service }
    }

    /// サーバーを起動する
    ///
    /// # Errors
    pub async fn run(self, addr: std::net::SocketAddr) -> anyhow::Result<()> {
        use anyhow::Context as _;
        use proto::cart::v1::cart_service_server::CartServiceServer;
        use proto::cart::v1::FILE_DESCRIPTOR_SET;
        use tower_http::catch_panic::CatchPanicLayer;

        let cart = CartServiceServer::new(self.service);
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
            .add_service(cart)
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
