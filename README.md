# Hermes-Lite v2.0 — Rust 2024 core

[![CI](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml/badge.svg)](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml)

Pinned to **rustc 1.98.1** (3 Sep 2026) via `rust-toolchain.toml`. Edition **2024** (stable since 1.85).

Python files in this repo are a prototype archive. The supported kernel is `cargo run`.

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

agent = Agent()  # Auto-finds binary
print(agent.run("Remember I prefer Rust"))
print(agent.stats())
```

### 3. Docker (Distroless)

```bash
docker build -t hermes-lite:latest .
docker run --rm -it -e OPENAI_API_KEY=sk-... -p 8000:8000 hermes-lite:latest
# Gateway on http://localhost:8000
```

## What I verified (Sep 2026)

- Latest stable: **1.98.1** patch for vtable codegen. [releases.rs](https://releases.rs/docs/1.98.1/)
- Edition 2024 is the current edition; do not stay on 2021 for new code.
- Use `Ipv4Addr::is_private` / explicit v6 checks — not a guessed `IpAddr::is_private` helper.
- After `Child::try_wait` reaps a process, read pipes yourself. Do **not** call `wait_with_output` on the same child.
- `serde_yaml` 0.9 still deserializes `config.yaml` but is deprecated on crates.io. Config load stays YAML for compatibility; a later swap is `serde-saphyr` once you freeze the API.
- HTTP client is **ureq 2.12** (blocking, no Tokio required for the CLI loop).

## Build / test

```bash
rustup show          # should print 1.98.1 from rust-toolchain.toml
make ci              # fmt-check, lint, test, build
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release

export OPENAI_API_KEY=sk-...   # or OPENROUTER_API_KEY
./target/release/hermes-lite chat
./target/release/hermes-lite run "Remember I prefer Rust"
./target/release/hermes-lite gateway --bind 127.0.0.1:8000
./target/release/hermes-lite mcp    # JSON-RPC on stdin
```

## Fixes in this pass

| Bug | Fix |
|---|---|
| Path jail used `canonicalize` on missing files | Component walk; reject `..` / absolute / prefix |
| Sandbox double-wait | `try_wait` then read pipes; `kill` on timeout |
| SSRF | `Ipv4Addr::is_private`, mapped v6, unique-local, DNS re-check |
| Missing MCP / jobs modules | `src/mcp.rs`, `src/jobs.rs` compile and are wired |
| Edition 2021 | Edition 2024 + `rust-version = "1.85"` |

## Still not a 1:1 Python port

Telegram/Discord bots and the FastAPI dashboard are not in this crate. The Rust binary covers the agent loop, SQLite memory, sandbox, SSRF fetch, learner, blocking HTTP gateway, and MCP-shaped stdio.

## License

MIT
