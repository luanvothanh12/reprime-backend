use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, HistogramOpts, HistogramVec, Opts, Registry,
};
use std::sync::Arc;

/// Application metrics collector
#[derive(Clone)]
pub struct AppMetrics {
    pub registry: Arc<Registry>,
    
    // HTTP metrics
    pub http_requests_total: CounterVec,
    pub http_request_duration_seconds: HistogramVec,
    pub http_requests_in_flight: GaugeVec,
    pub http_error_rate: CounterVec,

    // Database metrics
    pub database_connections_active: Gauge,
    pub database_connections_idle: Gauge,
    pub database_query_duration_seconds: HistogramVec,
    pub database_queries_total: CounterVec,
    pub database_query_errors_total: CounterVec,

    // Cache metrics
    pub cache_hits_total: CounterVec,
    pub cache_misses_total: CounterVec,
    pub cache_operations_duration_seconds: HistogramVec,

    // Application metrics
    pub users_created_total: Counter,
    pub users_updated_total: Counter,
    pub users_deleted_total: Counter,
    pub users_retrieved_total: Counter,

    // System metrics
    pub memory_usage_bytes: Gauge,
    pub cpu_usage_percent: Gauge,
}

impl AppMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());

        // HTTP metrics
        let http_requests_total = CounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests"),
            &["method", "endpoint", "status_code", "status_class"],
        )?;

        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["method", "endpoint"],
        )?;

        let http_requests_in_flight = GaugeVec::new(
            Opts::new("http_requests_in_flight", "Number of HTTP requests currently being processed"),
            &["method", "endpoint"],
        )?;

        let http_error_rate = CounterVec::new(
            Opts::new("http_error_rate", "HTTP error rate counter"),
            &["method", "endpoint", "status_code"],
        )?;

        // Database metrics
        let database_connections_active = Gauge::new(
            "database_connections_active",
            "Number of active database connections",
        )?;

        let database_connections_idle = Gauge::new(
            "database_connections_idle",
            "Number of idle database connections",
        )?;

        let database_query_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "database_query_duration_seconds",
                "Database query duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]),
            &["query_type", "table"],
        )?;

        let database_queries_total = CounterVec::new(
            Opts::new("database_queries_total", "Total number of database queries"),
            &["query_type", "table", "status"],
        )?;

        let database_query_errors_total = CounterVec::new(
            Opts::new("database_query_errors_total", "Total number of database query errors"),
            &["query_type", "table"],
        )?;

        // Cache metrics
        let cache_hits_total = CounterVec::new(
            Opts::new("cache_hits_total", "Total number of cache hits"),
            &["cache_type", "operation"],
        )?;

        let cache_misses_total = CounterVec::new(
            Opts::new("cache_misses_total", "Total number of cache misses"),
            &["cache_type", "operation"],
        )?;

        let cache_operations_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "cache_operations_duration_seconds",
                "Cache operation duration in seconds",
            )
            .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5]),
            &["cache_type", "operation"],
        )?;

        // Application metrics
        let users_created_total = Counter::new(
            "users_created_total",
            "Total number of users created",
        )?;

        let users_updated_total = Counter::new(
            "users_updated_total",
            "Total number of users updated",
        )?;

        let users_deleted_total = Counter::new(
            "users_deleted_total",
            "Total number of users deleted",
        )?;

        let users_retrieved_total = Counter::new(
            "users_retrieved_total",
            "Total number of user retrievals",
        )?;

        // System metrics
        let memory_usage_bytes = Gauge::new(
            "memory_usage_bytes",
            "Current memory usage in bytes",
        )?;

        let cpu_usage_percent = Gauge::new(
            "cpu_usage_percent",
            "Current CPU usage percentage",
        )?;

        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration_seconds.clone()))?;
        registry.register(Box::new(http_requests_in_flight.clone()))?;
        registry.register(Box::new(http_error_rate.clone()))?;
        registry.register(Box::new(database_connections_active.clone()))?;
        registry.register(Box::new(database_connections_idle.clone()))?;
        registry.register(Box::new(database_query_duration_seconds.clone()))?;
        registry.register(Box::new(database_queries_total.clone()))?;
        registry.register(Box::new(database_query_errors_total.clone()))?;
        registry.register(Box::new(cache_hits_total.clone()))?;
        registry.register(Box::new(cache_misses_total.clone()))?;
        registry.register(Box::new(cache_operations_duration_seconds.clone()))?;
        registry.register(Box::new(users_created_total.clone()))?;
        registry.register(Box::new(users_updated_total.clone()))?;
        registry.register(Box::new(users_deleted_total.clone()))?;
        registry.register(Box::new(users_retrieved_total.clone()))?;
        registry.register(Box::new(memory_usage_bytes.clone()))?;
        registry.register(Box::new(cpu_usage_percent.clone()))?;

        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration_seconds,
            http_requests_in_flight,
            http_error_rate,
            database_connections_active,
            database_connections_idle,
            database_query_duration_seconds,
            database_queries_total,
            database_query_errors_total,
            cache_hits_total,
            cache_misses_total,
            cache_operations_duration_seconds,
            users_created_total,
            users_updated_total,
            users_deleted_total,
            users_retrieved_total,
            memory_usage_bytes,
            cpu_usage_percent,
        })
    }

    /// Record an HTTP request with a trace correlation
    pub fn record_http_request(&self, method: &str, endpoint: &str, status_code: u16, duration: f64) {
        let status_class = match status_code {
            200..=299 => "2xx",
            300..=399 => "3xx",
            400..=499 => "4xx",
            500..=599 => "5xx",
            _ => "other",
        };

        self.http_requests_total
            .with_label_values(&[method, endpoint, &status_code.to_string(), status_class])
            .inc();

        self.http_request_duration_seconds
            .with_label_values(&[method, endpoint])
            .observe(duration);

        // Record error metrics for 4xx/5xx responses
        if status_code >= 400 {
            self.http_error_rate
                .with_label_values(&[method, endpoint, &status_code.to_string()])
                .inc();
        }

        // Log metrics with trace correlation for debugging
        if let Some(trace_id) = crate::telemetry::current_trace_id() {
            tracing::debug!(
                trace_id = %trace_id,
                method = %method,
                endpoint = %endpoint,
                status_code = status_code,
                status_class = %status_class,
                duration_seconds = duration,
                "HTTP request metrics recorded"
            );
        }
    }

    /// Record database query with trace correlation
    pub fn record_database_query(&self, query_type: &str, table: &str, status: &str, duration: f64) {
        self.database_queries_total
            .with_label_values(&[query_type, table, status])
            .inc();

        self.database_query_duration_seconds
            .with_label_values(&[query_type, table])
            .observe(duration);

        // Record errors separately
        if status == "error" {
            self.database_query_errors_total
                .with_label_values(&[query_type, table])
                .inc();
        }

        // Log database metrics with trace correlation
        if let Some(trace_id) = crate::telemetry::current_trace_id() {
            tracing::debug!(
                trace_id = %trace_id,
                query_type = %query_type,
                table = %table,
                status = %status,
                duration_seconds = duration,
                "Database query metrics recorded"
            );
        }
    }

    /// Update database connection metrics
    pub fn update_database_connections(&self, active: i64, idle: i64) {
        self.database_connections_active.set(active as f64);
        self.database_connections_idle.set(idle as f64);
    }

    /// Record user operations
    pub fn record_user_created(&self) {
        self.users_created_total.inc();
    }

    pub fn record_user_updated(&self) {
        self.users_updated_total.inc();
    }

    pub fn record_user_deleted(&self) {
        self.users_deleted_total.inc();
    }

    pub fn record_user_retrieved(&self) {
        self.users_retrieved_total.inc();
    }

    /// Update system metrics
    pub fn update_system_metrics(&self, memory_bytes: f64, cpu_percent: f64) {
        self.memory_usage_bytes.set(memory_bytes);
        self.cpu_usage_percent.set(cpu_percent);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self, cache_type: &str, operation: &str, duration: f64) {
        self.cache_hits_total
            .with_label_values(&[cache_type, operation])
            .inc();

        self.cache_operations_duration_seconds
            .with_label_values(&[cache_type, operation])
            .observe(duration);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self, cache_type: &str, operation: &str, duration: f64) {
        self.cache_misses_total
            .with_label_values(&[cache_type, operation])
            .inc();

        self.cache_operations_duration_seconds
            .with_label_values(&[cache_type, operation])
            .observe(duration);
    }
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create default AppMetrics")
    }
}
