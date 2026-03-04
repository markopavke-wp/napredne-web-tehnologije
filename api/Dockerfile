# ══════════════════════════════════════════════════════
# STAGE 1: Builder
# ══════════════════════════════════════════════════════
FROM rust:latest AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src
COPY migrations ./migrations

RUN touch src/main.rs && cargo build --release

# ══════════════════════════════════════════════════════
# STAGE 2: Runtime - koristi istu baznu sliku kao builder
# da GLIBC verzije budu kompatibilne
# ══════════════════════════════════════════════════════
FROM debian:trixie-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/api /app/api
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080

CMD ["/app/api"]