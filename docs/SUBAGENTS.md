# Sub-Agents in Hermes-Lite

## Overview

Hermes-Lite supports **parallel sub-agent delegation** for multi-step tasks. Each sub-agent:
- Runs in a background thread
- Has isolated context
- Executes tasks independently
- Returns results to the orchestrator

## Usage

### CLI Commands

```bash
# Spawn a sub-agent
./target/release/hermes-lite agents spawn "Research Rust async patterns"

# List all sub-agents
./target/release/hermes-lite agents list

# Get result of specific sub-agent
./target/release/hermes-lite agents get <task_id>
```

### Example Workflow

```bash
# Spawn multiple sub-agents for parallel research
./target/release/hermes-lite agents spawn "Research Rust async/await"
./target/release/hermes-lite agents spawn "Research Rust error handling"
./target/release/hermes-lite agents spawn "Research Rust memory safety"

# Check status
./target/release/hermes-lite agents list
# <id1>: completed - Research Rust async/await
# <id2>: running - Research Rust error handling
# <id3>: pending - Research Rust memory safety

# Get results
./target/release/hermes-lite agents get <id1>
```

### Python API

```python
from hermes_lite import SubAgentPool

pool = SubAgentPool()

# Spawn sub-agents
task1 = pool.spawn("Research Rust async patterns")
task2 = pool.spawn("Research Rust error handling")

# Check status
for task in pool.status():
    print(f"{task.id}: {task.status} - {task.description}")

# Get results
for task in pool.status():
    if task.status == "completed":
        print(f"Result: {task.result}")
```

## Architecture

```
Main Agent (Orchestrator)
    ├── Sub-Agent 1 (Research)
    ├── Sub-Agent 2 (Implementation)
    └── Sub-Agent 3 (Review)
```

Each sub-agent:
- Has its own `Store` connection (SQLite)
- Shares the same `Config` and `Skills`
- Runs in a background thread via `std::thread::spawn`
- Returns results via `Arc<Mutex<Vec<SubAgentTask>>>`

## Use Cases

### 1. Parallel Research

```bash
# Spawn 3 sub-agents to research different topics
./target/release/hermes-lite agents spawn "Research async/await in Rust"
./target/release/hermes-lite agents spawn "Research error handling patterns"
./target/release/hermes-lite agents spawn "Research memory safety guarantees"

# Merge results manually or in orchestrator
```

### 2. Multi-Step Workflow

```bash
# Step 1: Research
./target/release/hermes-lite agents spawn "Research best practices for X"

# Step 2: Implementation (after research completes)
./target/release/hermes-lite agents spawn "Implement X based on research"

# Step 3: Review
./target/release/hermes-lite agents spawn "Review implementation for issues"
```

### 3. Specialized Roles

```bash
# Researcher sub-agent
./target/release/hermes-lite agents spawn "[Researcher] Find information about Y"

# Implementer sub-agent
./target/release/hermes-lite agents spawn "[Implementer] Write code for Z"

# Reviewer sub-agent
./target/release/hermes-lite agents spawn "[Reviewer] Check for bugs and improvements"
```

## Limitations

| Limitation | Impact | Workaround |
|---|---|---|
| Shared model | All sub-agents use same LLM | Use different `HERMES_MODEL` env var per spawn |
| No result auto-merge | Must manually merge results | Add orchestrator logic to merge |
| No task dependencies | Tasks run independently | Use sequential spawning with delays |
| No priority queue | FIFO execution | Implement priority in `agents.rs` |

## Comparison with Hermes Agent

| Feature | Hermes Agent | Hermes-Lite |
|---|---|---|
| Sub-agent isolation | Separate contexts + models | Separate contexts, shared model |
| Result merging | Automatic | Manual |
| Task dependencies | Yes (DAG) | No (parallel only) |
| Priority queue | Yes | No |
| Resource limits | Per-agent | Shared |

**Verdict:** Hermes-Lite sub-agents are **simpler but functional** for parallel tasks. For complex workflows with dependencies, use sequential spawning or add DAG support.

## Best Practices

1. **Keep tasks independent** — Sub-agents work best for parallel, independent work
2. **Use descriptive names** — Prefix tasks with role: `[Researcher] ...`, `[Implementer] ...`
3. **Monitor status** — Check `agents list` frequently for long-running tasks
4. **Merge results** — Aggregate sub-agent outputs in orchestrator
5. **Limit concurrency** — Don't spawn too many sub-agents (RAM/CPU limits)

## Example: Full Workflow

```bash
# 1. Spawn research sub-agents
./target/release/hermes-lite agents spawn "[Research] Find Rust async best practices"
./target/release/hermes-lite agents spawn "[Research] Find Rust error handling patterns"

# 2. Wait for completion (poll)
./target/release/hermes-lite agents list

# 3. Spawn implementation sub-agent
./target/release/hermes-lite agents spawn "[Implement] Write async code based on research"

# 4. Spawn review sub-agent
./target/release/hermes-lite agents spawn "[Review] Check implementation for issues"

# 5. Get all results
./target/release/hermes-lite agents list
```

## Code Reference

- `src/agents.rs` — Sub-agent pool implementation
- `src/main.rs` — CLI commands (`agents spawn/list/get`)
- `hermes_lite.py` — Python wrapper (`SubAgentPool`)
