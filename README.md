# Hermes-Lite v2.0 — Production-Ready Rust Agent

[![CI](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml/badge.svg)](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml)

**Self-sufficient** • **Minimizes LLM calls** • **Caches responses** • **Auto-creates skills** • **End-to-end tested**

## Quick Start (End-to-End)

### 1. Build

```bash
rustup show  # Must be 1.98.1
cargo build --release
```

### 2. Test

```bash
# Unit tests (no API key needed)
cargo test --lib

# Integration tests (needs API key)
export OPENAI_API_KEY=sk-...
cargo test --test integration -- --ignored
```

### 3. Run

```bash
export OPENAI_API_KEY=sk-...
./target/release/hermes-lite chat
```

## Verify End-to-End

```bash
# 1. Check binary builds
file target/release/hermes-lite
# → ELF 64-bit LSB executable, statically linked

# 2. Check health
./target/release/hermes-lite run healthcheck
# → ok

# 3. Run agent
./target/release/hermes-lite run "What is 2+2?"
# → Agent responds

# 4. Check learning
./target/release/hermes-lite stats
# → Shows cache_size, skills_created, etc.

# 5. Check memories
./target/release/hermes-lite memories
# → Shows learned preferences

# 6. Check skills
./target/release/hermes-lite skills
# → Shows auto-created skills
```

## Self-Sufficiency Features

| Feature | How It Works | Benefit |
|---|---|---|
| **Response Cache** | MD5 hash of queries cached in-memory | Repeated questions = instant answers |
| **Skill Auto-Load** | Skills listed in system prompt | Agent uses learned patterns first |
| **Smart Learning** | Only creates skills for complex tasks | Avoids skill spam |
| **Preference Memory** | Extracts "I prefer", "always", "never" | Remembers user style |
| **Tool Batching** | Executes all tool calls per turn | Fewer LLM round-trips |

## All Features

| Category | Features |
|---|---|
| **Core** | Self-learning, SQLite memory, skill creation, response caching |
| **Tools** | Shell, files, fetch, search, memory, skills, skill_create |
| **Sub-Agents** | Parallel task delegation, background execution |
| **MCP** | JSON-RPC 2.0 server (Cursor, Claude Code compatible) |
| **Gateway** | REST API, rate-limited, token auth |
| **Security** | SSRF protection, path jail, ulimits, secrets management |
| **Resources** | 2MB binary, 256MB RAM limit, 10s CPU limit |
| **Deploy** | Docker, Docker Compose, systemd, bare metal |
| **Tests** | Unit tests, integration tests, CI pipeline |

## Resource Limits

| Resource | Limit |
|---|---|
| Binary | ~2MB (LTO, stripped) |
| Shell CPU | 10s per command |
| Shell Memory | 256MB virtual |
| Shell Files | 64 FDs |
| HTTP Body | 100KB max |
| SQLite | WAL, 16MB cache |

## Build & Test

```bash
rustup show
make ci              # fmt, lint, audit, test, build
cargo test --lib     # Unit tests (no API key)
cargo test --test integration -- --ignored  # E2E tests (needs API key)
cargo clippy --all-targets -- -D warnings
cargo audit
```

## Deploy

### Docker

```bash
cp .env.example .env
docker compose up -d
```

### Systemd

```bash
sudo cp hermes-lite.service /etc/systemd/system/
sudo systemctl enable hermes-lite
sudo systemctl start hermes-lite
```

## License

MIT
