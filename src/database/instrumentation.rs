use sqlx::PgPool;
use std::time::Instant;
use tracing::{instrument, Span};
use crate::metrics::AppMetrics;

/// Database instrumentation wrapper for query metrics and tracing
pub struct InstrumentedDatabase {
    pool: PgPool,
    metrics: Option<AppMetrics>,
}

impl InstrumentedDatabase {
    pub fn new(pool: PgPool, metrics: Option<AppMetrics>) -> Self {
        Self { pool, metrics }
    }

    /// Execute a query with full instrumentation
    #[instrument(
        name = "database_query",
        skip(self, query),
        fields(
            db.operation = "query",
            db.statement = %query,
            db.rows_affected = tracing::field::Empty,
            duration_ms = tracing::field::Empty,
            trace_id = tracing::field::Empty,
        )
    )]
    pub async fn execute_query<T>(&self, query: &str) -> Result<T, sqlx::Error>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let start = Instant::now();
        let span = Span::current();
        
        // Add trace correlation
        if let Some(trace_id) = crate::telemetry::current_trace_id() {
            span.record("trace_id", &trace_id);
        }

        let result = sqlx::query_as::<_, T>(query)
            .fetch_one(&self.pool)
            .await;

        let duration = start.elapsed();
        let duration_seconds = duration.as_secs_f64();
        let duration_ms = duration.as_millis() as f64;
        
        span.record("duration_ms", duration_ms);

        // Extract table name from query (simple heuristic)
        let table_name = extract_table_name(query);
        let query_type = extract_query_type(query);
        
        match &result {
            Ok(_) => {
                span.record("db.rows_affected", 1);
                
                if let Some(ref metrics) = self.metrics {
                    metrics.record_database_query(&query_type, &table_name, "success", duration_seconds);
                }

                tracing::info!(
                    query_type = %query_type,
                    table = %table_name,
                    duration_ms = duration_ms,
                    "Database query completed successfully"
                );
            }
            Err(e) => {
                if let Some(ref metrics) = self.metrics {
                    metrics.record_database_query(&query_type, &table_name, "error", duration_seconds);
                }

                tracing::error!(
                    query_type = %query_type,
                    table = %table_name,
                    duration_ms = duration_ms,
                    error = %e,
                    "Database query failed"
                );
            }
        }

        result
    }

    /// Execute multiple queries with instrumentation
    #[instrument(
        name = "database_query_many",
        skip(self, query),
        fields(
            db.operation = "query_many",
            db.statement = %query,
            db.rows_affected = tracing::field::Empty,
            duration_ms = tracing::field::Empty,
            trace_id = tracing::field::Empty,
        )
    )]
    pub async fn execute_query_many<T>(&self, query: &str) -> Result<Vec<T>, sqlx::Error>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let start = Instant::now();
        let span = Span::current();
        
        // Add trace correlation
        if let Some(trace_id) = crate::telemetry::current_trace_id() {
            span.record("trace_id", &trace_id);
        }

        let result = sqlx::query_as::<_, T>(query)
            .fetch_all(&self.pool)
            .await;

        let duration = start.elapsed();
        let duration_seconds = duration.as_secs_f64();
        let duration_ms = duration.as_millis() as f64;
        
        span.record("duration_ms", duration_ms);

        let table_name = extract_table_name(query);
        let query_type = extract_query_type(query);
        
        match &result {
            Ok(rows) => {
                let row_count = rows.len();
                span.record("db.rows_affected", row_count);
                
                if let Some(ref metrics) = self.metrics {
                    metrics.record_database_query(&query_type, &table_name, "success", duration_seconds);
                }

                tracing::info!(
                    query_type = %query_type,
                    table = %table_name,
                    rows_returned = row_count,
                    duration_ms = duration_ms,
                    "Database query completed successfully"
                );
            }
            Err(e) => {
                if let Some(ref metrics) = self.metrics {
                    metrics.record_database_query(&query_type, &table_name, "error", duration_seconds);
                }

                tracing::error!(
                    query_type = %query_type,
                    table = %table_name,
                    duration_ms = duration_ms,
                    error = %e,
                    "Database query failed"
                );
            }
        }

        result
    }

    /// Execute a command (INSERT, UPDATE, DELETE) with instrumentation
    #[instrument(
        name = "database_execute",
        skip(self, query),
        fields(
            db.operation = "execute",
            db.statement = %query,
            db.rows_affected = tracing::field::Empty,
            duration_ms = tracing::field::Empty,
            trace_id = tracing::field::Empty,
        )
    )]
    pub async fn execute_command(&self, query: &str) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        let start = Instant::now();
        let span = Span::current();
        
        // Add trace correlation
        if let Some(trace_id) = crate::telemetry::current_trace_id() {
            span.record("trace_id", &trace_id);
        }

        let result = sqlx::query(query)
            .execute(&self.pool)
            .await;

        let duration = start.elapsed();
        let duration_seconds = duration.as_secs_f64();
        let duration_ms = duration.as_millis() as f64;
        
        span.record("duration_ms", duration_ms);

        let table_name = extract_table_name(query);
        let query_type = extract_query_type(query);
        
        match &result {
            Ok(query_result) => {
                let rows_affected = query_result.rows_affected();
                span.record("db.rows_affected", rows_affected);
                
                if let Some(ref metrics) = self.metrics {
                    metrics.record_database_query(&query_type, &table_name, "success", duration_seconds);
                }

                tracing::info!(
                    query_type = %query_type,
                    table = %table_name,
                    rows_affected = rows_affected,
                    duration_ms = duration_ms,
                    "Database command completed successfully"
                );
            }
            Err(e) => {
                if let Some(ref metrics) = self.metrics {
                    metrics.record_database_query(&query_type, &table_name, "error", duration_seconds);
                }

                tracing::error!(
                    query_type = %query_type,
                    table = %table_name,
                    duration_ms = duration_ms,
                    error = %e,
                    "Database command failed"
                );
            }
        }

        result
    }

    /// Get connection pool metrics
    pub fn get_pool_metrics(&self) -> (u32, u32, u32) {
        let size = self.pool.size();
        let idle = self.pool.num_idle();
        let active = size.saturating_sub(idle as u32);

        if let Some(ref metrics) = self.metrics {
            metrics.update_database_connections(active as i64, idle as i64);
        }

        (active, idle as u32, size)
    }
}

/// Extract table name from SQL query (simple heuristic)
fn extract_table_name(query: &str) -> String {
    let query_lower = query.to_lowercase();
    let words: Vec<&str> = query_lower.split_whitespace().collect();
    
    for (i, word) in words.iter().enumerate() {
        match *word {
            "from" | "into" | "update" | "table" => {
                if i + 1 < words.len() {
                    return words[i + 1].trim_matches(|c: char| !c.is_alphanumeric() && c != '_').to_string();
                }
            }
            _ => continue,
        }
    }
    
    "unknown".to_string()
}

/// Extract query type from SQL query
fn extract_query_type(query: &str) -> String {
    let query_trimmed = query.trim().to_lowercase();
    
    if query_trimmed.starts_with("select") {
        "SELECT".to_string()
    } else if query_trimmed.starts_with("insert") {
        "INSERT".to_string()
    } else if query_trimmed.starts_with("update") {
        "UPDATE".to_string()
    } else if query_trimmed.starts_with("delete") {
        "DELETE".to_string()
    } else if query_trimmed.starts_with("create") {
        "CREATE".to_string()
    } else if query_trimmed.starts_with("drop") {
        "DROP".to_string()
    } else if query_trimmed.starts_with("alter") {
        "ALTER".to_string()
    } else {
        "OTHER".to_string()
    }
}
