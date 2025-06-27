# Reprime Backend

A modern, production-ready Rust backend application built with best practices and clean architecture.

## 🚀 Features

- **Clean Architecture**: Layered architecture with clear separation of concerns
- **Async/Await**: Built on Tokio for high-performance async operations
- **Database Integration**: PostgreSQL with SQLx for type-safe database operations
- **Configuration Management**: Environment-based configuration with TOML files
- **Structured Logging**: Comprehensive logging with tracing and tracing-subscriber
- **Error Handling**: Robust error handling with custom error types
- **Middleware**: CORS, compression, timeout, and request logging
- **Testing**: Unit and integration test setup
- **Containerization**: Docker and Docker Compose support
- **Database Migrations**: Automated database schema management

## 🏗️ Architecture

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library crate exports
├── config/              # Configuration management
├── errors.rs            # Custom error types
├── handlers/            # HTTP request handlers
├── middleware/          # Custom middleware
├── models/              # Data models and DTOs
├── repositories/        # Data access layer
├── routes.rs            # Route definitions
├── services/            # Business logic layer
└── utils/               # Utility functions
```

### Layer Responsibilities

- **Handlers**: Handle HTTP requests/responses, input validation
- **Services**: Business logic, orchestration between repositories
- **Repositories**: Data access, database operations
- **Models**: Data structures, DTOs, request/response types
- **Middleware**: Cross-cutting concerns (logging, CORS, etc.)

## 🛠️ Technology Stack

- **Framework**: [Axum](https://github.com/tokio-rs/axum) - Modern async web framework
- **Database**: [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- **Async Runtime**: [Tokio](https://tokio.rs/) - Async runtime
- **Serialization**: [Serde](https://serde.rs/) - Serialization framework
- **Logging**: [Tracing](https://tracing.rs/) - Structured logging
- **Configuration**: [Config](https://github.com/mehcode/config-rs) - Configuration management
- **Error Handling**: [Anyhow](https://github.com/dtolnay/anyhow) - Error handling

## 📋 Prerequisites

- Rust 1.75 or later
- PostgreSQL 12 or later
- Docker and Docker Compose (optional)

## 🚀 Quick Start

### 1. Clone the repository

```bash
git clone <repository-url>
cd reprime-backend
```

### 2. Set up environment

```bash
cp .env.example .env
# Edit .env with your configuration
```

### 3. Set up database

```bash
# Create database
createdb reprime_backend_dev

# Run migrations (will be done automatically on startup)
```

### 4. Run the application

```bash
# Development mode
cargo run

# Or with specific environment
RUN_MODE=development cargo run
```

The server will start on `http://127.0.0.1:3000`

## 🐳 Docker Setup

### Using Docker Compose (Recommended)

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Manual Docker Build

```bash
# Build image
docker build -t reprime-backend .

# Run container
docker run -p 3000:8080 \
  -e APP_DATABASE_URL=postgresql://user:pass@host/db \
  reprime-backend
```

## 📚 API Documentation

### Health Check

```http
GET /health
```

Response:
```json
{
  "status": "ok",
  "timestamp": "2024-01-01T00:00:00Z",
  "service": "reprime-backend",
  "version": "0.1.0"
}
```

### User Management

#### Create User
```http
POST /api/v1/users
Content-Type: application/json

{
  "email": "user@example.com",
  "username": "johndoe"
}
```

#### Get User
```http
GET /api/v1/users/{id}
```

#### List Users
```http
GET /api/v1/users?page=1&per_page=20
```

#### Update User
```http
PUT /api/v1/users/{id}
Content-Type: application/json

{
  "email": "newemail@example.com",
  "username": "newusername"
}
```

#### Delete User
```http
DELETE /api/v1/users/{id}
```

## ⚙️ Configuration

Configuration is managed through TOML files and environment variables:

### Configuration Files

- `config/default.toml` - Default configuration
- `config/development.toml` - Development environment
- `config/production.toml` - Production environment
- `config/local.toml` - Local overrides (optional)

### Environment Variables

All configuration can be overridden with environment variables using the `APP_` prefix:

```bash
APP_SERVER_HOST=0.0.0.0
APP_SERVER_PORT=8080
APP_DATABASE_URL=postgresql://localhost/mydb
APP_LOGGING_LEVEL=debug
```

## 🧪 Testing

### Run Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# With coverage
cargo tarpaulin --out html
```

### Test Database Setup

For integration tests, set up a separate test database:

```bash
createdb reprime_backend_test
export APP_DATABASE_URL=postgresql://localhost/reprime_backend_test
```

## 📊 Monitoring and Observability

### Logging

The application uses structured logging with tracing:

```rust
tracing::info!("User created successfully: {}", user.id);
tracing::error!("Database error: {:?}", error);
```

### Health Checks

Health check endpoint at `/health` provides:
- Service status
- Timestamp
- Version information

### Metrics (Future Enhancement)

Consider adding:
- Prometheus metrics
- Request duration tracking
- Database connection pool metrics

## 🔧 Development

### Code Style

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check for security vulnerabilities
cargo audit
```

### Database Migrations

```bash
# Create new migration
sqlx migrate add create_new_table

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

## 🚀 Deployment

### Environment Setup

1. **Production Database**: Set up PostgreSQL with proper credentials
2. **Environment Variables**: Configure all required environment variables
3. **SSL/TLS**: Configure reverse proxy (nginx/traefik) for HTTPS
4. **Monitoring**: Set up logging aggregation and monitoring

### Deployment Options

1. **Docker**: Use provided Dockerfile and docker-compose.yml
2. **Kubernetes**: Create K8s manifests based on Docker setup
3. **Cloud Platforms**: Deploy to AWS ECS, Google Cloud Run, etc.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## 📄 License

This project is licensed under the MIT OR Apache-2.0 license.

## 🔗 Resources

- [Axum Documentation](https://docs.rs/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Tokio Documentation](https://docs.rs/tokio/)
- [Tracing Documentation](https://docs.rs/tracing/)
- [Rust Book](https://doc.rust-lang.org/book/)