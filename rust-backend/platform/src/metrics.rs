use actix_web::{HttpResponse, Responder};
use prometheus::{Encoder, IntCounterVec, Registry, TextEncoder};
use std::sync::OnceLock;

static REGISTRY: OnceLock<Registry> = OnceLock::new();
static HTTP_COUNTER: OnceLock<IntCounterVec> = OnceLock::new();
static EVENT_COUNTER: OnceLock<IntCounterVec> = OnceLock::new();

pub fn init_metrics(service: &'static str) {
    let registry = REGISTRY.get_or_init(Registry::new);

    HTTP_COUNTER.get_or_init(|| {
        let counter = IntCounterVec::new(
            prometheus::Opts::new("http_requests_total", "Total HTTP requests"),
            &["service", "route", "method", "status"],
        )
        .expect("http counter");
        let _ = registry.register(Box::new(counter.clone()));
        counter
    });

    EVENT_COUNTER.get_or_init(|| {
        let counter = IntCounterVec::new(
            prometheus::Opts::new("events_total", "Total events processed"),
            &["service", "stream", "event_type", "outcome"],
        )
        .expect("event counter");
        let _ = registry.register(Box::new(counter.clone()));
        counter
    });

    prometheus::default_registry()
        .register(Box::new(
            prometheus::IntGauge::new(
                format!("{}_service_info", service.replace('-', "_")),
                "Service info marker",
            )
            .expect("service gauge"),
        ))
        .ok();
}

pub fn inc_event(service: &str, stream: &str, event_type: &str, outcome: &str) {
    if let Some(counter) = EVENT_COUNTER.get() {
        counter
            .with_label_values(&[service, stream, event_type, outcome])
            .inc();
    }
}

pub async fn metrics_handler() -> impl Responder {
    let mut metric_families = prometheus::gather();
    if let Some(registry) = REGISTRY.get() {
        metric_families.extend(registry.gather());
    }

    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    if encoder.encode(&metric_families, &mut buffer).is_err() {
        return HttpResponse::InternalServerError().body("failed to encode metrics");
    }

    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer)
}
