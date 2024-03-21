use std::backtrace::Backtrace;
use std::time::Duration;

use axum::body::Body;
use axum::extract::{Host, MatchedPath};
use axum::http::header::USER_AGENT;
use axum::http::{HeaderMap, HeaderName, Request, StatusCode};
use axum::response::Response;
use bytes::Bytes;
use http_body_util::Full;
use opentelemetry::propagation::Extractor;
use opentelemetry_semantic_conventions::trace::{
    CLIENT_ADDRESS, EXCEPTION_ESCAPED, EXCEPTION_MESSAGE, EXCEPTION_STACKTRACE, EXCEPTION_TYPE,
    HTTP_REQUEST_METHOD, HTTP_RESPONSE_STATUS_CODE, HTTP_ROUTE, NETWORK_PROTOCOL_VERSION, URL_PATH,
    USER_AGENT_ORIGINAL,
};
use tower_http::classify::ServerErrorsFailureClass;
use tracing::{field, Level, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

struct HeaderExtractor<'a>(&'a HeaderMap);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(HeaderName::as_str).collect::<Vec<_>>()
    }
}

fn method(req: &Request<Body>) -> &str {
    req.method().as_str()
}

fn route(req: &Request<Body>) -> &str {
    req.extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "", |p| p.as_str())
}

/// リクエストからスパンを作成する
pub fn make_span(req: &Request<Body>) -> Span {
    let empty = field::Empty;
    let span_name = format!("{} {}", method(req), route(req));
    let span = tracing::info_span!(
        "",
        otel.name = span_name,
        { URL_PATH } = empty,
        { HTTP_ROUTE } = empty,
        { HTTP_REQUEST_METHOD } = empty,
        http.request.headers = empty,
        { HTTP_RESPONSE_STATUS_CODE } = empty,
        { NETWORK_PROTOCOL_VERSION } = empty,
        { CLIENT_ADDRESS } = empty,
        { USER_AGENT_ORIGINAL } = empty,
        "error.type" = empty,
    );
    span.set_parent(opentelemetry::global::get_text_map_propagator(
        |propagator| propagator.extract(&HeaderExtractor(req.headers())),
    ));
    span
}

/// リクエストの情報をスパンに記録する
pub fn record_request(req: &Request<Body>, span: &Span) {
    span.record(URL_PATH, req.uri().path());
    span.record(HTTP_ROUTE, route(req));
    span.record(HTTP_REQUEST_METHOD, method(req));
    span.record("http.request.headers", field::debug(req.headers()));
    span.record(NETWORK_PROTOCOL_VERSION, field::debug(req.version()));
    span.record(
        CLIENT_ADDRESS,
        req.extensions().get::<Host>().map(|Host(address)| address),
    );
    span.record(
        USER_AGENT_ORIGINAL,
        req.headers()
            .get(USER_AGENT)
            .map(|v| v.to_str().unwrap_or_default()),
    );
}

/// レスポンスの情報をスパンに記録する
pub fn record_response(res: &Response, _: Duration, span: &Span) {
    let status = res.status();
    span.record(HTTP_RESPONSE_STATUS_CODE, field::display(status));
    if !status.is_success() {
        span.record("error.type", field::display(status));
    }
}

/// エラー時の情報をスパンに記録する
pub fn record_failure(err: ServerErrorsFailureClass, _: Duration, _: &Span) {
    tracing::event!(
        Level::ERROR,
        { EXCEPTION_MESSAGE } = err.to_string(),
        { EXCEPTION_TYPE } = err.to_string(),
        "exception"
    );
}

/// panic の情報をスパンに記録する
pub fn record_panic(
    err: Box<dyn std::any::Any + Send + 'static>,
) -> axum::http::Response<Full<Bytes>> {
    let message = if let Some(s) = err.downcast_ref::<String>() {
        s
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s
    } else {
        "unknown"
    };
    let backtrace = Backtrace::force_capture();
    tracing::event!(
        Level::ERROR,
        { EXCEPTION_ESCAPED } = true,
        { EXCEPTION_MESSAGE } = message,
        { EXCEPTION_STACKTRACE } = %backtrace,
        { EXCEPTION_TYPE } = "panic",
        "exception",
    );
    axum::http::Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::from(String::new()))
        .unwrap()
}
