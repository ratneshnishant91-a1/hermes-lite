# Stage 1: Build
FROM rust:1.98.1-slim AS builder

WORKDIR /app
COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release --locked

# Stage 2: Run (Distroless + Hardened)
FROM gcr.io/distroless/cc-debian12:latest

WORKDIR /app
COPY --from=builder /app/target/release/hermes-lite .
COPY config.yaml .

# Non-root user
USER nonroot:nonroot

# Read-only filesystem (except /tmp)
VOLUME /tmp

# Env vars
ENV RUST_LOG=info
ENV HERMES_BIND=0.0.0.0:8000

EXPOSE 8000

# Healthcheck
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD ["./hermes-lite", "run", "healthcheck"] || exit 1

# Security: run with --cap-drop=ALL --read-only --tmpfs /tmp
ENTRYPOINT ["./hermes-lite", "gateway"]
