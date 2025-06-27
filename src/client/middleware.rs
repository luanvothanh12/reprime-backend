use std::time::Duration;
use tower_http::{
    timeout::TimeoutLayer,
    trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse},
};
use tracing::Level;

/// HTTP Client middleware stack builder
pub struct HttpMiddlewareBuilder {
    timeout: Option<Duration>,
    enable_tracing: bool,
}

impl Default for HttpMiddlewareBuilder {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            enable_tracing: true,
        }
    }
}

impl HttpMiddlewareBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn enable_tracing(mut self, enable: bool) -> Self {
        self.enable_tracing = enable;
        self
    }

    pub fn build_timeout_layer(&self) -> Option<TimeoutLayer> {
        self.timeout.map(TimeoutLayer::new)
    }

    pub fn build_trace_layer(&self) -> Option<TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>> {
        if self.enable_tracing {
            Some(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO))
            )
        } else {
            None
        }
    }
}

/// Metrics middleware for HTTP clients
pub struct HttpClientMetrics {
    pub requests_total: prometheus::CounterVec,
    pub request_duration: prometheus::HistogramVec,
    pub requests_in_flight: prometheus::GaugeVec,
}

impl HttpClientMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let requests_total = prometheus::CounterVec::new(
            prometheus::Opts::new("http_client_requests_total", "Total HTTP client requests"),
            &["method", "host", "status_code"],
        )?;

        let request_duration = prometheus::HistogramVec::new(
            prometheus::HistogramOpts::new(
                "http_client_request_duration_seconds",
                "HTTP client request duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["method", "host"],
        )?;

        let requests_in_flight = prometheus::GaugeVec::new(
            prometheus::Opts::new("http_client_requests_in_flight", "HTTP client requests in flight"),
            &["method", "host"],
        )?;

        Ok(Self {
            requests_total,
            request_duration,
            requests_in_flight,
        })
    }

    pub fn register(&self, registry: &prometheus::Registry) -> Result<(), prometheus::Error> {
        registry.register(Box::new(self.requests_total.clone()))?;
        registry.register(Box::new(self.request_duration.clone()))?;
        registry.register(Box::new(self.requests_in_flight.clone()))?;
        Ok(())
    }
}

impl Default for HttpClientMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create HTTP client metrics")
    }
}
