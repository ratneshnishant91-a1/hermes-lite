# Hermes-Lite v2.0

A small, secure, durable agent runtime written in Rust 2024.

## What this is

Hermes-Lite is a **local-first runtime** that provides:

- **Workspace safety:** all file operations are confined to a configured root; path-escape attempts are rejected.
- **Prompt safety:** basic validation and rate-limiting primitives.
- **Persistent memory:** SQLite-backed key/value store for facts and small notes.
- **Cron jobs:** persistent, interval-based shell tasks with execution tracking.
- **MCP server:** a minimal Model Context Protocol server over stdio exposing `read_file` and `write_file`.
- **CLI interface:** simple commands for running prompts, managing cron jobs, and ticking the scheduler.

This is **not** a general-purpose LLM agent by itself. It is designed to be connected to an external LLM adapter or MCP client that supplies reasoning and tool orchestration.

## Stable vs experimental

**Stable (v2.0 baseline):**

- `hermes-lite run "..."` – local math, memory, and workspace file reads.
- `hermes-lite mcp` – MCP server over stdio with `read_file` and `write_file`.
- `hermes-lite cron add ...` and `hermes-lite tick` – persistent cron jobs and manual tick execution.
- SQLite storage for memory and cron state.
- Workspace path confinement and prompt validation.

**Experimental (not in this baseline):**

- Planner/executor, sub-agents, Telegram integration, HTTP gateway, and advanced networking.
- These will be added later behind optional Cargo features once the core is proven stable.

## Quick start

```bash
# Build
cargo build --release

# Run a local prompt
./target/release/hermes-lite run "add 2 and 3"
./target/release/hermes-lite run "remember hermes is a local agent runtime"
./target/release/hermes-lite run "recall hermes"
./target/release/hermes-lite run "read file config.yaml"

# Start MCP server (stdio)
./target/release/hermes-lite mcp

# Add a cron job (every 5 minutes, echo timestamp to a file)
./target/release/hermes-lite cron add "log-time" "every 5m" "date >> log.txt"

# Manually tick due cron jobs
./target/release/hermes-lite tick
```

Default configuration is loaded from `config.yaml` in the current directory:

```yaml
workspace_root: workspace
db_path: hermes.db
network:
  enabled: false
  allowed_domains: []
sandbox:
  timeout_secs: 10
```

## Project structure

- `src/agent.rs` – core agent runtime (math, memory, workspace tools).
- `src/config.rs` – configuration model and YAML loader.
- `src/cron.rs` – cron interval parsing and tick execution.
- `src/math.rs` – local math evaluator (natural language + expressions).
- `src/mcp.rs` – minimal MCP server over stdio.
- `src/security.rs` – prompt validation and rate-limiter skeleton.
- `src/store.rs` – SQLite store for memory and cron jobs.
- `src/tools.rs` – sandboxed shell and workspace file tools.
- `src/workspace.rs` – workspace root and path resolution logic.
- `src/main.rs` – CLI entry point.
- `tests/core.rs` – unit/integration tests that do not require an LLM.

## Security model (v2.0)

- **No arbitrary filesystem access:** all file paths are resolved relative to `workspace_root`; attempts to escape (e.g., `../`) are rejected.
- **Sandboxed shell:** `run_shell` blocks obviously dangerous commands and enforces a timeout.
- **Prompt validation:** basic checks for empty input, excessive size, and control characters.
- **No network by default:** `network.enabled` is `false` in the default config.

This runtime is intended to run alongside an LLM adapter that enforces higher-level policies (tool allowlists, approval flows, audit logs).

## Development

```bash
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --locked
```

CI runs these checks on every push/PR to `main` via `.github/workflows/ci.yml`.

## License

MIT
