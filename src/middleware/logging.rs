use tower_http::trace::{TraceLayer, MakeSpan};
use tracing::{Level, Span};
use axum::extract::Request;

/// Custom span maker that includes trace correlation fields
#[derive(Clone, Debug)]
pub struct TracedMakeSpan;

impl<B> MakeSpan<B> for TracedMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let method = request.method();
        let uri = request.uri();
        let version = request.version();
        let user_agent = request
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        tracing::info_span!(
            "http_request",
            method = %method,
            uri = %uri,
            version = ?version,
            user_agent = %user_agent,
            status_code = tracing::field::Empty,
            latency_ms = tracing::field::Empty,
            trace_id = tracing::field::Empty,
            span_id = tracing::field::Empty,
        )
    }
}

/// Custom request handler that logs with trace correlation
#[derive(Clone, Debug)]
pub struct TracedOnRequest;

impl<B> tower_http::trace::OnRequest<B> for TracedOnRequest {
    fn on_request(&mut self, request: &Request<B>, span: &Span) {
        // Extract or generate trace context
        let (trace_id, span_id) = crate::telemetry::extract_or_generate_trace_id(request.headers());

        span.record("trace_id", &trace_id);
        span.record("span_id", &span_id);

        let user_agent = request.headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown");

        crate::log_with_trace!(info,
            method = %request.method(),
            uri = %request.uri(),
            endpoint = %request.uri().path(),
            user_agent = %user_agent,
            "HTTP request started"
        );
    }
}

/// Custom response handler that logs with trace correlation and metrics
#[derive(Clone, Debug)]
pub struct TracedOnResponse;

impl tower_http::trace::OnResponse<axum::body::Body> for TracedOnResponse {
    fn on_response(
        self,
        response: &axum::response::Response,
        latency: std::time::Duration,
        span: &Span,
    ) {
        let status = response.status();
        let latency_ms = latency.as_millis() as f64;

        span.record("status_code", status.as_u16());
        span.record("latency_ms", latency_ms);

        let level = if status.is_server_error() {
            Level::ERROR
        } else if status.is_client_error() {
            Level::WARN
        } else {
            Level::INFO
        };

        match level {
            Level::ERROR => crate::log_with_trace!(error,
                status_code = status.as_u16(),
                latency_ms = latency_ms,
                status_class = "5xx",
                "HTTP request failed"
            ),
            Level::WARN => crate::log_with_trace!(warn,
                status_code = status.as_u16(),
                latency_ms = latency_ms,
                status_class = "4xx",
                "HTTP request client error"
            ),
            _ => crate::log_with_trace!(info,
                status_code = status.as_u16(),
                latency_ms = latency_ms,
                status_class = "2xx",
                "HTTP request completed successfully"
            ),
        }
    }
}

pub fn logging_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    TracedMakeSpan,
    TracedOnRequest,
    TracedOnResponse,
> {
    TraceLayer::new_for_http()
        .make_span_with(TracedMakeSpan)
        .on_request(TracedOnRequest)
        .on_response(TracedOnResponse)
}
