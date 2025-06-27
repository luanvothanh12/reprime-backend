.PHONY: help build run test clean fmt lint audit docker-build docker-run setup-db migrate

# Default target
help:
	@echo "Available commands:"
	@echo "  build        - Build the project"
	@echo "  run          - Run the application in development mode"
	@echo "  test         - Run all tests"
	@echo "  clean        - Clean build artifacts"
	@echo "  fmt          - Format code"
	@echo "  lint         - Run clippy linter"
	@echo "  audit        - Check for security vulnerabilities"
	@echo "  docker-build - Build Docker image"
	@echo "  docker-run   - Run with Docker Compose"
	@echo "  setup-db     - Set up development database"
	@echo "  migrate      - Run database migrations"

# Build the project
build:
	cargo build

# Build for release
build-release:
	cargo build --release

# Run the application
run:
	RUN_MODE=development cargo run

# Run in production mode
run-prod:
	RUN_MODE=production cargo run

# Run tests
test:
	cargo test

# Run tests with coverage
test-coverage:
	cargo tarpaulin --out html

# Clean build artifacts
clean:
	cargo clean

# Format code
fmt:
	cargo fmt

# Check formatting
fmt-check:
	cargo fmt -- --check

# Run clippy linter
lint:
	cargo clippy -- -D warnings

# Check for security vulnerabilities
audit:
	cargo audit

# Build Docker image
docker-build:
	docker build -t reprime-backend .

# Run with Docker Compose
docker-run:
	docker-compose up -d

# Stop Docker Compose
docker-stop:
	docker-compose down

# View Docker logs
docker-logs:
	docker-compose logs -f

# Set up development database
setup-db:
	createdb reprime_backend_dev || true
	createdb reprime_backend_test || true

# Run database migrations
migrate:
	sqlx migrate run

# Create a new migration
migrate-create:
	@read -p "Enter migration name: " name; \
	sqlx migrate add $$name

# Revert last migration
migrate-revert:
	sqlx migrate revert

# Install development dependencies
install-deps:
	cargo install sqlx-cli
	cargo install cargo-tarpaulin
	cargo install cargo-audit

# Full development setup
setup: install-deps setup-db
	cp .env.example .env
	@echo "Development environment set up!"
	@echo "Please edit .env file with your configuration"

# CI/CD pipeline simulation
ci: fmt-check lint test audit

# Development workflow
dev: fmt lint test run
