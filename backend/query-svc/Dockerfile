FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY query-svc/Cargo.toml ./query-svc/
RUN mkdir query-svc/src && echo "fn main() {}" > query-svc/src/main.rs

RUN cargo build --release --package query-svc
RUN rm query-svc/src/main.rs

COPY query-svc/src ./query-svc/src
RUN cargo build --release --package query-svc

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/query-svc /usr/local/bin/

EXPOSE 3002

CMD ["query-svc"]