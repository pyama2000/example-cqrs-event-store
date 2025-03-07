use app::command::usecase::CommandUseCaseExt;
use app::query::usecase::QueryUseCaseExt;
use proto::order::v1::order_service_server::OrderService;
use proto::order::v1::{
    CancelRequest, CancelResponse, CreateRequest, CreateResponse, DeliveredRequest,
    DeliveredResponse, GetRequest, GetResponse, Item, ListPreparedOrdersRequest,
    ListPreparedOrdersResponse, ListTenantReceivedOrdersRequest, ListTenantReceivedOrdersResponse,
    PickedUpRequest, PickedUpResponse, PreparedRequest, PreparedResponse,
};
use tonic::{Code, Request, Response, Status};
use tonic_types::{ErrorDetails, StatusExt as _};

pub struct Service<C: CommandUseCaseExt, Q: QueryUseCaseExt> {
    command: C,
    query: Q,
}

impl<C: CommandUseCaseExt, Q: QueryUseCaseExt> Service<C, Q> {
    pub fn new(command: C, query: Q) -> Self {
        Self { command, query }
    }
}

#[tonic::async_trait]
impl<C, Q> OrderService for Service<C, Q>
where
    C: CommandUseCaseExt + Send + Sync + 'static,
    Q: QueryUseCaseExt + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self), err(Debug), ret)]
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
                    .add_bad_request_violation("cart_id", format!("{e:#}"))
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
                            .add_bad_request_violation("tenant_id", format!("{e:#}"))
                            .to_owned(),
                    )
                })?;
                let item_id = item_id.parse().map_err(|e: anyhow::Error| {
                    Status::with_error_details(
                        Code::InvalidArgument,
                        format!("invalid item id: {item_id}"),
                        ErrorDetails::new()
                            .add_bad_request_violation("item_id", format!("{e:#}"))
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
                Err(e) => return Err(Status::unknown(format!("{e:#}"))),
            },
            Err(e) => return Err(Status::unknown(format!("{e:#}"))),
        }
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
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
                    .add_bad_request_violation("id", format!("{e:#}"))
                    .to_owned(),
            )
        })?;
        match self.command.prepared(id).await {
            Ok(result) => match result {
                Ok(()) => return Ok(Response::new(PreparedResponse {})),
                Err(e) => return Err(Status::unknown(format!("{e:#}"))),
            },
            Err(e) => return Err(Status::unknown(format!("{e:#}"))),
        }
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
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
                    .add_bad_request_violation("id", format!("{e:#}"))
                    .to_owned(),
            )
        })?;
        match self.command.picked_up(id).await {
            Ok(result) => match result {
                Ok(()) => return Ok(Response::new(PickedUpResponse {})),
                Err(e) => return Err(Status::unknown(format!("{e:#}"))),
            },
            Err(e) => return Err(Status::unknown(format!("{e:#}"))),
        }
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
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
                    .add_bad_request_violation("id", format!("{e:#}"))
                    .to_owned(),
            )
        })?;
        match self.command.delivered(id).await {
            Ok(result) => match result {
                Ok(()) => return Ok(Response::new(DeliveredResponse {})),
                Err(e) => return Err(Status::unknown(format!("{e:#}"))),
            },
            Err(e) => return Err(Status::unknown(format!("{e:#}"))),
        }
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
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
                    .add_bad_request_violation("id", format!("{e:#}"))
                    .to_owned(),
            )
        })?;
        match self.command.cancel(id).await {
            Ok(result) => match result {
                Ok(()) => return Ok(Response::new(CancelResponse {})),
                Err(e) => return Err(Status::unknown(format!("{e:#}"))),
            },
            Err(e) => return Err(Status::unknown(format!("{e:#}"))),
        }
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
    async fn get(&self, req: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        use proto::order::v1::get_response::OrderStatus;
        use proto::order::v1::Item;

        let GetRequest { id } = req.into_inner();
        let Some(id) = id else {
            return Err(Status::invalid_argument("id must be set"));
        };
        let order = match id {
            proto::order::v1::get_request::Id::OrderId(id) => {
                let id = id.parse().map_err(|e: anyhow::Error| {
                    Status::with_error_details(
                        Code::InvalidArgument,
                        format!("invalid order id: {id}"),
                        ErrorDetails::new()
                            .add_bad_request_violation("order_id", format!("{e:#}"))
                            .to_owned(),
                    )
                })?;
                match self.query.get_by_order_id(id).await {
                    Ok(result) => match result {
                        Ok(option) => match option {
                            Some(order) => order,
                            None => return Err(Status::not_found("order not found")),
                        },
                        Err(e) => return Err(Status::unknown(format!("{e:#}"))),
                    },
                    Err(e) => return Err(Status::unknown(format!("{e:#}"))),
                }
            }
            proto::order::v1::get_request::Id::CartId(id) => {
                let id = id.parse().map_err(|e: anyhow::Error| {
                    Status::with_error_details(
                        Code::InvalidArgument,
                        format!("invalid cart id: {id}"),
                        ErrorDetails::new()
                            .add_bad_request_violation("cart_id", format!("{e:#}"))
                            .to_owned(),
                    )
                })?;
                match self.query.get_by_cart_id(id).await {
                    Ok(result) => match result {
                        Ok(option) => match option {
                            Some(order) => order,
                            None => return Err(Status::not_found("order not found")),
                        },
                        Err(e) => return Err(Status::unknown(format!("{e:#}"))),
                    },
                    Err(e) => return Err(Status::unknown(format!("{e:#}"))),
                }
            }
        };
        let items: Vec<_> = order
            .items()
            .iter()
            .map(|item| Item {
                tenant_id: item.tenant_id().to_string(),
                item_id: item.id().to_string(),
                quantity: item.quantity(),
            })
            .collect();
        let status = match order.status() {
            app::query::model::OrderStatus::Received => OrderStatus::Received,
            app::query::model::OrderStatus::Prepared => OrderStatus::Prepared,
            app::query::model::OrderStatus::OnTheWay => OrderStatus::OnTheWay,
            app::query::model::OrderStatus::Delivered => OrderStatus::Delivered,
            app::query::model::OrderStatus::Canceled => OrderStatus::Cancelled,
        }
        .into();
        return Ok(Response::new(GetResponse {
            id: order.id().to_string(),
            items,
            status,
        }));
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
    async fn list_tenant_received_orders(
        &self,
        req: Request<ListTenantReceivedOrdersRequest>,
    ) -> Result<Response<ListTenantReceivedOrdersResponse>, Status> {
        let ListTenantReceivedOrdersRequest { tenant_id } = req.into_inner();
        let tenant_id = tenant_id.parse().map_err(|e: anyhow::Error| {
            Status::with_error_details(
                Code::InvalidArgument,
                format!("invalid tenant id: {tenant_id}"),
                ErrorDetails::new()
                    .add_bad_request_violation("tenant_id", format!("{e:#}"))
                    .to_owned(),
            )
        })?;
        match self.query.list_tenant_received_order_ids(tenant_id).await {
            Ok(result) => match result {
                Ok(ids) => {
                    return Ok(Response::new(ListTenantReceivedOrdersResponse {
                        ids: ids.into_iter().map(|id| id.to_string()).collect(),
                    }))
                }
                Err(e) => return Err(Status::unknown(format!("{e:#}"))),
            },
            Err(e) => return Err(Status::unknown(format!("{e:#}"))),
        }
    }

    #[tracing::instrument(skip(self), err(Debug), ret)]
    async fn list_prepared_orders(
        &self,
        _: Request<ListPreparedOrdersRequest>,
    ) -> Result<Response<ListPreparedOrdersResponse>, Status> {
        match self.query.list_prepared_order_ids().await {
            Ok(result) => match result {
                Ok(ids) => {
                    return Ok(Response::new(ListPreparedOrdersResponse {
                        ids: ids.into_iter().map(|id| id.to_string()).collect(),
                    }))
                }
                Err(e) => return Err(Status::unknown(format!("{e:#}"))),
            },
            Err(e) => return Err(Status::unknown(format!("{e:#}"))),
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

    /// # Errors
    #[tracing::instrument(skip(self), err(Debug), ret)]
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
            .layer(observability::server::grpc_trace_layer())
            .add_service(order)
            .add_service(refrection)
            .add_service(health)
            .serve_with_shutdown(addr, observability::server::shutdown())
            .await
            .with_context(|| "execute the server")
    }
}
