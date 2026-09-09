# Hermes-Lite v2.0 — Rust 2024 core (Hardened)

[![CI](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml/badge.svg)](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml)

Pinned to **rustc 1.98.1** (3 Sep 2026). Edition **2024**. **Security-hardened**.

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

### 3. Docker (Hardened)

```bash
docker build -t hermes-lite:latest .
docker run --rm -it \
  --read-only --cap-drop=ALL --tmpfs /tmp \
  -e OPENAI_API_KEY=sk-... -p 8000:8000 hermes-lite:latest
```

## Hardening Features

| Layer | Measure |
|---|---|
| **Secrets** | API keys wrapped in `secrecy::Secret` (never logged) |
| **Input** | Path traversal, control chars, length limits |
| **Gateway** | Rate-limited (60 req/min/IP), body size limited |
| **Logging** | Structured JSON (`RUST_LOG_JSON=1`), panic hook |
| **Container** | Distroless, non-root, read-only FS, `cap_drop=ALL` |
| **Dependencies** | `cargo-audit` in CI |

## Build / test

```bash
rustup show
make ci              # fmt, lint, audit, test, build
cargo test
cargo clippy --all-targets -- -D warnings
cargo audit
```

## Production Deployment

```bash
# Build
docker build -t hermes-lite:latest .

# Run (hardened)
docker run --rm -d \
  --name hermes \
  --read-only \
  --cap-drop=ALL \
  --tmpfs /tmp \
  --health-cmd='./hermes-lite run healthcheck' \
  --health-interval=30s \
  -e OPENAI_API_KEY=sk-... \
  -p 8000:8000 \
  hermes-lite:latest
```

## License

MIT
