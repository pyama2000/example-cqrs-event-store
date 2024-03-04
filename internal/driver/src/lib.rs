use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use app::{WidgetService, WidgetServiceError};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use lib::Error;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT, SERVICE_NAME, SERVICE_VERSION,
};
use serde::Deserialize;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::signal;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
pub struct Server<T: ToSocketAddrs> {
    addr: T,
    router: Router,
}

impl<T: ToSocketAddrs + std::fmt::Display> Server<T> {
    pub fn new<S>(addr: T, service: Arc<S>) -> Self
    where
        S: WidgetService + Debug + Send + Sync + 'static,
    {
        let router = Router::new()
            .route("/healthz", get(|| async { StatusCode::OK }))
            .route("/panic", get(|| async { panic!("panic!") }))
            .nest(
                "/widgets",
                Router::new().route("/", post(create_widget)).nest(
                    "/:widget_id",
                    Router::new()
                        .route("/name", post(change_widget_name))
                        .route("/description", post(change_widget_description)),
                ),
            )
            .with_state(service)
            .layer(TraceLayer::new_for_http())
            .layer(TimeoutLayer::new(Duration::from_millis(1500)))
            .layer(CatchPanicLayer::new());
        Self { addr, router }
    }

    pub async fn run(self) -> Result<(), Error> {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_trace_config(
                opentelemetry_sdk::trace::Config::default()
                    .with_id_generator(opentelemetry_sdk::trace::RandomIdGenerator::default())
                    .with_resource(opentelemetry_sdk::Resource::from_schema_url(
                        [
                            opentelemetry::KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
                            opentelemetry::KeyValue::new(
                                SERVICE_VERSION,
                                env!("CARGO_PKG_VERSION"),
                            ),
                            opentelemetry::KeyValue::new(DEPLOYMENT_ENVIRONMENT, "production"),
                        ],
                        opentelemetry_semantic_conventions::SCHEMA_URL,
                    )),
            )
            .with_batch_config(opentelemetry_sdk::trace::BatchConfig::default())
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    // TODO: 外部から値を注入する
                    .with_endpoint("http://localhost:4317"),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .with(tracing_opentelemetry::OpenTelemetryLayer::new(tracer))
            .init();
        let listener = TcpListener::bind(&self.addr).await?;
        tracing::info!("listening: {}", &self.addr);
        axum::serve(listener, self.router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct CreateWidget {
    widget_name: String,
    widget_description: String,
}

#[derive(Deserialize)]
struct ChangeWidgetName {
    widget_name: String,
}

#[derive(Deserialize)]
struct ChangeWidgetDescription {
    widget_description: String,
}

#[tracing::instrument]
async fn create_widget<S: WidgetService + Debug>(
    State(service): State<Arc<S>>,
    Json(CreateWidget {
        widget_name,
        widget_description,
    }): Json<CreateWidget>,
) -> impl IntoResponse {
    match service.create_widget(widget_name, widget_description).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "widget_id": id })),
        )
            .into_response(),
        Err(e) => handling_service_error(e).into_response(),
    }
}

#[tracing::instrument]
async fn change_widget_name<S: WidgetService + Debug>(
    Path(widget_id): Path<String>,
    State(service): State<Arc<S>>,
    Json(ChangeWidgetName { widget_name }): Json<ChangeWidgetName>,
) -> impl IntoResponse {
    match service.change_widget_name(widget_id, widget_name).await {
        Ok(_) => StatusCode::ACCEPTED.into_response(),
        Err(e) => handling_service_error(e).into_response(),
    }
}

#[tracing::instrument]
async fn change_widget_description<S: WidgetService + Debug>(
    Path(widget_id): Path<String>,
    State(service): State<Arc<S>>,
    Json(ChangeWidgetDescription { widget_description }): Json<ChangeWidgetDescription>,
) -> impl IntoResponse {
    match service
        .change_widget_description(widget_id, widget_description)
        .await
    {
        Ok(_) => StatusCode::ACCEPTED.into_response(),
        Err(e) => handling_service_error(e).into_response(),
    }
}

fn handling_service_error(err: WidgetServiceError) -> impl IntoResponse {
    match err {
        WidgetServiceError::AggregateNotFound => StatusCode::NOT_FOUND.into_response(),
        WidgetServiceError::AggregateConfilict => StatusCode::CONFLICT.into_response(),
        WidgetServiceError::InvalidValue => StatusCode::BAD_REQUEST.into_response(),
        WidgetServiceError::Unknown(e) => {
            tracing::error!(e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .unwrap_or_else(|e| panic!("failed to install Ctrl+C handler: {e}"))
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .unwrap_or_else(|e| panic!("failed to install signal handler: {e}"))
            .recv()
            .await;
    };
    tokio::select! {
        _ = ctrl_c => tracing::debug!("receive ctrl_c signal"),
        _ = terminate => tracing::debug!("receive terminate"),
    }
    tracing::info!("signal received, starting graceful shutdown");
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    use app::{MockWidgetService, WidgetService, WidgetServiceError};
    use axum::body::{self, Body};
    use axum::http::header::CONTENT_TYPE;
    use axum::http::{Method, Request, Response, StatusCode};
    use lib::{DateTime, Error};
    use mockall::predicate;
    use tower::ServiceExt;

    use crate::Server;

    type AsyncAssertFn<'a> = fn(
        name: &'a str,
        response: Response<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

    const ADDR: &str = "127.0.0.1:8080";
    const CONTENT_TYPE_APPLICATION_JSON: &str = "application/json";
    const WIDGET_NAME: &str = "部品名";
    const WIDGET_DESCRIPTION: &str = "部品の説明";

    /// HealthCheck エンドポイントのテスト
    #[tokio::test]
    async fn test_healthcheck() -> Result<(), Error> {
        let service = MockWidgetService::new();
        let server = Server::new(ADDR, Arc::new(service));
        let response = server
            .router
            .oneshot(Request::builder().uri("/healthz").body(Body::empty())?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        Ok(())
    }

    /// 部品を作成するエンドポイントのテスト
    #[tokio::test]
    async fn test_create_widget() -> Result<(), Error> {
        struct TestCase<'a, T: WidgetService> {
            name: &'a str,
            service: T,
            request: Request<Body>,
            assert: AsyncAssertFn<'a>,
        }
        let tests = vec![
            TestCase {
                name: "リクエストボディの JSON の形式が正しい場合、201 が返る",
                service: {
                    let mut service = MockWidgetService::new();
                    service
                        .expect_create_widget()
                        .with(
                            predicate::eq(WIDGET_NAME.to_string()),
                            predicate::eq(WIDGET_DESCRIPTION.to_string()),
                        )
                        .returning(|_, _| {
                            Box::pin(async { Ok(DateTime::DT2023_01_01_00_00_00_00.id()) })
                        });
                    service
                },
                request: Request::builder()
                    .method(Method::POST)
                    .uri("/widgets")
                    .header(CONTENT_TYPE, CONTENT_TYPE_APPLICATION_JSON)
                    .body(Body::from(
                        serde_json::json!({
                            "widget_name": WIDGET_NAME,
                            "widget_description": WIDGET_DESCRIPTION
                        })
                        .to_string(),
                    ))?,
                assert: (move |name, response| {
                    Box::pin(async move {
                        assert_eq!(response.status(), StatusCode::CREATED, "{name}");
                        let json: serde_json::Value = serde_json::from_slice(
                            &body::to_bytes(response.into_body(), usize::MAX).await?,
                        )?;
                        assert_eq!(
                            json,
                            serde_json::json!({
                                "widget_id": DateTime::DT2023_01_01_00_00_00_00.id()
                            }),
                            "{name}"
                        );
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "Service から InvalidValue のエラーが返ってきた場合、400 が返る",
                service: {
                    let mut service = MockWidgetService::new();
                    service.expect_create_widget().returning(|_, _| {
                        Box::pin(async { Err(WidgetServiceError::InvalidValue) })
                    });
                    service
                },
                request: Request::builder()
                    .method(Method::POST)
                    .uri("/widgets")
                    .header(CONTENT_TYPE, CONTENT_TYPE_APPLICATION_JSON)
                    .body(Body::from(
                        serde_json::json!({
                            "widget_name": "",
                            "widget_description": ""
                        })
                        .to_string(),
                    ))?,
                assert: (move |name, response| {
                    Box::pin(async move {
                        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{name}");
                        assert!(
                            body::to_bytes(response.into_body(), usize::MAX)
                                .await?
                                .is_empty(),
                            "{name}"
                        );
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "Service から Unknown のエラーが返ってきた場合、500 が返る",
                service: {
                    let mut service = MockWidgetService::new();
                    service.expect_create_widget().returning(|_, _| {
                        Box::pin(async { Err(WidgetServiceError::Unknown("unknown".into())) })
                    });
                    service
                },
                request: Request::builder()
                    .method(Method::POST)
                    .uri("/widgets")
                    .header(CONTENT_TYPE, CONTENT_TYPE_APPLICATION_JSON)
                    .body(Body::from(
                        serde_json::json!({
                            "widget_name": "",
                            "widget_description": ""
                        })
                        .to_string(),
                    ))?,
                assert: (move |name, response| {
                    Box::pin(async move {
                        assert_eq!(
                            response.status(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "{name}"
                        );
                        Ok(())
                    })
                }),
            },
        ];
        for test in tests {
            let server = Server::new(ADDR, Arc::new(test.service));
            let response = server.router.oneshot(test.request).await?;
            (test.assert)(test.name, response).await?;
        }
        Ok(())
    }

    /// 部品名を変更するエンドポイントのテスト
    #[tokio::test]
    async fn test_change_widget_name() -> Result<(), Error> {
        struct TestCase<'a, T: WidgetService> {
            name: &'a str,
            service: T,
            request: Request<Body>,
            assert: AsyncAssertFn<'a>,
        }
        let tests = vec![
            TestCase {
                name: "リクエストボディの JSON の形式が正しい場合、202 が返る",
                service: {
                    let mut service = MockWidgetService::new();
                    service
                        .expect_change_widget_name()
                        .with(
                            predicate::eq(DateTime::DT2023_01_01_00_00_00_00.id()),
                            predicate::eq(WIDGET_NAME.to_string()),
                        )
                        .returning(|_, _| Box::pin(async { Ok(()) }));
                    service
                },
                request: Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/widgets/{}/name",
                        DateTime::DT2023_01_01_00_00_00_00.id()
                    ))
                    .header(CONTENT_TYPE, CONTENT_TYPE_APPLICATION_JSON)
                    .body(Body::from(
                        serde_json::json!({
                            "widget_name": WIDGET_NAME,
                        })
                        .to_string(),
                    ))?,
                assert: (move |name, response| {
                    Box::pin(async move {
                        assert_eq!(response.status(), StatusCode::ACCEPTED, "{name}");
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "Service から InvalidValue のエラーが返ってきた場合、400 が返る",
                service: {
                    let mut service = MockWidgetService::new();
                    service.expect_change_widget_name().returning(|_, _| {
                        Box::pin(async { Err(WidgetServiceError::InvalidValue) })
                    });
                    service
                },
                request: Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/widgets/{}/name",
                        DateTime::DT2023_01_01_00_00_00_00.id()
                    ))
                    .header(CONTENT_TYPE, CONTENT_TYPE_APPLICATION_JSON)
                    .body(Body::from(
                        serde_json::json!({"widget_name": ""}).to_string(),
                    ))?,
                assert: (move |name, response| {
                    Box::pin(async move {
                        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{name}");
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "Service から Unknown のエラーが返ってきた場合、500 が返る",
                service: {
                    let mut service = MockWidgetService::new();
                    service.expect_change_widget_name().returning(|_, _| {
                        Box::pin(async { Err(WidgetServiceError::Unknown("unknown".into())) })
                    });
                    service
                },
                request: Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/widgets/{}/name",
                        DateTime::DT2023_01_01_00_00_00_00.id()
                    ))
                    .header(CONTENT_TYPE, CONTENT_TYPE_APPLICATION_JSON)
                    .body(Body::from(
                        serde_json::json!({
                            "widget_name": WIDGET_NAME,
                        })
                        .to_string(),
                    ))?,
                assert: (move |name, response| {
                    Box::pin(async move {
                        assert_eq!(
                            response.status(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "{name}"
                        );
                        Ok(())
                    })
                }),
            },
        ];
        for test in tests {
            let server = Server::new(ADDR, Arc::new(test.service));
            let response = server.router.oneshot(test.request).await?;
            (test.assert)(test.name, response).await?;
        }
        Ok(())
    }

    /// 部品の説明を変更するエンドポイントのテスト
    #[tokio::test]
    async fn test_change_widget_description() -> Result<(), Error> {
        struct TestCase<'a, T: WidgetService> {
            name: &'a str,
            service: T,
            request: Request<Body>,
            assert: AsyncAssertFn<'a>,
        }
        let tests = vec![
            TestCase {
                name: "リクエストボディの JSON の形式が正しい場合、202 が返る",
                service: {
                    let mut service = MockWidgetService::new();
                    service
                        .expect_change_widget_description()
                        .with(
                            predicate::eq(DateTime::DT2023_01_01_00_00_00_00.id()),
                            predicate::eq(WIDGET_DESCRIPTION.to_string()),
                        )
                        .returning(|_, _| Box::pin(async { Ok(()) }));
                    service
                },
                request: Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/widgets/{}/description",
                        DateTime::DT2023_01_01_00_00_00_00.id()
                    ))
                    .header(CONTENT_TYPE, CONTENT_TYPE_APPLICATION_JSON)
                    .body(Body::from(
                        serde_json::json!({
                            "widget_description": WIDGET_DESCRIPTION,
                        })
                        .to_string(),
                    ))?,
                assert: (move |name, response| {
                    Box::pin(async move {
                        assert_eq!(response.status(), StatusCode::ACCEPTED, "{name}");
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "Service から InvalidValue のエラーが返ってきた場合、400 が返る",
                service: {
                    let mut service = MockWidgetService::new();
                    service
                        .expect_change_widget_description()
                        .returning(|_, _| {
                            Box::pin(async { Err(WidgetServiceError::InvalidValue) })
                        });
                    service
                },
                request: Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/widgets/{}/description",
                        DateTime::DT2023_01_01_00_00_00_00.id()
                    ))
                    .header(CONTENT_TYPE, CONTENT_TYPE_APPLICATION_JSON)
                    .body(Body::from(
                        serde_json::json!({"widget_description": ""}).to_string(),
                    ))?,
                assert: (move |name, response| {
                    Box::pin(async move {
                        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{name}");
                        Ok(())
                    })
                }),
            },
            TestCase {
                name: "Service から Unknown のエラーが返ってきた場合、500 が返る",
                service: {
                    let mut service = MockWidgetService::new();
                    service
                        .expect_change_widget_description()
                        .returning(|_, _| {
                            Box::pin(async { Err(WidgetServiceError::Unknown("unknown".into())) })
                        });
                    service
                },
                request: Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/widgets/{}/description",
                        DateTime::DT2023_01_01_00_00_00_00.id()
                    ))
                    .header(CONTENT_TYPE, CONTENT_TYPE_APPLICATION_JSON)
                    .body(Body::from(
                        serde_json::json!({
                            "widget_description": WIDGET_DESCRIPTION,
                        })
                        .to_string(),
                    ))?,
                assert: (move |name, response| {
                    Box::pin(async move {
                        assert_eq!(
                            response.status(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "{name}"
                        );
                        Ok(())
                    })
                }),
            },
        ];
        for test in tests {
            let server = Server::new(ADDR, Arc::new(test.service));
            let response = server.router.oneshot(test.request).await?;
            (test.assert)(test.name, response).await?;
        }
        Ok(())
    }
}
