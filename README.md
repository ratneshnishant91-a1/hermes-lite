# Hermes-Lite v2.0 — Self-Sufficient Rust Agent

[![CI](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml/badge.svg)](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml)

**Minimizes LLM calls** • **Caches responses** • **Auto-creates skills** • **Learns preferences**

## Self-Sufficiency Features

| Feature | How It Works | Benefit |
|---|---|---|
| **Response Cache** | MD5 hash of queries cached in-memory | Repeated questions = instant answers |
| **Skill Auto-Load** | Skills listed in system prompt | Agent uses learned patterns first |
| **Smart Learning** | Only creates skills for complex tasks (>10 messages, >20 chars) | Avoids skill spam |
| **Preference Memory** | Extracts "I prefer", "always", "never" patterns | Remembers user style |
| **Tool Batching** | Executes all tool calls in parallel per turn | Fewer LLM round-trips |

## Quick Start

```bash
cargo build --release
export OPENAI_API_KEY=sk-...
./target/release/hermes-lite chat
```

## Example: Self-Sufficient Workflow

```bash
# First time: Calls LLM, learns pattern
./target/release/hermes-lite run "Backup my SQLite database"
# → Creates skill: auto_backup_my_sqlite

# Second time: Uses skill, minimal LLM
./target/release/hermes-lite run "Backup my SQLite database"
# → [cached] or uses skill directly

# Check what it learned
./target/release/hermes-lite skills
./target/release/hermes-lite memories
```

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

## Introspection

```bash
./target/release/hermes-lite memories 20
./target/release/hermes-lite skills
./target/release/hermes-lite backup > backup.sql
./target/release/hermes-lite stats  # Shows cache_size
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

## Build

```bash
rustup show  # 1.98.1
make ci
cargo test
cargo clippy --all-targets -- -D warnings
cargo audit
```

## License

MIT
