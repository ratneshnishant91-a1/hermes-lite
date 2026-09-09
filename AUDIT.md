# Agentic Audit — Hermes-Lite v2.0

## Scoring rubric (0–5 each)

1. Autonomy (goal pursuit without hand-holding)
2. Tool use (breadth, reliability, safety)
3. Memory (short-term context + long-term facts/skills)
4. Planning / decomposition (multi-step tasks)
5. Self-improvement (learns from experience)
6. Reflection / error recovery (detects & fixes mistakes)
7. Multi-agent collaboration (delegation, coordination)
8. Human alignment (preferences, constraints, approval)

## Current state (before fixes)

| Dimension | Score | Evidence | Gaps |
|---|---|---|---|
| Autonomy | 3 | Runs tool loops, gateway, cron | No explicit goal stack; stops after one turn |
| Tool use | 4 | 9 tools, SSRF-safe, ulimits | No retry/backoff, no result validation beyond errors |
| Memory | 4 | SQLite memories, skills, cache | No retrieval scoring; skills not auto-applied |
| Planning | 2 | LLM can plan internally | No explicit plan artifact or step tracker |
| Self-improvement | 3 | Auto skills, preference memory | No performance metrics; skills rarely reused |
| Reflection | 1 | None | No self-critique or retry loop |
| Multi-agent | 2 | Sub-agent pool exists | No task routing, no result aggregation |
| Alignment | 3 | Approval for dangerous shell | No explicit user constraints/preferences store |

**Total: 22 / 40 → “proto-agentic”**

## Critical failure modes

- **One-turn myopia:** After answering, the agent doesn’t maintain goals or resume work.
- **No reflection:** If a tool fails or answer is wrong, it doesn’t self-correct.
- **Skills underused:** Skills are created but not proactively matched to new tasks.
- **No plan trace:** Multi-step work relies on LLM internal state; hard to audit or resume.
- **Sub-agents orphaned:** Spawned tasks run independently; no coordinator merges results.
- **Preferences implicit:** User likes are remembered as text, not enforced constraints.

## Fixes to reach 35+ / 40

1. **Goal stack + continuation** (Autonomy +1, Planning +1)
2. **Reflection loop with self-critique** (Reflection +2)
3. **Skill retrieval & auto-apply** (Memory +1, Self-improvement +1)
4. **Plan artifact (explicit steps)** (Planning +1)
5. **Sub-agent coordinator** (Multi-agent +2)
6. **Constraints store + enforcement** (Alignment +1)
7. **Tool retry/backoff + validation** (Tool use +1)

## Target state

| Dimension | Target |
|---|---|
| Autonomy | 4 |
| Tool use | 5 |
| Memory | 5 |
| Planning | 4 |
| Self-improvement | 4 |
| Reflection | 3 |
| Multi-agent | 4 |
| Alignment | 4 |
| **Total** | **33–35 / 40** |

## Test plan (acceptance)

- Run a multi-step task (e.g., “research X, summarize, save report”) and verify:
  - Plan is visible (steps list)
  - Sub-agents can be spawned and results merged
  - On tool error, agent retries or revises plan
  - Learned skills are suggested/applied to similar tasks
  - User preferences constrain behavior (e.g., “prefer Rust”) 
