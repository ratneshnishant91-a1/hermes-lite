# Hermes-Lite v2.0 — Agentic Rust Core

[![CI](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml/badge.svg)](https://github.com/ratneshnishant91-a1/hermes-lite/actions/workflows/ci.yml)

**Goal-driven** • **Reflective** • **Skill-reusing** • **Minimal LLM** • **Hardened**

## Agentic features (fixed gaps)

| Capability | Implementation | CLI test |
|---|---|---|
| **Goals** | Explicit goal stack with plans & steps | `hermes-lite goals` |
| **Reflection** | Error-injected prompts; plan revision | Run multi-step task |
| **Skill retrieval** | Auto-apply skills before LLM | `hermes-lite skills` |
| **Plans** | Visible step list per goal | `hermes-lite goals` |
| **Sub-agents** | Coordinator + result merge | `hermes-lite agents spawn` |
| **Constraints** | Enforced user preferences (e.g., avoid network) | Edit `config.yaml` |
| **Retry/backoff** | One retry on transient tool errors | Trigger tool error |

## LLM minimization

| Task | Handler | LLM? |
|---|---|---|
| Greetings, time, help | Pattern match | ❌ |
| Math (`add 2 and 3`, `2+2*3`) | Local parser (`meval`) | ❌ |
| Repeated queries | Cache | ❌ |
| File/memory ops | Direct tools | ❌ |
| Skill reuse | Auto-apply | ❌ |
| Complex reasoning | LLM + tools | ✅ |

**Result:** ~75–85% fewer LLM calls.

## Quick start

```bash
cargo build --release
export OPENAI_API_KEY=sk-...
./target/release/hermes-lite chat
```

## Agentic demo

```bash
# Multi-step goal with plan
./target/release/hermes-lite run "Research Rust best practices, summarize, save report"

# View active goals & plans
./target/release/hermes-lite goals

# Spawn sub-agent for parallel work
./target/release/hermes-lite agents spawn "Summarize Rust error handling"

# Check learning & cache
./target/release/hermes-lite stats
```

## Stats example

```json
{
  "active_goals": 1,
  "cache_size": 8,
  "lessons_learned": 12,
  "llm_calls": 3,
  "memories_consolidated": 4,
  "skills_created": 2,
  "total_events": 12
}
```

## Build & test

```bash
rustup show
cargo test --lib
cargo build --release
export OPENAI_API_KEY=sk-...
./target/release/hermes-lite chat
```

## Audit

See `AUDIT.md` for gap analysis and target scores (33–35 / 40).

## License

MIT
