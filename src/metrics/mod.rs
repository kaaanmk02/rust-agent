use lazy_static::lazy_static;
use prometheus::{
    Encoder, Histogram, HistogramOpts, IntCounterVec, IntGauge, Opts, Registry,
    TextEncoder,
};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref REQUESTS: IntCounterVec = IntCounterVec::new(
        Opts::new("requests_total", "Total requests"),
        &["handler"]
    ).unwrap();
    pub static ref ERRORS: IntCounterVec = IntCounterVec::new(
        Opts::new("errors_total", "Total errors"),
        &["handler"]
    ).unwrap();
    pub static ref LATENCY: Histogram = Histogram::with_opts(
        HistogramOpts::new("request_latency_seconds", "Request latency")
    ).unwrap();
    pub static ref QUEUE_DEPTH: IntGauge = IntGauge::new(
        "queue_depth", "Current queue depth"
    ).unwrap();
    pub static ref ACTIVE_ADAPTERS: IntGauge = IntGauge::new(
        "active_adapters", "Active adapters"
    ).unwrap();
}

pub fn init_metrics() {
    let _ = REGISTRY.register(Box::new(REQUESTS.clone()));
    let _ = REGISTRY.register(Box::new(ERRORS.clone()));
    let _ = REGISTRY.register(Box::new(LATENCY.clone()));
    let _ = REGISTRY.register(Box::new(QUEUE_DEPTH.clone()));
    let _ = REGISTRY.register(Box::new(ACTIVE_ADAPTERS.clone()));
}

pub fn metrics_text() -> String {
    let mut buf = Vec::new();
    let encoder = TextEncoder::new();
    let _ = encoder.encode(&REGISTRY.gather(), &mut buf);
    String::from_utf8(buf).unwrap_or_default()
}
