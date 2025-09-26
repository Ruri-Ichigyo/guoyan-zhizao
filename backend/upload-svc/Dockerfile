FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY upload-svc/Cargo.toml ./upload-svc/
RUN mkdir upload-svc/src && echo "fn main() {}" > upload-svc/src/main.rs

# Build dependencies
RUN cargo build --release --package upload-svc
RUN rm upload-svc/src/main.rs

# Copy source code and rebuild
COPY upload-svc/src ./upload-svc/src
RUN cargo build --release --package upload-svc

FROM debian:bookworm-slim

# Install OpenSSL and CA certificates
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/upload-svc /usr/local/bin/

EXPOSE 3001

CMD ["upload-svc"]