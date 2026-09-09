# Hermes-Lite v2.0 — Rust 2024 core (Hardened + Optimized)

[![CI](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml/badge.svg)](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml)

Pinned to **rustc 1.98.1**. Edition **2024**. **Security-hardened**. **Resource-optimized**.

## Quick Start

### 1. Rust (Native)

```bash
cargo build --release
export OPENAI_API_KEY=sk-...
./target/release/hermes-lite chat
```

### 2. Python Wrapper

```python
from hermes_lite import Agent
agent = Agent()
print(agent.run("Remember I prefer Rust"))
```

### 3. Docker Compose (Production)

```bash
cp .env.example .env
docker compose up -d
```

## Resource Limits (Lightweight)

| Resource | Limit | Purpose |
|---|---|---|
| **CPU** | 10s per shell command | Prevents infinite loops |
| **Memory** | 256MB virtual memory (shell) | Prevents OOM |
| **Files** | 64 open files (shell) | Prevents FD exhaustion |
| **HTTP Response** | 100KB max | Prevents memory bloat |
| **SQLite** | WAL mode, 16MB cache | Fast, low-memory |
| **Binary** | ~2MB (stripped, LTO) | Minimal disk footprint |

## Introspection

```bash
./target/release/hermes-lite memories 20
./target/release/hermes-lite skills
./target/release/hermes-lite backup > backup.sql
```

## Hardening Features

| Layer | Measure |
|---|---|
| **Secrets** | `secrecy::Secret` (never logged) |
| **Input** | Path traversal, control chars, length limits |
| **Gateway** | Rate-limited (60 req/min/IP) |
| **Logging** | Structured JSON (`RUST_LOG_JSON=1`) |
| **Container** | Distroless, non-root, read-only, `cap_drop=ALL` |
| **Systemd** | CPU quota, memory limits, sandboxed |

## Build / test

```bash
rustup show
make ci
cargo test
cargo clippy --all-targets -- -D warnings
cargo audit
```

## Production Deployment

### Docker

```bash
docker compose up -d
```

### Systemd (Bare Metal)

```bash
sudo cp hermes-lite.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable hermes-lite
sudo systemctl start hermes-lite
```

## License

MIT
