use adapter::persistence::connect;
use adapter::repository::WidgetRepository;
use app::WidgetServiceImpl;
use driver::Server;
use lib::{application_environment, database_url, opentelemetry_endpoint, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _guard = start_instrument()?;

    let pool = connect(&database_url()).await?;
    let repository = WidgetRepository::new(pool);
    let service = WidgetServiceImpl::new(repository);
    let server = Server::new("0.0.0.0:8080", service.into());
    server.run().await?;

    Ok(())
}

struct OpenTelemetryGuard;

impl Drop for OpenTelemetryGuard {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
        opentelemetry::global::shutdown_logger_provider();
    }
}

fn start_instrument() -> Result<OpenTelemetryGuard, Error> {
    use opentelemetry::{global, KeyValue};
    use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
    use opentelemetry_otlp::{new_exporter, new_pipeline, WithExportConfig};
    use opentelemetry_sdk::{
        logs, runtime,
        trace::{self, BatchConfig, RandomIdGenerator},
        Resource,
    };
    use opentelemetry_semantic_conventions::{
        resource::{DEPLOYMENT_ENVIRONMENT, SERVICE_NAME, SERVICE_VERSION},
        SCHEMA_URL,
    };
    use tracing_opentelemetry::OpenTelemetryLayer;
    use tracing_subscriber::{
        filter::LevelFilter, fmt, layer::SubscriberExt, registry, util::SubscriberInitExt,
        EnvFilter,
    };

    let resource = Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new(DEPLOYMENT_ENVIRONMENT, application_environment()),
        ],
        SCHEMA_URL,
    );
    let tracer = new_pipeline()
        .tracing()
        .with_trace_config(
            trace::Config::default()
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(resource.clone()),
        )
        .with_batch_config(BatchConfig::default())
        .with_exporter(
            new_exporter()
                .tonic()
                .with_endpoint(opentelemetry_endpoint()),
        )
        .install_batch(runtime::Tokio)?;
    let _ = new_pipeline()
        .logging()
        .with_log_config(logs::Config::default().with_resource(resource))
        .with_exporter(
            new_exporter()
                .tonic()
                .with_endpoint(opentelemetry_endpoint()),
        )
        .install_batch(runtime::Tokio)?;
    registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .with(OpenTelemetryTracingBridge::new(&global::logger_provider()))
        .init();
    Ok(OpenTelemetryGuard)
}
