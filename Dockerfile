FROM rust:1.90-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src ./src

RUN cargo build --release


FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/vehicle-simulator /app/vahicle-simulator

EXPOSE 8080

ENTRYPOINT ["/app/vehicle-simulator"]