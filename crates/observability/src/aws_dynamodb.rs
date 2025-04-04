struct Injector<'a, S: std::hash::BuildHasher>(
    &'a mut std::collections::HashMap<String, String, S>,
);

impl<S> opentelemetry::propagation::Injector for Injector<'_, S>
where
    S: std::hash::BuildHasher,
{
    fn set(&mut self, key: &str, value: String) {
        let _ = self.0.insert(key.to_string(), value);
    }
}

pub fn inject<S>(metadata: &mut std::collections::HashMap<String, String, S>)
where
    S: std::hash::BuildHasher,
{
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;

    let cx = tracing::Span::current().context();
    opentelemetry::global::get_text_map_propagator(|p| {
        p.inject_context(&cx, &mut Injector(metadata));
    });
}

struct Extractor<'a, S: std::hash::BuildHasher>(&'a std::collections::HashMap<String, String, S>);

impl<S> opentelemetry::propagation::Extractor for Extractor<'_, S>
where
    S: std::hash::BuildHasher,
{
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(String::as_str).collect()
    }
}

pub fn extract<S>(
    metadata: &std::collections::HashMap<String, String, S>,
) -> opentelemetry::trace::SpanContext
where
    S: std::hash::BuildHasher,
{
    use opentelemetry::trace::TraceContextExt as _;

    let cx = opentelemetry::global::get_text_map_propagator(|p| p.extract(&Extractor(metadata)));
    cx.span().span_context().clone()
}
