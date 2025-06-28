# Build stage
FROM rust:1.88-slim as builder

# Install build dependencies including curl for utoipa-swagger-ui
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# First, copy only the manifest files for dependency caching
COPY Cargo.toml ./

# Create dummy src/main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies - this layer will be cached unless Cargo.toml changes
RUN cargo build --release
RUN rm -rf src

# Now copy the actual source code
COPY src ./src
COPY migrations ./migrations
COPY config ./config

# Build the actual application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/reprime-backend .
COPY --from=builder /app/config ./config
COPY --from=builder /app/migrations ./migrations

# Create a non-root user
RUN useradd -r -s /bin/false appuser && chown -R appuser:appuser /app
USER appuser

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUN_MODE=production
ENV APP_SERVER_HOST=0.0.0.0
ENV APP_SERVER_PORT=8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the binary
CMD ["./reprime-backend"]