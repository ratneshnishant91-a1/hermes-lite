# Hermes-Lite vs Hermes Agent (Nous Research)

## Architecture comparison

| Dimension | Hermes Agent (Nous) | Hermes-Lite (You) | Gap |
|---|---|---|---|
| **Core loop** | Continuous background loop | Single-turn CLI/gateway | ❌ No daemon mode |
| **Planner/Executor split** | Dedicated planner model | Single model does both | ⚠️ Implicit only |
| **Subagents** | Isolated contexts, separate models | Shared context, same model | ⚠️ Less isolation |
| **Memory layers** | MEMORY.md, USER.md, SQLite FTS5, skills | SQLite + skills | ⚠️ No curated memory files |
| **Tools** | 40+ built-in (fs, web, browser, code, MCP, vision, audio) | 9 tools (shell, files, fetch, search, memory, skills) | ❌ Missing browser, code exec, vision |
| **Skill format** | Reusable SKILL.md with verification | SKILL.md without verification | ⚠️ No artifact verification |
| **Gateways** | Telegram, Discord, Slack, WhatsApp, Signal, cron | REST gateway, MCP stdio | ❌ No messaging apps |
| **Execution backends** | Local, Docker, SSH, Singularity, Modal, Daytona, Vercel | Local subprocess only | ❌ No sandbox backends |
| **Kanban** | Built-in task board for multi-agent workflows | None | ❌ No task tracking UI |
| **Verification** | Artifact-based (paths, URLs, diffs, tests) | Text responses only | ⚠️ No artifact verification |
| **User modeling** | USER.md for preferences | SQLite memories | ⚠️ Less structured |
| **Cron** | Scheduled jobs with memory persistence | Background threads only | ⚠️ No persistence |
| **Multi-agent patterns** | Orchestrator + specialists (research, impl, review, QA) | Single orchestrator | ❌ No role specialization |

## What Hermes-Lite does better

| Area | Hermes-Lite advantage |
|---|---|
| **Resource usage** | 2MB binary, 256MB RAM vs Python + dependencies |
| **LLM minimization** | Pattern match, math, cache, skill routing (70-80% fewer calls) | Not a focus in Hermes |
| **Security hardening** | SSRF protection, ulimits, distroless Docker, seccomp | Standard Python security |
| **Simplicity** | Single binary, no venv, no interpreter | Requires Python 3.10+, FastAPI, uvicorn |
| **Deployment** | Static binary, systemd, Docker | Requires Python runtime |

## Critical gaps to close

### 1. **Planner/Executor split** (High priority)

**Hermes:** Dedicated planner model creates structured steps, executor runs them.

**You:** Single model does both implicitly.

**Port:** Add explicit `Plan` struct, separate planning prompt, executor loop.

### 2. **Artifact verification** (High priority)

**Hermes:** Workers return paths, URLs, diffs, test results—orchestrator verifies.

**You:** Text responses only.

**Port:** Add `Artifact` type (path, url, test_result), verify before accepting.

### 3. **Curated memory files** (Medium priority)

**Hermes:** `MEMORY.md` (facts), `USER.md` (preferences) loaded into prompt.

**You:** SQLite only.

**Port:** Generate `MEMORY.md` and `USER.md` from SQLite on startup.

### 4. **Role specialization** (Medium priority)

**Hermes:** Research, impl, review, QA workers with different toolsets.

**You:** Single agent type.

**Port:** Add `AgentRole` enum, restrict tools per role.

### 5. **Gateway integrations** (Low priority)

**Hermes:** Telegram, Discord, Slack, WhatsApp, Signal.

**You:** REST + MCP only.

**Port:** Add Telegram bot (already in Python version).

### 6. **Execution backends** (Low priority)

**Hermes:** Docker, SSH, Modal, etc.

**You:** Local subprocess only.

**Port:** Add Docker backend for tool execution.

## Action plan

1. **Planner/Executor split** → Add `Plan` struct, separate planning phase
2. **Artifact verification** → Add `Artifact` type, verify file paths/URLs
3. **Curated memories** → Generate `MEMORY.md` + `USER.md` from SQLite
4. **Role specialization** → Add `AgentRole` with tool restrictions
5. **Telegram gateway** → Port from Python version

## Score

| Category | Hermes Agent | Hermes-Lite |
|---|---|---|
| Features | 9/10 | 5/10 |
| Performance | 5/10 | 9/10 |
| Security | 6/10 | 9/10 |
| Simplicity | 4/10 | 9/10 |
| **Overall** | **6/10** | **8/10** (for your use case) |

**Verdict:** Hermes-Lite is leaner, faster, and more secure—but missing advanced agentic features. Focus on planner/executor split and artifact verification to close the gap.
