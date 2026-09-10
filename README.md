# Hermes-Lite v2.5.1

Hermes-Lite is a Rust 2024, local-first agent runtime for deterministic tools, durable state, and safe agent-harness experimentation.

## Run it

```bash
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --locked

./target/release/hermes-lite status
./target/release/hermes-lite run "add 2 and 3"
./target/release/hermes-lite mcp
```

## Core capabilities

- Local math evaluation and SQLite-backed key/value memory.
- Workspace-confined file operations and an MCP stdio server.
- Persistent cron jobs (`cron add`, `cron list`, `cron remove`, `tick`).
- A v2.5.1 durable-learning foundation: append-only JSONL run events, deterministic failure clustering, and versioned harness manifests.

## v2.5.1 learning model

The runtime records evidence before recommending improvements. It does **not** autonomously rewrite source code, relax security policy, or enable network access. The workflow is: record a run, inspect failure clusters, add a regression test, deliberately revise the versioned harness manifest, and roll back on regression.

See [`docs/V2_5_1.md`](docs/V2_5_1.md) for the operating model.

## Security boundary

File operations are confined to `workspace_root`; path traversal is rejected. Network access is disabled by default. External effects should be mediated by an application-level approval policy.
