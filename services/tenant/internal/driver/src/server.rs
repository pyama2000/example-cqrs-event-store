use std::fmt::Debug;
use std::net::SocketAddr;

use app::{CommandUseCaseExt, QueryUseCaseExt};
use opentelemetry::trace::TraceContextExt;
use proto::tenant::v1::tenant_service_server::{TenantService, TenantServiceServer};
use proto::tenant::v1::{
    AddItemsRequest, AddItemsResponse, CreateRequest, CreateResponse, ListItemsRequest,
    ListItemsResponse, ListTenantsRequest, ListTenantsResponse, RemoveItemsRequest,
    RemoveItemsResponse, FILE_DESCRIPTOR_SET,
};
use tonic::{Code, Request, Response, Status};
use tonic_types::{ErrorDetails, StatusExt};
use tracing::instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
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
    C: CommandUseCaseExt + Send + Sync + 'static + Debug,
    Q: QueryUseCaseExt + Send + Sync + 'static + Debug,
{
    #[instrument(skip(self), err, ret)]
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

    #[instrument(skip(self), err, ret)]
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

    #[instrument(skip(self), err, ret)]
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

    #[instrument(skip(self), err, ret)]
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

    #[instrument(skip(self), err, ret)]
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
    C: CommandUseCaseExt + Send + Sync + 'static + Debug,
    Q: QueryUseCaseExt + Send + Sync + 'static + Debug,
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
        use std::time::Duration;

        use tower_http::catch_panic::CatchPanicLayer;
        use tower_http::trace::TraceLayer;

        let tenant_service = TenantServiceServer::new(self.service);
        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()?;
        let (_, health_service) = tonic_health::server::health_reporter();
        tonic::transport::Server::builder()
            .timeout(Duration::from_millis(500))
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
            .layer(
                TraceLayer::new_for_grpc()
                    .make_span_with(MakeSpan)
                    .on_request(OnRequest)
                    .on_response(OnResponse)
                    .on_failure(OnFailure),
            )
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

#[derive(Clone)]
struct MakeSpan;

impl<B> tower_http::trace::MakeSpan<B> for MakeSpan {
    fn make_span(&mut self, req: &http::Request<B>) -> tracing::Span {
        use tracing_opentelemetry::OpenTelemetrySpanExt as _;

        if req.uri().path().contains("ServerReflectionInfo") {
            // NOTE: リフレクションのトレースは出力しない
            return tracing::Span::none();
        }
        let span = tracing::info_span!("", otel.name = %req.uri().path()[1..]);

        let ctx = opentelemetry::global::get_text_map_propagator(|p| {
            p.extract(&opentelemetry_http::HeaderExtractor(req.headers()))
        });
        if ctx.span().span_context().is_valid() {
            span.set_parent(ctx);
        }

        span
    }
}

#[derive(Clone)]
struct OnRequest;

impl<B> tower_http::trace::OnRequest<B> for OnRequest {
    fn on_request(&mut self, req: &http::Request<B>, span: &tracing::Span) {
        use opentelemetry_semantic_conventions::trace::{
            NETWORK_PEER_ADDRESS, NETWORK_PEER_PORT, RPC_GRPC_REQUEST_METADATA, RPC_METHOD,
            RPC_SERVICE, RPC_SYSTEM,
        };
        use tracing_opentelemetry::OpenTelemetrySpanExt as _;

        span.set_attribute(RPC_SYSTEM, "grpc");
        let uri = req.uri().clone();
        {
            let v: Vec<_> = uri.path()[1..].split('/').collect();
            if v.len() == 2 {
                if let Some(service) = v.first() {
                    span.set_attribute(RPC_SERVICE, (*service).to_string());
                }
                if let Some(method) = v.get(1) {
                    span.set_attribute(RPC_METHOD, (*method).to_string());
                }
            }
        }
        if let Some(host) = uri.host() {
            span.set_attribute(NETWORK_PEER_ADDRESS, (*host).to_string());
        }
        if let Some(port) = uri.port() {
            span.set_attribute(NETWORK_PEER_PORT, port.to_string());
        }
        for (key, value) in req.headers().clone() {
            let (key, value) = match (key, value.to_str()) {
                (Some(key), Ok(value)) => (key, value.to_string()),
                _ => continue,
            };
            span.set_attribute(format!("{RPC_GRPC_REQUEST_METADATA}.{key}"), value);
        }
    }
}

#[derive(Clone)]
struct OnResponse;

impl<B> tower_http::trace::OnResponse<B> for OnResponse {
    fn on_response(self, res: &http::Response<B>, _: std::time::Duration, span: &tracing::Span) {
        use opentelemetry_semantic_conventions::trace::{
            RPC_GRPC_RESPONSE_METADATA, RPC_GRPC_STATUS_CODE,
        };
        use tracing_opentelemetry::OpenTelemetrySpanExt as _;

        let headers = res.headers().clone();
        for (key, value) in headers {
            let (key, value) = match (key, value.to_str()) {
                (Some(key), Ok(value)) => (key, value.to_string()),
                _ => continue,
            };
            span.set_attribute(format!("{RPC_GRPC_RESPONSE_METADATA}.{key}"), value.clone());
        }
        match tonic::Status::from_header_map(res.headers()) {
            Some(status) if status.code() != tonic::Code::Ok => {
                span.set_attribute(RPC_GRPC_STATUS_CODE, i64::from(i32::from(status.code())));
            }
            _ => span.set_attribute(RPC_GRPC_STATUS_CODE, 0),
        }
    }
}

#[derive(Clone)]
struct OnFailure;

impl<F: std::any::Any> tower_http::trace::OnFailure<F> for OnFailure {
    fn on_failure(
        &mut self,
        failure_classification: F,
        _: std::time::Duration,
        span: &tracing::Span,
    ) {
        if let Some(value) = (&failure_classification as &dyn std::any::Any)
            .downcast_ref::<tower_http::classify::GrpcFailureClass>()
        {
            match value {
                tower_http::classify::GrpcFailureClass::Code(code) => {
                    match tonic::Code::from_i32(code.get()) {
                        Code::Ok
                        | Code::Cancelled
                        | Code::InvalidArgument
                        | Code::NotFound
                        | Code::AlreadyExists
                        | Code::PermissionDenied
                        | Code::ResourceExhausted
                        | Code::FailedPrecondition
                        | Code::Aborted
                        | Code::OutOfRange
                        | Code::Unauthenticated => {
                            span.set_status(opentelemetry::trace::Status::Unset);
                        }
                        _ => (),
                    }
                }
                tower_http::classify::GrpcFailureClass::Error(_) => (),
            }
        }
    }
}
