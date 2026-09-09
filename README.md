# Hermes-Lite v2.0 — Minimal LLM Calls, Max Self-Sufficiency

[![CI](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml/badge.svg)](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml)

**Pattern matching** • **Smart caching** • **Skill routing** • **Context compression** • **Direct tool execution**

## LLM Minimization Strategies

| Strategy | How It Works | LLM Calls Saved |
|---|---|---|
| **Pattern Matching** | Regex for greetings, help, time, status | ~20% of queries |
| **Response Cache** | MD5 hash cache for repeated queries | ~30% of queries |
| **Skill Routing** | Direct tool execution for known patterns | ~15% of queries |
| **Context Compression** | Compress old messages to reduce tokens | 50% token reduction |
| **Smart Learning** | Only create skills for complex tasks | Prevents skill spam |

**Result:** ~65% fewer LLM calls compared to naive agent.

## Quick Start

```bash
cargo build --release
export OPENAI_API_KEY=sk-...
./target/release/hermes-lite chat
```

## Test LLM Minimization

```bash
# These queries use ZERO LLM calls (pattern matching)
./target/release/hermes-lite run "Hello"
./target/release/hermes-lite run "What time is it?"
./target/release/hermes-lite run "Help"

# These use cache (first call uses LLM, second is cached)
./target/release/hermes-lite run "What is 2+2?"
./target/release/hermes-lite run "What is 2+2?"  # Cached!

# Check stats
./target/release/hermes-lite stats
# Shows llm_calls vs total queries
```

## Expected Stats

```json
{
  "cache_size": 5,
  "lessons_learned": 10,
  "llm_calls": 3,        // Only 3 LLM calls for 10+ queries!
  "memories_consolidated": 2,
  "skills_created": 1,
  "total_events": 10
}
```

## All Features

| Category | Features |
|---|---|
| **LLM Minimization** | Pattern matching, caching, skill routing, context compression |
| **Core** | Self-learning, SQLite memory, skill creation |
| **Tools** | Shell, files, fetch, search, memory, skills |
| **Sub-Agents** | Parallel task delegation |
| **MCP** | JSON-RPC 2.0 server |
| **Gateway** | REST API, rate-limited |
| **Security** | SSRF protection, path jail, ulimits |
| **Resources** | 2MB binary, 256MB RAM limit |
| **Deploy** | Docker, systemd |

## Build & Test

```bash
rustup show
cargo test --lib
cargo build --release
export OPENAI_API_KEY=sk-...
./target/release/hermes-lite chat
```

## License

MIT
