#!/bin/bash

echo "🚀 Starting Reprime Backend Monitoring Stack..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Start the monitoring stack
echo "📊 Starting Prometheus, Grafana, Loki, Mimir, and Node Exporter..."
docker compose -f docker-compose.monitoring.yml up -d

# Wait a moment for services to start
echo "⏳ Waiting for services to start..."
sleep 10

# Check if services are running
echo "🔍 Checking service status..."
docker compose -f docker-compose.monitoring.yml ps

echo ""
echo "✅ Monitoring stack started successfully!"
echo ""
echo "🌐 Access URLs:"
echo "   • Grafana:    http://localhost:3001 (admin/admin)"
echo "   • Prometheus: http://localhost:9091"
echo "   • Loki:       http://localhost:3100"
echo "   • Mimir:      http://localhost:9009"
echo "   • Node Exporter: http://localhost:9101"
echo ""
echo "🦀 To start your Rust backend with Loki integration:"
echo "   LOKI_URL=http://localhost:3100 cargo run"
echo ""
echo "📈 Backend metrics will be available at:"
echo "   http://localhost:8080/metrics"
echo ""
echo "🛑 To stop the monitoring stack:"
echo "   docker-compose -f docker-compose.monitoring.yml down"
