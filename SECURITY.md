# Hermes-Lite v1.5 - Security Hardening Guide

## Defense-in-Depth Layers

### 1. Input Validation ✅
- Strict schema validation for all inputs
- Path traversal prevention
- SQL/Script/Command injection detection
- Length limits on all inputs

### 2. Sandbox Isolation ✅
- Docker containers with hardened settings
- Capability dropping (no-new-privileges)
- Network disabled by default
- CPU/memory limits
- Read-only filesystem where possible

### 3. Credential Filtering ✅
- Whitelist-only environment variables
- Automatic secret redaction
- No API keys in sandboxed processes
- Secure token generation

### 4. Authorization ✅
- 5-layer user authorization (deny-by-default)
- Per-platform allowlists
- API token authentication
- Rate limiting per user

### 5. Audit Logging ✅
- Tamper-evident hash chain
- All security events logged
- Tool execution tracking
- Access attempt logging

### 6. Health Monitoring ✅
- Liveness probes
- Readiness checks
- System metrics (CPU, memory, disk)
- Graceful degradation

## Security Configuration

### Docker Hardening

```yaml
sandbox:
  mode: "docker"
  docker_image: "python:3.11-slim"
  memory_limit_mb: 512
  cpu_limit: 1.0
  network_enabled: false
  security_opt:
    - "no-new-privileges:true"
  cap_drop:
    - ALL
```

### Rate Limiting

```python
from security import RateLimiter

limiter = RateLimiter(max_requests=100, window_seconds=60)

allowed, wait = limiter.is_allowed(user_id)
if not allowed:
    return f"Rate limited. Try in {wait:.1f}s"
```

### Input Validation

```python
from security import InputValidator

valid, error = InputValidator.validate_user_message(user_input)
if not valid:
    raise ValueError(f"Invalid input: {error}")

valid, error = InputValidator.validate_filename(filename)
if not valid:
    raise ValueError(f"Invalid filename: {error}")
```

### Audit Logging

```python
from security import AuditLogger

audit = AuditLogger()

# Log security event
audit.log_security_event("DANGEROUS_COMMAND_BLOCKED", {
    "command": "rm -rf /",
    "user_id": "user-123",
}, user_id="user-123")

# Log tool execution
audit.log_tool_execution("shell", {"command": "ls -la"}, "user-123")

# Log access
audit.log_access("workspace", "read", "user-123", success=True)
```

### Health Checks

```python
from health import HealthChecker

health = HealthChecker(config)

# Liveness probe
liveness = health.check_liveness()
# {"status": "alive", "uptime_seconds": 3600, ...}

# Readiness probe
readiness = health.check_readiness()
# {"status": "healthy", "checks": {"database": {...}, ...}}

# System metrics
metrics = health.get_metrics()
# {"cpu_percent": 15.2, "memory_percent": 45.6, ...}
```

## Security Checklist

Before deploying to production:

- [ ] Enable Docker sandbox mode
- [ ] Disable network in sandbox
- [ ] Set rate limits (max_requests, window_seconds)
- [ ] Configure API authentication
- [ ] Enable audit logging
- [ ] Set up health check monitoring
- [ ] Review dangerous command patterns
- [ ] Test input validation
- [ ] Verify credential filtering
- [ ] Set resource quotas (CPU, memory)

## Threat Model

### Protected Against

✅ Path traversal attacks
✅ Command injection
✅ SQL injection
✅ Script injection
✅ Credential leakage
✅ Resource exhaustion
✅ Unauthorized access
✅ Privilege escalation

### Not Protected Against

❌ Social engineering
❌ Physical access attacks
❌ Compromised dependencies
❌ Zero-day vulnerabilities in Docker
❌ Side-channel attacks

## Incident Response

If security incident detected:

1. **Isolate** - Stop affected containers
2. **Preserve** - Save audit logs
3. **Analyze** - Review tamper-evident logs
4. **Remediate** - Fix vulnerability
5. **Report** - Document incident

## Compliance

- All security events logged with hash chain
- No secrets in logs (automatic redaction)
- Rate limiting prevents DoS
- Input validation prevents injection
- Sandbox prevents host compromise

## References

- [Hermes Agent Security Model](https://github.com/NousResearch/hermes-agent)
- [Docker Security Best Practices](https://docs.docker.com/engine/security/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
