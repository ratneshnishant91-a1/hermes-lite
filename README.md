# Hermes-Lite v2.0 — Rust 2024 core (Hardened)

[![CI](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml/badge.svg)](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml)

Pinned to **rustc 1.98.1** (3 Sep 2026). Edition **2024**. **Security-hardened**. **Introspectable**.

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
# Edit .env with your API key
docker compose up -d
```

## Introspection (See What It Learned)

```bash
# View recent memories
./target/release/hermes-lite memories 20

# List learned skills
./target/release/hermes-lite skills

# Backup state
./target/release/hermes-lite backup > backup.sql

# Stats
./target/release/hermes-lite stats
```

## Hardening Features

| Layer | Measure |
|---|---|
| **Secrets** | API keys wrapped in `secrecy::Secret` |
| **Input** | Path traversal, control chars, length limits |
| **Gateway** | Rate-limited (60 req/min/IP) |
| **Logging** | Structured JSON (`RUST_LOG_JSON=1`) |
| **Container** | Distroless, non-root, read-only, `cap_drop=ALL` |
| **State** | SQLite backup/restore, volumes |

## Build / test

```bash
rustup show
make ci
cargo test
cargo clippy --all-targets -- -D warnings
cargo audit
```

## Production Deployment

```bash
docker compose up -d
# Gateway on http://localhost:8000
# Data persisted in Docker volumes
```

## License

MIT
