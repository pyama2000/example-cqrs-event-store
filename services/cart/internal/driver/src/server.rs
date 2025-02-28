use app::command::error::CommandUseCaseError;
use app::command::usecase::CommandUseCaseExt;
use proto::cart::v1::cart_service_server::CartService;
use proto::cart::v1::{
    AddItemRequest, AddItemResponse, CreateRequest, CreateResponse, GetRequest, GetResponse,
    PlaceOrderRequest, PlaceOrderResponse, RemoveItemRequest, RemoveItemResponse,
};
use tokio::signal;
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
impl<C> CartService for Service<C>
where
    C: CommandUseCaseExt + Send + Sync + 'static,
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
                Ok(()) => return Ok(Response::new(PlaceOrderResponse{})),
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

    async fn get(&self, _req: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        todo!()
    }
}

/// サーバーを安全に終了するための仕組み(Graceful shutdown)
async fn _shutdown_signal() {
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
