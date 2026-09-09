# Security Policy

## Hardening Measures

### Code Level
- **Secrets:** API keys wrapped in `secrecy::Secret` to prevent accidental logging.
- **Input Validation:** Path traversal, control characters, and length limits enforced.
- **Rate Limiting:** Gateway protected against DoS (60 req/min per IP).
- **Panic Handling:** Custom panic hook logs crashes without leaking stack data.
- **Structured Logging:** JSON logs for audit trails (enable with `RUST_LOG_JSON=1`).

### Container Level
- **Distroless:** No shell, no package manager, minimal attack surface.
- **Non-root:** Runs as `nonroot` user.
- **Read-only filesystem:** Root FS is immutable (`--read-only`).
- **Capabilities:** Drop all (`--cap-drop=ALL`).
- **Tmpfs:** `/tmp` mounted as tmpfs for ephemeral writes.

### Deployment
```bash
docker run --rm -it \
  --read-only \
  --cap-drop=ALL \
  --tmpfs /tmp \
  -e OPENAI_API_KEY=sk-... \
  -p 8000:8000 \
  hermes-lite:latest
```

## Reporting Vulnerabilities

Report security issues via GitHub Private Vulnerability Reporting.

## Audit

Run `cargo audit` to check for known vulnerable dependencies.
