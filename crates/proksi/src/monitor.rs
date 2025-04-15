use prometheus::{register_counter, register_gauge, Counter, Gauge};
use once_cell::sync::Lazy;

// Define Prometheus metrics as static variables
static REQUEST_COUNTER: Lazy<Counter> = Lazy::new(|| {
    register_counter!(
        "proksi_requests_total",
        "Total number of requests processed"
    )
    .unwrap()
});

static ACTIVE_CONNECTIONS: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "proksi_active_connections",
        "Number of currently active connections"
    )
    .unwrap()
});

pub fn init_prometheus() {
    // Force initialization of metrics
    Lazy::force(&REQUEST_COUNTER);
    Lazy::force(&ACTIVE_CONNECTIONS);
}