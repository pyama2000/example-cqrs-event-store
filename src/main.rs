use adapter::persistence::connect;
use adapter::repository::WidgetRepository;
use app::WidgetServiceImpl;
use driver::Server;
use lib::{application_environment, database_url, opentelemetry_endpoint, Error};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{BatchConfig, RandomIdGenerator};
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT, SERVICE_NAME, SERVICE_VERSION,
};
use opentelemetry_semantic_conventions::SCHEMA_URL;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(opentelemetry_sdk::Resource::from_schema_url(
                    [
                        opentelemetry::KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
                        opentelemetry::KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                        opentelemetry::KeyValue::new(
                            DEPLOYMENT_ENVIRONMENT,
                            application_environment(),
                        ),
                    ],
                    SCHEMA_URL,
                )),
        )
        .with_batch_config(BatchConfig::default())
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(opentelemetry_endpoint()),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    let _ = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_log_config(opentelemetry_sdk::logs::Config::default().with_resource(
            opentelemetry_sdk::Resource::from_schema_url(
                [
                    opentelemetry::KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
                    opentelemetry::KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                    opentelemetry::KeyValue::new(DEPLOYMENT_ENVIRONMENT, application_environment()),
                ],
                SCHEMA_URL,
            ),
        ))
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(opentelemetry_endpoint()),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(
            opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(
                &opentelemetry::global::logger_provider(),
            ),
        )
        .init();

    let pool = connect(&database_url()).await?;
    let repository = WidgetRepository::new(pool);
    let service = WidgetServiceImpl::new(repository);
    let server = Server::new("0.0.0.0:8080", service.into());
    server.run().await?;

    opentelemetry::global::shutdown_tracer_provider();
    opentelemetry::global::shutdown_logger_provider();
    Ok(())
}
