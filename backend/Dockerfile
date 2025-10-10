# Multi-stage build for Rust backend

# Build stage
FROM rust:1.75-slim AS builder

# Install dependencies for compilation
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files first to leverage Docker layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from build stage
COPY --from=builder /app/target/release/gyz-backend /app/gyz-backend

# Create a non-root user
RUN useradd -m -u 1001 appuser && \
    chown -R appuser:appuser /app

USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["/app/gyz-backend"]

# Labels for metadata
LABEL maintainer="guoyan-zhizao-team@example.com"
LABEL description="Backend service for 果蔬智能分拣系统 (Intelligent Fruit and Vegetable Sorting System)"
LABEL version="1.0.0"
