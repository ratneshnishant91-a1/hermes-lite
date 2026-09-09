# Hermes-Lite v2.0 — Rust Core

The agent **core is now Rust**. Python files remain as a prototype archive; new work should go through the Rust crate.

Rust is the better core here because the loop does a lot of tool I/O, process isolation, and long-running state. You get memory safety without a GC pause, cheap threads, and a single static binary you can ship.

## Why Rust for this core

- The conversation loop, sandbox, and SQLite store are performance- and safety-critical.
- Process env filtering and path jails are easier to audit when they are not duck-typed.
- `cargo build --release` produces one binary: no `venv`, no interpreter.
- Python is still useful later as a skill/runtime plugin, not as the kernel.

## Build

```bash
# needs Rust 1.74+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cd hermes-lite
cargo build --release

export OPENAI_API_KEY=sk-...   # or OPENROUTER_API_KEY
./target/release/hermes-lite chat
```

## Commands

```bash
hermes-lite chat                 # REPL
hermes-lite run "hello"         # one shot
hermes-lite stats               # learning stats
hermes-lite gateway --bind 127.0.0.1:8000
```

Gateway:

```bash
curl http://127.0.0.1:8000/health
curl -X POST http://127.0.0.1:8000/chat \
  -H 'Content-Type: application/json' \
  -d '{"message":"Remember that I prefer Rust"}'
```

## Architecture (unchanged)

```
User → Agent loop → Context → Model
                       ↓
                 tool? → approval → sandbox/workspace → result → Context
                       ↓
                 Self-learner (skills + memory)
```

## Rust modules

| Module | Role |
|---|---|
| `agent` | 20-step tool loop |
| `model` | OpenAI/OpenRouter chat completions |
| `tools` | shell, files, fetch, search, memory, skills |
| `sandbox` | filtered env + timeout |
| `store` | SQLite sessions/messages/memories |
| `learner` | auto SKILL.md + preference memory |
| `network` | SSRF checks |
| `security` | input validation, rate limit |
| `workspace` | path jail |

Python (`*.py`) is **legacy**. The supported entrypoint is `cargo run -- chat`.

## License

MIT
