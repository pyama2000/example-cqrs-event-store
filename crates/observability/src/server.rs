/// サーバーを安全に終了するために特定のシグナルを受信したか監視する
///
/// # Panics
///
/// 特定のシグナルを受信する準備ができなかったらパニックを起こす
///
/// # Examples
///
/// ```no_run
/// let (_, health_service) = tonic_health::server::health_reporter();
/// tonic::transport::Server::builder()
///     .add_service(health_service)
///     .serve_with_shutdown("[::1]:50051", observability::server::shutdown())
///     .await
///     .unwrap();
/// ```
#[cfg(feature = "server")]
pub async fn shutdown() {
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

/// gRPCサーバーをトレースするためのミドルウェア
///
/// # Examples
///
/// ```no_run
/// let (_, health_service) = tonic_health::server::health_reporter();
/// tonic::transport::Server::builder()
///     .layer(observability::server::grpc_trace_layer())
///     .add_service(health_service)
///     .serve("[::1]:50051")
///     .await
///     .unwrap();
/// ```
#[cfg(feature = "server")]
#[must_use]
pub fn grpc_trace_layer() -> tower_http::trace::TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::GrpcErrorsAsFailures>,
    MakeSpan,
    OnRequest,
    OnResponse,
    tower_http::trace::DefaultOnBodyChunk,
    tower_http::trace::DefaultOnEos,
    OnFailure,
> {
    tower_http::trace::TraceLayer::new_for_grpc()
        .make_span_with(MakeSpan)
        .on_request(OnRequest)
        .on_response(OnResponse)
        .on_failure(OnFailure)
}

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct MakeSpan;

#[cfg(feature = "server")]
impl<B> tower_http::trace::MakeSpan<B> for MakeSpan {
    fn make_span(&mut self, req: &http::Request<B>) -> tracing::Span {
        use opentelemetry::trace::TraceContextExt as _;
        use tracing_opentelemetry::OpenTelemetrySpanExt as _;

        // NOTE: リフレクションのトレースは出力しない
        if req.uri().path().contains("ServerReflectionInfo") {
            return tracing::Span::none();
        }

        let span = tracing::info_span!("", otel.name = %req.uri().path()[1..]);
        let parent = opentelemetry::global::get_text_map_propagator(|p| {
            p.extract(&opentelemetry_http::HeaderExtractor(req.headers()))
        });
        if parent.span().span_context().is_valid() {
            span.set_parent(parent);
        }

        span
    }
}

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct OnRequest;

#[cfg(feature = "server")]
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

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct OnResponse;

#[cfg(feature = "server")]
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

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct OnFailure;

#[cfg(feature = "server")]
impl<F: std::any::Any> tower_http::trace::OnFailure<F> for OnFailure {
    fn on_failure(
        &mut self,
        failure_classification: F,
        _: std::time::Duration,
        span: &tracing::Span,
    ) {
        use tonic::Code;
        use tracing_opentelemetry::OpenTelemetrySpanExt as _;

        if let Some(value) = (&failure_classification as &dyn std::any::Any)
            .downcast_ref::<tower_http::classify::GrpcFailureClass>()
        {
            match value {
                tower_http::classify::GrpcFailureClass::Code(code) => {
                    match Code::from_i32(code.get()) {
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
