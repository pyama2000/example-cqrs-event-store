#[must_use]
pub fn context() -> opentelemetry::Context {
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;

    tracing::Span::current().context()
}

struct Injector<'a>(&'a mut tonic::metadata::MetadataMap);

impl opentelemetry::propagation::Injector for Injector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::try_from(&value) {
                self.0.insert(key, val);
            }
        }
    }
}

pub fn inject(cx: &opentelemetry::Context, metadata: &mut tonic::metadata::MetadataMap) {
    opentelemetry::global::get_text_map_propagator(|p| {
        p.inject_context(cx, &mut Injector(metadata));
    });
}
