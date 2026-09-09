# Hermes-Lite v1.5 — Hardened Self-Learning AI Agent

🚀 **Production-hardened autonomous AI agent with self-learning and secure internet access**

## ✨ Key Features

### 🧠 Self-Learning (USP)
- Automatic skill creation from tasks
- Memory consolidation
- Performance feedback loop
- Continuous improvement

### 🌐 Secure Internet Access
- **SSRF protection** - Blocks private/internal IPs
- **Domain allowlists** - Whitelist specific domains
- **Port blocking** - Prevents access to sensitive ports
- **Response size limits** - Prevents DoS
- **Network audit logging** - All requests logged

### 🛡️ Production Hardening
- Input validation, Docker sandboxing
- Credential filtering, rate limiting
- Audit logging, health checks
- Resource quotas

### 🎨 Web Dashboard
- Real-time metrics, security audit viewer
- Learning statistics, health monitoring

## Quick Start

```bash
git clone https://github.com/ratneshnishant91-a1/hermes-lite.git
cd hermes-lite
pip install -r requirements.txt
export OPENAI_API_KEY="sk-..."
python demo.py
```

## Internet Access

### Default: Disabled (Secure)

By default, network access is **disabled** for maximum security:

```yaml
network:
  enabled: false  # Default
```

### Enable Internet Access

```yaml
network:
  enabled: true
  allowed_domains: []  # Empty = all domains (caution!)
  blocked_ports: [22, 23, 25, 53, 135, 139, 445, ...]
  max_response_size: 10485760  # 10MB
  timeout: 30
```

### Domain Allowlist (Recommended)

```yaml
network:
  enabled: true
  allowed_domains:
    - "api.github.com"
    - "raw.githubusercontent.com"
    - "pypi.org"
    - "docs.python.org"
    - "html.duckduckgo.com"
```

### SSRF Protection

Automatically blocks:
- Private IPs (10.x.x.x, 172.16-31.x.x, 192.168.x.x)
- Localhost (127.0.0.1, ::1)
- Internal networks (fc00::/7)

### Usage Example

```python
from network import NetworkConfig, safe_fetch_url

config = NetworkConfig(
    enabled=True,
    allowed_domains=["api.github.com"],
    timeout=30,
)

result = safe_fetch_url("https://api.github.com/repos/torvalds/linux", config)
print(result["content"][:500])
```

## Security Features

### 1. Input Validation
```python
from security import InputValidator
valid, error = InputValidator.validate_user_message(user_input)
```

### 2. Docker Sandboxing
```yaml
sandbox:
  mode: "docker"
  network_enabled: false  # Isolated from network
```

### 3. Rate Limiting
```python
from security import RateLimiter
limiter = RateLimiter(max_requests=100, window_seconds=60)
```

### 4. Audit Logging
```python
from security import AuditLogger
audit = AuditLogger()
audit.log_tool_execution("fetch_url", {"url": "..."}, "user-123")
```

### 5. Health Checks
```python
from health import HealthChecker
health = HealthChecker(config)
readiness = health.check_readiness()
```

## Self-Learning Example

```python
from agent import Agent

agent = Agent()

# Task 1: Create file
agent.run("Create a Python file")
# → Auto-creates skill

# Task 2: Similar task
agent.run("Create another file")
# → Uses learned skill!

# Check learning
stats = agent.get_learning_stats()
print(stats)
```

## Dashboard

```bash
python dashboard.py
# http://127.0.0.1:8080
```

- Real-time metrics
- Security audit logs
- Network access logs
- Learning stats

## Production Deployment

```bash
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
- [ ] Network: disabled or allowlisted
- [ ] Rate limiting configured
- [ ] API auth enabled
- [ ] Audit logging active
- [ ] Health checks monitored
- [ ] Resource quotas set

## Files

- `network.py` - Secure internet access (SSRF protection)
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
