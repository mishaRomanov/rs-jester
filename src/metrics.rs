use lazy_static::lazy_static;
use prometheus;

// Create and register metrics according to documentation.
lazy_static! {
    pub static ref TOTAL_REQUESTS_COUNTER: prometheus::IntCounter =
        prometheus::register_int_counter!("requests_total", "total requests counter").unwrap();
}

lazy_static! {
    pub static ref ERROR_COUNTER: prometheus::IntCounter =
        prometheus::register_int_counter!("errors_total", "total errors counter").unwrap();
}

lazy_static! {
    pub static ref REQUEST_HISTOGRAM: prometheus::Histogram = prometheus::register_histogram!(
        "request_proccessing_time",
        "Histogram of request durations in seconds",
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .unwrap();
}

lazy_static! {
    pub static ref UPSTREAMS_STATUS: prometheus::GaugeVec = prometheus::register_gauge_vec!(
        "upstreams_status",
        "Status of upstream servers (1 = healthy, 0 = unhealthy)",
        &["upstream"]
    )
    .unwrap();
}
