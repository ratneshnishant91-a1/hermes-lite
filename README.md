# Hermes-Lite v1.5 — Hardened Self-Learning AI Agent

🚀 **Production-hardened autonomous AI agent with self-learning capability**

## ✨ Key Features

### 🧠 Self-Learning (USP)
- Automatic skill creation from tasks
- Memory consolidation
- Performance feedback loop
- Continuous improvement

### 🛡️ Production Hardening
- **Input validation** - Prevents injection attacks
- **Docker sandboxing** - OS-level isolation
- **Credential filtering** - No secrets in sandbox
- **Rate limiting** - Prevents abuse
- **Audit logging** - Tamper-evident security logs
- **Health checks** - Liveness/readiness probes
- **Resource quotas** - CPU, memory, disk limits

### 🎨 Web Dashboard
- Real-time metrics
- Security audit viewer
- Learning statistics
- Health monitoring

## Quick Start

```bash
git clone https://github.com/ratneshnishant91-a1/hermes-lite.git
cd hermes-lite
pip install -r requirements.txt
export OPENAI_API_KEY="sk-..."
python demo.py
```

## Security Features

### 1. Input Validation
```python
from security import InputValidator

valid, error = InputValidator.validate_user_message(user_input)
# Prevents: path traversal, SQL injection, XSS, command injection
```

### 2. Docker Sandboxing
```yaml
sandbox:
  mode: "docker"
  network_enabled: false
  cap_drop: ["ALL"]
  security_opt: ["no-new-privileges:true"]
```

### 3. Rate Limiting
```python
from security import RateLimiter

limiter = RateLimiter(max_requests=100, window_seconds=60)
allowed, wait = limiter.is_allowed(user_id)
```

### 4. Audit Logging
```python
from security import AuditLogger

audit = AuditLogger()
audit.log_tool_execution("shell", {"command": "ls"}, "user-123")
# Tamper-evident hash chain
```

### 5. Health Checks
```python
from health import HealthChecker

health = HealthChecker(config)
liveness = health.check_liveness()
readiness = health.check_readiness()
metrics = health.get_metrics()
```

## Self-Learning Example

```python
from agent import Agent

agent = Agent()

# Task 1
agent.run("Create a Python file")
# → Auto-creates skill

# Task 2 (similar)
agent.run("Create another file")
# → Uses learned skill!

# Check learning
stats = agent.get_learning_stats()
print(stats)
# {'total_events': 2, 'skills_created': 1, ...}
```

## Dashboard

```bash
python dashboard.py
# http://127.0.0.1:8080
```

- Real-time metrics
- Security audit logs
- Learning stats
- Health status

## Production Deployment

```bash
# Docker
docker build -t hermes-lite .
docker run -d -p 8080:8080 \
  --memory=512m \
  --cpus=1.0 \
  -e OPENAI_API_KEY=sk-... \
  hermes-lite

# Health check
curl http://localhost:8080/health
```

## Security Checklist

- [ ] Docker sandbox enabled
- [ ] Network disabled in sandbox
- [ ] Rate limiting configured
- [ ] API auth enabled
- [ ] Audit logging active
- [ ] Health checks monitored
- [ ] Resource quotas set
- [ ] Input validation tested

## Files

- `security.py` - Input validation, audit logging, rate limiting
- `health.py` - Liveness/readiness probes
- `learner.py` - Self-learning engine
- `agent.py` - Hardened agent
- `sandbox.py` - Docker isolation
- `dashboard.py` - Web UI

## License

MIT

## Acknowledgments

Security patterns from [Hermes Agent](https://github.com/NousResearch/hermes-agent).
