# Stage 1: Build
FROM rust:bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    make \
    perl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies layer: copy only manifests and build a dummy binary first.
# When only src/ changes, Docker reuses this cached layer and skips re-downloading crates.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null; rm -rf src

# Build the real application
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# ca-certificates is required to make HTTPS calls to push service endpoints
# (fcm.googleapis.com, updates.push.services.mozilla.com, etc.)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1001 appuser
WORKDIR /app

COPY --from=builder /app/target/release/my-cloud-messaging ./server
COPY --from=builder /app/migrations ./migrations

RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 8080

CMD ["./server"]
