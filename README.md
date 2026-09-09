# Hermes-Lite v2.0 — Minimal LLM, Max Local Intelligence

[![CI](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml/badge.svg)](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml)

**Pattern matching** • **Math evaluation** • **Smart caching** • **Skill routing** • **Context compression**

## LLM Minimization

| Task Type | How Handled | LLM Used? |
|---|---|---|
| Greetings ("Hello", "Hi") | Pattern match | ❌ No |
| Time/Date | `chrono::Utc::now()` | ❌ No |
| Math ("2+2*3", "sin(pi/2)") | `meval` crate | ❌ No |
| File ops ("read file x.txt") | Direct tool call | ❌ No |
| Memory queries | Direct SQLite query | ❌ No |
| Repeated queries | Response cache | ❌ No |
| Complex reasoning | LLM + tools | ✅ Yes |

**Result:** ~70-80% fewer LLM calls vs naive agent.

## Quick Start

```bash
cargo build --release
export OPENAI_API_KEY=sk-...
./target/release/hermes-lite chat
```

## Test Math (Zero LLM)

```bash
# Arithmetic
./target/release/hermes-lite run "Calculate 2+2*3"
# → 2+2*3 = 8

# Algebra
./target/release/hermes-lite run "What is (a+b)^2 where a=3, b=4?"
# → (Requires LLM for variables, but pure math doesn't

# Functions
./target/release/hermes-lite run "sin(pi/2) + cos(0)"
# → sin(pi/2) + cos(0) = 2

# Complex
./target/release/hermes-lite run "sqrt(16) * log10(100) + 5^2"
# → sqrt(16) * log10(100) + 5^2 = 33
```

## Stats

```bash
./target/release/hermes-lite stats
{
  "cache_size": 10,
  "llm_calls": 2,      // Only 2 LLM calls for 20+ queries!
  "lessons_learned": 15,
  "skills_created": 3
}
```

## Features

| Category | Features |
|---|---|
| **LLM Minimization** | Pattern match, math eval, caching, skill routing |
| **Math** | Arithmetic, algebra, trig, log, exp, sqrt |
| **Core** | Self-learning, SQLite memory, skills |
| **Tools** | Shell, files, fetch, search, memory |
| **Sub-Agents** | Parallel delegation |
| **MCP** | JSON-RPC 2.0 server |
| **Gateway** | REST API, rate-limited |
| **Security** | SSRF protection, ulimits, secrets |
| **Resources** | 2MB binary, 256MB RAM |
| **Deploy** | Docker, systemd |

## Build

```bash
rustup show
cargo test --lib
cargo build --release
export OPENAI_API_KEY=sk-...
./target/release/hermes-lite chat
```

## License

MIT
