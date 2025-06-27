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
    
    // Database metrics
    pub database_connections_active: Gauge,
    pub database_connections_idle: Gauge,
    pub database_query_duration_seconds: HistogramVec,
    pub database_queries_total: CounterVec,
    
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
            &["method", "endpoint", "status_code"],
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
        registry.register(Box::new(database_connections_active.clone()))?;
        registry.register(Box::new(database_connections_idle.clone()))?;
        registry.register(Box::new(database_query_duration_seconds.clone()))?;
        registry.register(Box::new(database_queries_total.clone()))?;
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
            database_connections_active,
            database_connections_idle,
            database_query_duration_seconds,
            database_queries_total,
            users_created_total,
            users_updated_total,
            users_deleted_total,
            users_retrieved_total,
            memory_usage_bytes,
            cpu_usage_percent,
        })
    }

    /// Record an HTTP request
    pub fn record_http_request(&self, method: &str, endpoint: &str, status_code: u16, duration: f64) {
        self.http_requests_total
            .with_label_values(&[method, endpoint, &status_code.to_string()])
            .inc();
        
        self.http_request_duration_seconds
            .with_label_values(&[method, endpoint])
            .observe(duration);
    }

    /// Record database query
    pub fn record_database_query(&self, query_type: &str, table: &str, status: &str, duration: f64) {
        self.database_queries_total
            .with_label_values(&[query_type, table, status])
            .inc();
        
        self.database_query_duration_seconds
            .with_label_values(&[query_type, table])
            .observe(duration);
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
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics")
    }
}
