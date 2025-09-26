FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY sort-svc/Cargo.toml ./sort-svc/
RUN mkdir sort-svc/src && echo "fn main() {}" > sort-svc/src/main.rs

RUN cargo build --release --package sort-svc
RUN rm sort-svc/src/main.rs

COPY sort-svc/src ./sort-svc/src
RUN cargo build --release --package sort-svc

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sort-svc /usr/local/bin/

EXPOSE 3003

CMD ["sort-svc"]