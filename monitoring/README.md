# Enhanced Monitoring Setup for Reprime Backend

This directory contains the comprehensive monitoring setup for the Reprime Backend using the modern observability stack.

## Components

- **Prometheus**: Short-term metrics collection and storage
- **Mimir**: Long-term metrics storage with fast querying
- **Loki**: Log aggregation and storage
- **Promtail**: Log collection agent
- **Grafana**: Unified visualization for metrics and logs
- **Node Exporter**: System metrics collection
- **axum-prometheus**: Automatic HTTP metrics collection

## Quick Start

1. Start the monitoring stack:
   ```bash
   docker-compose -f docker-compose.monitoring.yml up -d
   ```

2. Start your Rust backend application with Loki integration:
   ```bash
   LOKI_URL=http://localhost:3100 cargo run
   ```

3. Access the services:
   - **Grafana**: http://localhost:3001 (admin/admin)
   - **Prometheus**: http://localhost:9091
   - **Loki**: http://localhost:3100
   - **Mimir**: http://localhost:9009
   - **Backend Metrics**: http://localhost:8080/metrics

## Available Metrics

### HTTP Metrics (via axum-prometheus)
- `axum_http_requests_total`: Total number of HTTP requests
- `axum_http_requests_duration_seconds`: HTTP request duration histogram
- `axum_http_requests_pending`: Number of HTTP requests currently being processed

### System Metrics (via node-exporter)
- `node_cpu_seconds_total`: CPU usage statistics
- `node_memory_*`: Memory usage metrics
- `node_filesystem_*`: Filesystem usage metrics
- `node_network_*`: Network interface statistics

## Logs Integration

### Structured Logging
- **JSON Format**: All logs are structured in JSON format
- **Loki Integration**: Logs are automatically sent to Loki
- **Labels**: Logs include service and version labels
- **Correlation**: Logs can be correlated with metrics using timestamps

### Log Queries in Grafana
- View logs by service: `{service="reprime-backend"}`
- Filter by log level: `{service="reprime-backend"} |= "ERROR"`
- Search for specific text: `{service="reprime-backend"} |= "user created"`

## Grafana Dashboards

The setup includes a pre-configured dashboard for the Reprime Backend that shows:
- HTTP request rates
- Request duration percentiles
- Error rates
- Database performance
- User operation metrics

## Configuration

### Prometheus
- Configuration: `monitoring/prometheus.yml`
- Scrapes metrics from the backend every 5 seconds
- Retains data for 200 hours

### Grafana
- Datasource: Automatically configured to use Prometheus
- Dashboards: Auto-provisioned from `monitoring/grafana/dashboards/`
- Default credentials: admin/admin

## Customization

### Adding New Metrics
1. Add metrics to `src/metrics.rs`
2. Register them in the `AppMetrics::new()` method
3. Use them in your handlers or services

### Creating New Dashboards
1. Create dashboards in Grafana UI
2. Export as JSON
3. Save to `monitoring/grafana/dashboards/`
4. Restart Grafana to auto-provision

## Troubleshooting

### Backend Metrics Not Showing
- Ensure the backend is running on the correct port (8080)
- Check that `/metrics` endpoint is accessible
- Verify Prometheus can reach `host.docker.internal:8080`

### Grafana Dashboard Empty
- Check Prometheus is scraping successfully at http://localhost:9091/targets
- Verify the backend is exposing metrics at http://localhost:8080/metrics
- Check Grafana datasource configuration

### Docker Issues
- Ensure Docker and Docker Compose are installed
- Check that ports 3001, 9091, and 9101 are not in use
- Run `docker-compose logs` to check for errors
