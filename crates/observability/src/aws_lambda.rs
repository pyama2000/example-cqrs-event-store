#[pin_project::pin_project]
pub struct OpenTelemetryFuture<F, C> {
    #[pin]
    future: Option<F>,
    flush: C,
}

impl<F, C> Future for OpenTelemetryFuture<F, C>
where
    F: Future,
    C: Fn() -> Result<(), crate::Error>,
{
    type Output = F::Output;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let ready = std::task::ready!(
            self.as_mut()
                .project()
                .future
                .as_pin_mut()
                .expect("future polled after completion")
                .poll(cx)
        );
        std::pin::Pin::set(&mut self.as_mut().project().future, None);
        if let Err(e) = (self.project().flush)() {
            tracing::error!("{e:?}");
            panic!("failed to execute flush closure");
        }
        std::task::Poll::Ready(ready)
    }
}

pub struct OpenTelemetryService<S, C> {
    inner: S,
    flush: C,
    coldstart: bool,
    service_version: String,
}

impl<S, C> tower::Service<lambda_runtime::LambdaInvocation> for OpenTelemetryService<S, C>
where
    S: tower::Service<lambda_runtime::LambdaInvocation, Response = ()>,
    C: Fn() -> Result<(), crate::Error> + Clone,
{
    type Response = ();
    type Error = S::Error;
    type Future = OpenTelemetryFuture<tracing::instrument::Instrumented<S::Future>, C>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: lambda_runtime::LambdaInvocation) -> Self::Future {
        use opentelemetry_semantic_conventions::resource::{
            CLOUD_PROVIDER, CLOUD_REGION, FAAS_NAME, FAAS_VERSION,
        };
        use opentelemetry_semantic_conventions::trace::{
            CLOUD_RESOURCE_ID, FAAS_COLDSTART, FAAS_INVOCATION_ID, FAAS_TRIGGER,
        };
        use tracing::Instrument as _;
        use tracing_opentelemetry::OpenTelemetrySpanExt as _;

        let span = tracing::info_span!("", "otel.name" = req.context.env_config.function_name);
        span.set_attribute(CLOUD_PROVIDER, "aws");
        span.set_attribute(
            CLOUD_REGION,
            std::env::var("AWS_REGION")
                .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
                .unwrap_or_default(),
        );
        span.set_attribute(CLOUD_RESOURCE_ID, req.context.invoked_function_arn.clone());
        span.set_attribute(
            FAAS_NAME,
            std::env::var("AWS_LAMBDA_FUNCTION_NAME").unwrap_or_default(),
        );
        span.set_attribute(FAAS_VERSION, self.service_version.clone());
        span.set_attribute(FAAS_TRIGGER, "datasource");
        span.set_attribute(FAAS_INVOCATION_ID, req.context.request_id.clone());
        span.set_attribute(FAAS_COLDSTART, self.coldstart);
        self.coldstart = false;
        let future = {
            let _guard = span.enter();
            self.inner.call(req)
        };
        Self::Future {
            future: Some(future.instrument(span)),
            flush: self.flush.clone(),
        }
    }
}

/// AWS Lambda関数を計装するためのレイヤー
///
/// # Examples
///
/// ```
/// let runtime = lambda_runtime::Runtime(lambda_runtime::service_fn(handler));
/// ```
pub struct OpenTelemetryLayer<C> {
    flush: C,
    service_version: String,
}

impl<C> OpenTelemetryLayer<C>
where
    C: Fn() -> Result<(), crate::Error> + Clone,
{
    pub fn new(flush: C, service_version: String) -> Self {
        Self {
            flush,
            service_version,
        }
    }
}

impl<S, C> tower::Layer<S> for OpenTelemetryLayer<C>
where
    C: Fn() -> Result<(), crate::Error> + Clone,
{
    type Service = OpenTelemetryService<S, C>;

    fn layer(&self, inner: S) -> Self::Service {
        Self::Service {
            inner,
            flush: self.flush.clone(),
            coldstart: true,
            service_version: self.service_version.clone(),
        }
    }
}

/// Lambda Rust ランタイムを起動してイベントのポーリングを開始する。
///
/// # Example
/// ```no_run
/// #[tokio::main]
/// async fn main() -> Result<(), lambda_runtime::Error> {
///     let force_flush = observability::provider::init_providers(
///         env!("CARGO_PKG_NAME"),
///         env!("CARGO_PKG_VERSION"),
///     )
///     .expect("failed to init providers");
///     observability::aws_lambda::run(
///         handler,
///         force_flush,
///         env!("CARGO_PKG_NAME"),
///         Span::current().span().span_context()
///     ).await?;
///     Ok(())
/// }
///
/// async fn handler(event: lambda_runtime::LambdaEvent<serde_json::Value>) -> Result<Value, lambda_runtime::Error> {
///     Ok(event.payload)
/// }
/// ```
///
/// # Errors
///
/// ランタイムの起動や関数で問題が起きるとエラーが発生する
pub async fn run<A, F, R, B, S, D, E, C>(
    handler: F,
    flush: C,
    service_version: String,
) -> Result<(), lambda_runtime::Error>
where
    F: tower::Service<lambda_runtime::LambdaEvent<A>, Response = R>,
    F::Future: Future<Output = Result<R, F::Error>>,
    F::Error: Into<lambda_runtime::Diagnostic> + std::fmt::Debug,
    A: for<'de> serde::Deserialize<'de>,
    R: lambda_runtime::IntoFunctionResponse<B, S>,
    B: serde::Serialize,
    S: futures_core::Stream<Item = Result<D, E>> + Unpin + Send + 'static,
    D: Into<bytes::Bytes> + Send,
    E: Into<lambda_runtime::Error> + Send + std::fmt::Debug,
    C: Fn() -> Result<(), crate::Error> + Clone,
{
    let runtime = lambda_runtime::Runtime::new(handler)
        .layer(OpenTelemetryLayer::new(flush, service_version));
    runtime.run().await
}

pub fn add_link(cx: opentelemetry::trace::SpanContext, span: &tracing::Span) {
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;

    span.add_link(cx);
}
