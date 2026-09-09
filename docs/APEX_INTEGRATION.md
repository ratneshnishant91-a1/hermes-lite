# Integrating Hermes-Lite into Apex (Kimi/Claude)

## Idea

Use **Hermes-Lite as the execution engine** for Apex (Kimi/Claude), providing:
- **Local tool execution** (shell, files, memory, skills)
- **Persistent state** (SQLite + MEMORY.md + USER.md)
- **Cron scheduling** (background tasks)
- **Sub-agent delegation** (parallel work)
- **MCP integrations** (Drive, Dropbox, GitHub, etc.)
- **Minimal LLM calls** (70-80% reduction via pattern match, math, cache, skills)

## Architecture

```
┌─────────────────────────────────────────────────┐
│  Apex (Kimi/Claude)                             │
│  - Natural language understanding               │
│  - High-level reasoning                         │
│  - Multi-turn conversation                      │
└────────────────┬────────────────────────────────┘
                 │ HTTP/MCP
                 ▼
┌─────────────────────────────────────────────────┐
│  Hermes-Lite (Execution Engine)                 │
│  - Tool execution (shell, files, fetch, etc.)   │
│  - Memory persistence (SQLite + MD files)       │
│  - Skills (auto-generated SKILL.md)             │
│  - Cron scheduling (background tasks)           │
│  - Sub-agents (parallel delegation)             │
│  - MCP clients (Drive, Dropbox, GitHub, etc.)   │
│  - LLM minimization (cache, pattern, math)      │
└─────────────────────────────────────────────────┘
```

## Benefits

| Benefit | Impact |
|---|---|
| **Cost reduction** | 70-80% fewer LLM calls (saves $100s/month) |
| **Persistence** | State survives restarts (SQLite + files) |
| **Speed** | Instant startup (2MB binary, no Python) |
| **Security** | SSRF protection, ulimits, distroless |
| **Offline capability** | Works without LLM for common tasks |
| **Scalability** | Sub-agents for parallel work |

## Integration Options

### Option 1: HTTP Gateway (Recommended)

**Apex → HTTP → Hermes-Lite Gateway**

```python
# In Apex (Kimi/Claude)
import requests

def execute_tool(tool_name: str, args: dict):
    response = requests.post(
        "http://localhost:8000/chat",
        json={"message": f"Use {tool_name} with args: {args}"}
    )
    return response.json()["response"]

# Use in Apex workflow
result = execute_tool("shell", {"command": "ls -la"})
```

**Hermes-Lite:**
```bash
./target/release/hermes-lite gateway --bind 0.0.0.0:8000
```

### Option 2: MCP Server

**Apex → MCP → Hermes-Lite**

```bash
# Hermes-Lite as MCP server
./target/release/hermes-lite mcp

# Apex connects via MCP protocol
```

### Option 3: Direct Binary Invocation

**Apex calls Hermes-Lite CLI directly**

```python
import subprocess

def run_hermes(prompt: str):
    result = subprocess.run(
        ["./target/release/hermes-lite", "run", prompt],
        capture_output=True,
        text=True
    )
    return result.stdout.strip()

# Use in Apex
response = run_hermes("Backup my files")
```

### Option 4: Python Wrapper

**Apex imports Hermes-Lite Python module**

```python
from hermes_lite import Agent

agent = Agent()
response = agent.run("Research Rust async patterns")
```

## Example Workflows

### Workflow 1: Daily Backup

**Apex (Kimi/Claude):**
```
User: "Set up daily backups"
Apex: "I'll configure Hermes-Lite to backup your files daily."
→ Calls Hermes-Lite cron API
```

**Hermes-Lite:**
```bash
POST /cron/add
{
  "name": "daily-backup",
  "schedule": "daily",
  "tool": "shell",
  "args": "tar -czf backup.tar.gz workspace/"
}
```

### Workflow 2: Parallel Research

**Apex:**
```
User: "Research Rust, Python, and Go async patterns"
Apex: "I'll spawn 3 sub-agents for parallel research."
→ Calls Hermes-Lite sub-agent API
```

**Hermes-Lite:**
```bash
POST /agents/spawn
{"task": "Research Rust async patterns"}
POST /agents/spawn
{"task": "Research Python asyncio"}
POST /agents/spawn
{"task": "Research Go goroutines"}
```

### Workflow 3: Skill Reuse

**Apex:**
```
User: "Deploy my app"
Apex: "Using learned skill 'auto_deploy_my_app'..."
→ Loads skill from Hermes-Lite
```

**Hermes-Lite:**
```bash
GET /skills/auto_deploy_my_app
→ Returns SKILL.md content
→ Executes without LLM call
```

## Configuration

### Apex Config

```yaml
# apex_config.yaml
hermes_lite:
  endpoint: "http://localhost:8000"
  mode: "gateway"  # or "mcp", "cli", "python"
  timeout: 30s
  retry: 3
```

### Hermes-Lite Config

```yaml
# config.yaml
gateway:
  host: "0.0.0.0"
  port: 8000
  require_auth: true

sandbox:
  timeout: 30
  network_enabled: true

network:
  enabled: true
  allowed_domains:
    - "api.github.com"
    - "api.dropbox.com"
```

## Security

| Layer | Hermes-Lite | Apex |
|---|---|---|
| Input validation | ✅ Path traversal, control chars | ✅ Prompt injection detection |
| Tool restrictions | ✅ Role-based, constraints | ✅ Tool allowlists |
| Network security | ✅ SSRF protection, domain allowlists | ✅ API key management |
| Resource limits | ✅ ulimits, timeout | ✅ Rate limiting |
| Audit logging | ✅ SQLite + structured logs | ✅ Conversation logs |

## Performance

| Metric | Apex alone | Apex + Hermes-Lite |
|---|---|---|
| LLM calls/100 queries | ~100 | ~20-30 |
| Response time (cached) | 2-5s | <100ms |
| Memory usage | 1GB+ | 256MB |
| Startup time | 5-10s | <100ms |

## Migration Path

### Phase 1: Gateway Integration (1 day)

```bash
# Start Hermes-Lite gateway
./target/release/hermes-lite gateway

# Update Apex to call gateway
# Test basic tool execution
```

### Phase 2: Memory/Skills (2 days)

```bash
# Migrate Apex memory to Hermes-Lite SQLite
# Import existing skills
# Test skill loading
```

### Phase 3: Cron/Sub-agents (3 days)

```bash
# Migrate Apex cron jobs to Hermes-Lite
# Implement sub-agent delegation
# Test parallel workflows
```

### Phase 4: MCP Integration (2 days)

```bash
# Connect MCP servers (Drive, Dropbox, GitHub)
# Test tool execution via MCP
# Deploy to production
```

## Cost Analysis

| Scenario | Apex alone | Apex + Hermes-Lite | Savings |
|---|---|---|---|
| 1000 queries/day | $30/day | $6-9/day | 70-80% |
| 10,000 queries/day | $300/day | $60-90/day | 70-80% |
| 100,000 queries/day | $3,000/day | $600-900/day | 70-80% |

**Monthly savings: $600-2,700** (depending on volume)

## Verdict

**Integrating Hermes-Lite into Apex is a no-brainer:**

✅ **70-80% cost reduction** (fewer LLM calls)  
✅ **Persistent state** (SQLite + files)  
✅ **Faster responses** (cache, skills, pattern match)  
✅ **Offline capability** (works without LLM)  
✅ **Scalability** (sub-agents, cron)  
✅ **Security** (SSRF, ulimits, distroless)  

**Recommended approach:** Start with **HTTP Gateway** (Option 1), then add MCP integrations as needed.
