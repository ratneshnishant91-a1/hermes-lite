# Building Hermes-Lite (Rust core)

## Toolchain

This repo pins **rustc 1.98.1** via `rust-toolchain.toml`. Do not override it unless you know what you are doing.

```bash
rustup show
# toolchain: 1.98.1-x86_64-unknown-linux-gnu (default)
```

If `rustup show` does not pick up 1.98.1 automatically:

```bash
rustup install 1.98.1
rustup default 1.98.1
```

## Build / test

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```

Run:

```bash
export OPENAI_API_KEY=sk-...   # or OPENROUTER_API_KEY
./target/release/hermes-lite chat
./target/release/hermes-lite run "Remember I prefer Rust"
./target/release/hermes-lite stats
./target/release/hermes-lite gateway --bind 127.0.0.1:8000
./target/release/hermes-lite mcp
```

## Known gaps vs Python v2.0

Rust crate covers the core loop, SQLite memory, sandbox, SSRF fetch, learner, HTTP gateway, MCP stdio.

Not yet ported to Rust:

- Telegram / Discord bots
- FastAPI dashboard
- Sub-agent pool, background executor, cron (skeleton in `jobs.rs` only)

## Upgrading dependencies

Check versions:

```bash
cargo outdated  # if cargo-outdated is installed
```

Update carefully:

```bash
cargo update -p ureq
# then re-run cargo test && cargo clippy
```

## Notes

- `serde_yaml` 0.9 is deprecated but still used for `config.yaml` compatibility. A future swap to `serde-saphyr` is possible once you freeze the API.
- HTTP client is **ureq 2.12** (blocking). No Tokio required for the CLI loop.
- SSRF checks use `Ipv4Addr::is_private` and explicit v6 rules; not a guessed `IpAddr::is_private` helper.
