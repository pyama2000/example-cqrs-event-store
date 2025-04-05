use crate::Error;

/// 各種プロバイダーの登録し、滞留しているテレメトリーデータを送信してプロバイダーを安全に終了するクロージャーを返す。
///
/// # Errors
///
/// 各種プロバイダーの登録時に何らかの問題が発生したらエラーが返る。  
/// 返り値のクロージャー内でプロバイダーを終了できなかった場合も同様にエラーが返る。
///
/// # Examples
///
/// ```no_run
/// let shutdown_providers = observability::provider::init_providers().expect("failed to init providers");
///
/// // Some codes
///
/// shutdown_providers().expect("failed to shutdown providers");
/// ```
#[cfg(feature = "provider")]
pub fn init_providers(
    service_name: &'static str,
    service_version: &'static str,
) -> Result<impl Fn() -> Result<(), Error>, Error> {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry::KeyValue;
    use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
    use opentelemetry_sdk::Resource;
    use opentelemetry_semantic_conventions::resource::SERVICE_VERSION;
    use opentelemetry_semantic_conventions::SCHEMA_URL;
    use tracing::level_filters::LevelFilter;
    use tracing_opentelemetry::OpenTelemetryLayer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt as _;
    use tracing_subscriber::EnvFilter;

    let resource = Resource::builder()
        .with_service_name(service_name)
        .with_schema_url(
            [KeyValue::new(SERVICE_VERSION, service_version)],
            SCHEMA_URL,
        )
        .build();

    let tracer = init_tracer(resource.clone())?;
    let meter = init_meter(resource.clone())?;
    let logger = init_logger(resource)?;

    tracing_subscriber::registry()
        .with(OpenTelemetryLayer::new(
            tracer.tracer(env!("CARGO_PKG_NAME")),
        ))
        .with(OpenTelemetryTracingBridge::new(&logger))
        .with(
            tracing_subscriber::fmt::layer()
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .with_target(true),
        )
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
                .add_directive("aws_config=error".parse()?)
                .add_directive("aws_smithy_runtime=error".parse()?),
        )
        .init();

    let shutdown = move || {
        tracer.shutdown()?;
        meter.shutdown()?;
        logger.shutdown()?;
        Ok::<_, Error>(())
    };
    Ok(shutdown)
}

#[allow(clippy::doc_markdown)]
/// テレメトリデータを送信するOpenTelemetry CollectorのgRPCエンドポイント
#[cfg(feature = "provider")]
static OPENTELEMETRY_COLLECTOR_GRPC_ENDPOINT: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| {
        let host = option_env!("OPENTELEMETRY_COLLECTOR_HOST").unwrap_or("0.0.0.0");
        let port = option_env!("OPENTELEMETRY_COLLECTOR_GRPC_PORT").unwrap_or("4317");
        format!("http://{host}:{port}")
    });

#[allow(clippy::doc_markdown)]
/// トレースとスパンを作成するTracerProviderを登録する
#[cfg(feature = "provider")]
fn init_tracer(
    resource: opentelemetry_sdk::Resource,
) -> Result<opentelemetry_sdk::trace::SdkTracerProvider, Error> {
    use opentelemetry_otlp::{SpanExporter, WithExportConfig};
    use opentelemetry_sdk::trace::{RandomIdGenerator, SdkTracerProvider};

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(OPENTELEMETRY_COLLECTOR_GRPC_ENDPOINT.as_str())
        .build()?;
    let provider = SdkTracerProvider::builder()
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();
    opentelemetry::global::set_tracer_provider(provider.clone());
    Ok(provider)
}

#[allow(clippy::doc_markdown)]
/// メーターと計装を作成するMeterProviderを登録する
#[cfg(feature = "provider")]
fn init_meter(
    resource: opentelemetry_sdk::Resource,
) -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, Error> {
    use opentelemetry_otlp::{MetricExporter, WithExportConfig};
    use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};

    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(OPENTELEMETRY_COLLECTOR_GRPC_ENDPOINT.as_str())
        .build()?;
    let reader = PeriodicReader::builder(exporter).build();
    let provider = SdkMeterProvider::builder()
        .with_resource(resource)
        .with_reader(reader)
        .build();
    opentelemetry::global::set_meter_provider(provider.clone());
    Ok(provider)
}

#[allow(clippy::doc_markdown)]
/// ロガーを作成するLoggerProviderを登録する
#[cfg(feature = "provider")]
fn init_logger(
    resource: opentelemetry_sdk::Resource,
) -> Result<opentelemetry_sdk::logs::SdkLoggerProvider, Error> {
    use opentelemetry_otlp::{LogExporter, WithExportConfig};
    use opentelemetry_sdk::logs::SdkLoggerProvider;

    let exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint(OPENTELEMETRY_COLLECTOR_GRPC_ENDPOINT.as_str())
        .build()?;
    let provider = SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();
    Ok(provider)
}
