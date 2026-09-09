# Hermes-Lite v2.0 — Complete Autonomous AI Platform

🚀 **Production-ready self-learning AI agent with full enterprise features**

## ✨ Complete Feature Set

### 🧠 Self-Learning (USP)
- Automatic skill creation from tasks
- Memory consolidation
- Performance feedback loop
- Continuous improvement

### 🌐 Secure Internet Access
- SSRF protection
- Domain allowlists
- Port blocking
- Response size limits
- Network audit logging

### 🛡️ Production Hardening
- Input validation, Docker sandboxing
- Credential filtering, rate limiting
- Audit logging, health checks
- Resource quotas

### 🎨 Web Dashboard
- Real-time metrics
- Security audit viewer
- Learning statistics
- Health monitoring

### 🚀 Gateway API
- REST API with authentication
- Telegram bot integration
- Discord support (configurable)
- Token-based auth

### 🔌 MCP Server
- Model Context Protocol support
- Compatible with Cursor, Claude Code
- JSON-RPC 2.0 over stdio

### 🤖 Sub-Agent Delegation
- Parallel task execution
- Isolated sub-agents
- Task monitoring

### ⚡ Background Tasks
- Async tool execution
- Non-blocking operations
- Task queue management

### ⏰ Cron Scheduler
- Scheduled tasks
- Recurring jobs
- Persistent scheduling

## Quick Start

```bash
git clone https://github.com/ratneshnishant91-a1/hermes-lite.git
cd hermes-lite
pip install -r requirements.txt
export OPENAI_API_KEY="sk-..."
python main.py
```

## Gateway API

```bash
python main.py --gateway
# http://127.0.0.1:8000

# Create token
curl http://127.0.0.1:8000/token -H "Content-Type: application/json" -d '{"user_id":"default"}'

# Chat
curl http://127.0.0.1:8000/chat -H "Authorization: Bearer TOKEN" -H "Content-Type: application/json" -d '{"message":"Hello"}'
```

## Telegram Bot

```yaml
# config.yaml
gateway:
  telegram_token: "123456:ABC-DEF..."
  telegram_allowed_users: [123456789]
```

## MCP Integration

```bash
python main.py --mcp
# Use with Cursor, Claude Code, etc.
```

## Sub-Agent Delegation

```python
from agent import Agent
agent = Agent()

# Delegate task
result = agent.delegate("Research Python best practices")
print(f"Task {result['task_id']}: {result['status']}")

# Check status
status = agent.get_delegation_status(task_id)
print(status)
```

## Background Tasks

```python
# Execute in background
result = agent.background_execute("shell", {"command": "sleep 10"})
print(f"Task {result['task_id']}: {result['status']}")
```

## Cron Scheduler

```python
# Schedule recurring task
result = agent.add_cron_job("daily-backup", "daily", "shell", {"command": "tar -czf backup.tar.gz workspace/"})
print(f"Scheduled: {result['job_id']} - next: {result['next_run']}")

# List jobs
jobs = agent.list_cron_jobs()
for job in jobs:
    print(f"{job['name']}: {job['schedule']} - next: {job['next_run']}")
```

## Dashboard

```bash
python dashboard.py
# http://127.0.0.1:8080
```

## Docker Deployment

```bash
docker build -t hermes-lite .
docker run -d -p 8000:8000 -p 8080:8080 -e OPENAI_API_KEY=sk-... hermes-lite
```

## Files

- `agent.py` - Self-learning agent
- `gateway.py` - REST API + Telegram
- `mcp_server.py` - MCP server
- `agents.py` - Sub-agent delegation
- `background.py` - Async tasks
- `cron.py` - Cron scheduler
- `sandbox.py` - Docker sandboxing
- `router.py` - Multi-provider router
- `security.py` - Security hardening
- `learner.py` - Self-learning
- `network.py` - Secure internet
- `dashboard.py` - Web UI

## License

MIT

## Acknowledgments

Architecture inspired by [NousResearch/hermes-agent](https://github.com/NousResearch/hermes-agent).
