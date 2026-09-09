# Hermes-Lite v2.0 — Complete Rust Agent Platform

[![CI](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml/badge.svg)](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml)

**Rustc 1.98.1** • **Edition 2024** • **Hardened** • **Lightweight** • **Complete**

## Features

| Category | Features |
|---|---|
| **Core** | Self-learning, SQLite memory, skill creation, context compression |
| **Tools** | Shell, files, fetch, search, memory, skills |
| **Sub-Agents** | Parallel task delegation, background execution |
| **MCP** | JSON-RPC 2.0 server (Cursor, Claude Code compatible) |
| **Gateway** | REST API, rate-limited, token auth |
| **Security** | SSRF protection, path jail, ulimits, secrets management |
| **Resources** | 2MB binary, 256MB RAM limit, 10s CPU limit per command |
| **Deploy** | Docker, Docker Compose, systemd, bare metal |

## Quick Start

### Rust CLI

```bash
cargo build --release
export OPENAI_API_KEY=sk-...
./target/release/hermes-lite chat
```

### Sub-Agents

```bash
# Spawn parallel task
./target/release/hermes-lite agents spawn "Research Python best practices"

# List tasks
./target/release/hermes-lite agents list

# Get result
./target/release/hermes-lite agents get <task_id>
```

### MCP Server

```bash
./target/release/hermes-lite mcp
# Use with Cursor, Claude Code, etc.
```

### Docker

```bash
cp .env.example .env
docker compose up -d
```

## Introspection

```bash
./target/release/hermes-lite memories 20
./target/release/hermes-lite skills
./target/release/hermes-lite backup > backup.sql
./target/release/hermes-lite stats
```

## Resource Limits

| Resource | Limit |
|---|---|
| Binary | ~2MB (LTO, stripped) |
| Shell CPU | 10s per command |
| Shell Memory | 256MB virtual |
| Shell Files | 64 FDs |
| HTTP Body | 100KB max |
| SQLite | WAL, 16MB cache |

## Security

- Secrets: `secrecy::Secret` (never logged)
- Input: Path traversal, control chars blocked
- Gateway: 60 req/min/IP rate limit
- Container: Distroless, non-root, read-only, `cap_drop=ALL`
- Systemd: CPU 50%, Memory 512M max

## Build

```bash
rustup show  # 1.98.1
make ci      # fmt, lint, audit, test, build
cargo test
cargo clippy --all-targets -- -D warnings
cargo audit
```

## License

MIT
