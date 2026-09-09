# Private Integration: Hermes-Lite → Apex (Internal)

**Note:** Apex is private. This guide is for internal use only.

## What Changes When Apex Uses Hermes-Lite

### Apex's Current Architecture (Private)

```
┌─────────────────────────────────────────┐
│  Apex (Private Agent Framework)         │
│  - Agent system                         │
│  - Connection layer (MCP, APIs)         │
│  - Tool execution                       │
│  - Conversation management              │
└─────────────────────────────────────────┘
```

**Limitations:**
- High LLM dependency (every tool call costs money)
- No persistent memory across sessions
- No skill learning system
- No cron/background tasks
- No offline capability
- Resource heavy (Python + dependencies)

### Apex + Hermes-Lite Architecture

```
┌─────────────────────────────────────────┐
│  Apex (Private)                         │
│  - Natural language understanding       │
│  - Multi-turn conversation              │
│  - Existing agents & connections        │
└────────────────┬────────────────────────┘
                 │ HTTP/MCP/Internal API
                 ▼
┌─────────────────────────────────────────┐
│  Hermes-Lite (Your Execution Engine)    │
│  - LLM minimization (70-80% reduction)  │
│  - Persistent memory (SQLite + files)   │
│  - Skills (auto-generated SKILL.md)     │
│  - Cron scheduling (background tasks)   │
│  - Sub-agents (parallel delegation)     │
│  - Offline mode (works without LLM)     │
│  - Security (SSRF, ulimits, distroless) │
│  - Resource efficiency (2MB, 256MB)     │
└─────────────────────────────────────────┘
```

**Upgrades:**
- ✅ 70-80% fewer LLM calls (massive cost savings)
- ✅ Persistent memory (remembers users)
- ✅ Auto-learning (creates skills from experience)
- ✅ Background tasks (cron scheduling)
- ✅ Parallel work (sub-agents)
- ✅ Offline capability (works without LLM)
- ✅ Production security (SSRF, ulimits)
- ✅ Edge deployment (2MB binary, $5 VPS)

## Integration Points

### 1. Tool Execution Layer

**Current Apex:**
```python
# Every tool call uses LLM
def execute_tool(tool_name: str, args: dict):
    # LLM figures out how to execute
    # Costs $0.002-0.01 per call
    return llm_execute(tool_name, args)
```

**With Hermes-Lite:**
```python
# 70-80% use local execution (no LLM)
def execute_tool(tool_name: str, args: dict):
    # Hermes-Lite handles:
    # - Math (local)
    # - Pattern match (local)
    # - Cache hits (local)
    # - Skills (local)
    # - Only complex tasks use LLM
    return hermes_execute(tool_name, args)
```

**Impact:** 70-80% cost reduction on tool execution

### 2. Memory Layer

**Current Apex:**
```python
# Session-only memory
memory = {}
# Lost when session ends
```

**With Hermes-Lite:**
```python
# Persistent memory (SQLite + files)
# Survives restarts
# Accumulates over time
memory = HermesLiteMemory("user_id")
memory.save_preference("prefers_rust", True)
memory.save_fact("deployed_via_docker", "2026-09-09")
```

**Impact:** Apex remembers users forever

### 3. Skill System

**Current Apex:**
```python
# Every task requires fresh LLM reasoning
def deploy_app():
    # LLM figures out steps every time
    # Slow, expensive
    return llm_reason("How to deploy app?")
```

**With Hermes-Lite:**
```python
# First time: LLM creates skill
# Second time: Uses skill (no LLM)
def deploy_app():
    if skill_exists("auto_deploy_app"):
        return execute_skill("auto_deploy_app")  # $0
    else:
        skill = llm_create_skill("deploy_app")  # $0.01 (one-time)
        save_skill(skill)
        return execute_skill(skill)
```

**Impact:** Apex learns from experience, gets faster/cheaper

### 4. Cron/Background Tasks

**Current Apex:**
```python
# Can't run scheduled tasks
# Requires external cron service
# No persistence
```

**With Hermes-Lite:**
```python
# Built-in cron (persists in SQLite)
hermes.cron_add(
    name="daily_backup",
    schedule="daily",
    tool="shell",
    args="tar -czf backup.tar.gz workspace/"
)
# Runs automatically, even when Apex is offline
```

**Impact:** Apex can work autonomously 24/7

### 5. Sub-Agent Delegation

**Current Apex:**
```python
# Sequential execution
result1 = execute_task("research rust")
result2 = execute_task("research python")
result3 = execute_task("research go")
# Slow for complex workflows
```

**With Hermes-Lite:**
```python
# Parallel execution via sub-agents
task1 = hermes.spawn("research rust")
task2 = hermes.spawn("research python")
task3 = hermes.spawn("research go")
# 3x faster
results = merge([task1.result, task2.result, task3.result])
```

**Impact:** Apex can parallelize complex workflows

## Deployment Options

### Option 1: Sidecar (Recommended)

```bash
# Run Hermes-Lite alongside Apex
./hermes-lite gateway --bind 127.0.0.1:8000

# Apex calls via localhost HTTP
# Low latency, isolated
```

**Pros:**
- Isolated processes
- Easy to scale
- Low latency (localhost)

**Cons:**
- Two processes to manage

### Option 2: Embedded (Python Module)

```python
# Import Hermes-Lite directly in Apex
from hermes_lite import Agent

agent = Agent()
result = agent.run("Backup workspace")
```

**Pros:**
- Single process
- Direct API calls

**Cons:**
- Python dependency
- Less isolation

### Option 3: MCP Server

```bash
# Hermes-Lite as MCP server
./hermes-lite mcp

# Apex connects via MCP protocol
# Tools appear in Apex's tool registry
```

**Pros:**
- Standard protocol
- Auto-discovery of tools

**Cons:**
- MCP overhead
- More complex setup

### Option 4: CLI Subprocess

```python
# Apex spawns Hermes-Lite CLI
import subprocess

def hermes_run(prompt: str):
    return subprocess.run(
        ["./hermes-lite", "run", prompt],
        capture_output=True,
        text=True
    ).stdout.strip()
```

**Pros:**
- Maximum isolation
- No integration code

**Cons:**
- Process spawn overhead
- Less efficient

## Security Model

```
┌─────────────────┐
│     Apex        │
│  (Private)      │
└────────┬────────┘
         │ Authenticated
         ▼
┌─────────────────┐
│  Hermes-Lite    │
│  - SSRF protect │
│  - Path jail    │
│  - ulimits      │
│  - Distroless   │
└─────────────────┘
```

**Layers:**
1. Apex authentication (your existing auth)
2. Hermes-Lite SSRF protection (blocks private IPs)
3. Path jail (no `../` escapes)
4. ulimits (CPU, memory, files)
5. Distroless container (no shell, no package manager)

## Cost Analysis (Internal)

| Metric | Apex alone | Apex + Hermes-Lite | Savings |
|---|---|---|---|
| LLM calls/100 queries | 100 | 20-30 | 70-80% |
| Cost/100 queries | $0.20-1.00 | $0.04-0.30 | 70-80% |
| Monthly cost (10k/day) | $300 | $60-90 | $210-240 |
| Monthly cost (100k/day) | $3,000 | $600-900 | $2,100-2,400 |

**ROI:** Integration pays for itself in <1 month at scale

## Migration Plan (Internal)

### Phase 1: HTTP Gateway (1 week)

```bash
# Deploy Hermes-Lite gateway
./hermes-lite gateway --bind 0.0.0.0:8000

# Update Apex tool execution layer
# Test with simple tools (math, file ops)
```

### Phase 2: Memory/Skills (2 weeks)

```bash
# Migrate Apex memory to Hermes-Lite SQLite
# Implement skill loading in Apex
# Test skill reuse (no LLM)
```

### Phase 3: Cron/Sub-agents (2 weeks)

```bash
# Migrate Apex cron jobs to Hermes-Lite
# Implement sub-agent delegation
# Test parallel workflows
```

### Phase 4: Production (1 week)

```bash
# Deploy to production
# Monitor cost savings
# Tune LLM minimization thresholds
```

**Total: 6 weeks**

## Metrics to Track

| Metric | Before | Target | Measurement |
|---|---|---|---|
| LLM calls/100 queries | 100 | 20-30 | Apex analytics |
| Cost/query | $0.002-0.01 | $0.0004-0.003 | API bills |
| Response time (cached) | 2-5s | <100ms | Apex latency |
| Memory persistence | 0% | 100% | User retention |
| Skill reuse rate | 0% | 50%+ | Skill analytics |
| Cron task success | N/A | 99%+ | Cron logs |
| Sub-agent parallelism | 1x | 3-5x | Workflow timing |

## Success Criteria

✅ **70-80% cost reduction** (measured via API bills)  
✅ **<100ms cached responses** (measured via Apex latency)  
✅ **Persistent memory** (users noticed Apex remembers them)  
✅ **Auto-learning** (skills created from experience)  
✅ **Background tasks** (cron jobs running autonomously)  
✅ **Parallel workflows** (sub-agents for complex tasks)  

## Next Steps (Private)

1. **Deploy Hermes-Lite gateway** in your Apex infrastructure
2. **Update Apex tool execution** to call Hermes-Lite first
3. **Migrate memory** to SQLite + files
4. **Test cost savings** (track LLM call reduction)
5. **Iterate** (add more local optimizations)

**This is your private upgrade path.** Hermes-Lite transforms Apex from a chatbot into a self-sufficient agent.

---

**Private. Internal use only.**
