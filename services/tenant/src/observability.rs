type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

static RESOURCE: std::sync::LazyLock<opentelemetry_sdk::Resource> =
    std::sync::LazyLock::new(|| {
        use opentelemetry::KeyValue;
        use opentelemetry_sdk::Resource;
        use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
        use opentelemetry_semantic_conventions::SCHEMA_URL;

        Resource::from_schema_url(
            [
                KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
                KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            ],
            SCHEMA_URL,
        )
    });

static OPENTELEMETRY_ENDPOINT: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    let host = option_env!("OPENTELEMETRY_HOST").unwrap_or("0.0.0.0");
    let port = option_env!("OPENTELEMETRY_COLLECTOR_GRPC_PORT").unwrap_or("4317");
    format!("http://{host}:{port}")
});

fn init_tracer() -> Result<opentelemetry_sdk::trace::TracerProvider, Error> {
    use opentelemetry_otlp::{SpanExporter, WithExportConfig};
    use opentelemetry_sdk::trace::{RandomIdGenerator, TracerProvider};

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(OPENTELEMETRY_ENDPOINT.as_str())
        .build()?;
    let provider = TracerProvider::builder()
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(RESOURCE.clone())
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build();
    opentelemetry::global::set_tracer_provider(provider.clone());
    Ok(provider)
}

fn init_meter() -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, Error> {
    use opentelemetry_otlp::{MetricExporter, WithExportConfig};
    use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};

    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(OPENTELEMETRY_ENDPOINT.as_str())
        .build()?;
    let reader = PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio).build();
    let provider = SdkMeterProvider::builder()
        .with_resource(RESOURCE.clone())
        .with_reader(reader)
        .build();
    opentelemetry::global::set_meter_provider(provider.clone());
    Ok(provider)
}

fn init_logger() -> Result<opentelemetry_sdk::logs::LoggerProvider, Error> {
    use opentelemetry_otlp::{LogExporter, WithExportConfig};
    use opentelemetry_sdk::logs::LoggerProvider;

    let exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint(OPENTELEMETRY_ENDPOINT.as_str())
        .build()?;
    let provider = LoggerProvider::builder()
        .with_resource(RESOURCE.clone())
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build();
    Ok(provider)
}

pub(crate) fn instrument() -> Result<impl Fn() -> Result<(), Error>, Error> {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
    use tracing::level_filters::LevelFilter;
    use tracing_opentelemetry::OpenTelemetryLayer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt as _;
    use tracing_subscriber::EnvFilter;

    let tracer = init_tracer()?;
    let meter = init_meter()?;
    let logger = init_logger()?;

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
                .from_env_lossy(),
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
