# Hermes-Lite v1.5 — AI Agent with Web Dashboard

Production-ready autonomous AI agent with **Web UI Dashboard**, sessions, memory, skills, web browsing, multi-provider fallback, streaming, MCP, gateway API, and robust sandboxing.

## ✨ What's New in v1.5

- 🎨 **Web Dashboard** — Real-time metrics, logs, session management
- 📊 **Visual Analytics** — Charts for request history, latency, success rate
- 🔐 **API Management** — Token-based auth, user management
- 🖥️ **Task Monitoring** — Background tasks, cron jobs, delegations

## Quick Start

```bash
# Clone
git clone https://github.com/ratneshnishant91-a1/hermes-lite.git
cd hermes-lite

# Install
pip install -r requirements.txt

# Set API key
export OPENROUTER_API_KEY="sk-or-..."

# Run CLI
python -c "from store import Store; print('Core modules loaded')"

# Run Dashboard
python dashboard.py

# Open http://127.0.0.1:8080
```

## Dashboard Features

- **Real-time metrics** — Requests, success rate, latency
- **Activity logs** — Recent queries with provider/model info
- **Session viewer** — Active sessions, message history
- **Task monitor** — Background tasks, cron jobs, delegations
- **API manager** — Create/revoke tokens, manage users

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Web Dashboard |
| `/api/metrics` | GET | Observability metrics |
| `/api/logs` | GET | Recent request logs |
| `/api/chat` | POST | Chat with agent |
| `/api/sessions` | GET | List sessions |
| `/api/tasks` | GET | List background tasks |

## Security

- ✅ Docker sandboxing (default)
- ✅ 5-layer user authorization
- ✅ Dangerous command approval (regex patterns)
- ✅ Credential filtering
- ✅ API authentication (Bearer tokens)
- ✅ Network isolation

## Configuration

Edit `config.yaml`:

```yaml
gateway:
  host: "127.0.0.1"
  port: 8000
  require_auth: true

sandbox:
  mode: "docker"
  network_enabled: false
```

## License

MIT

## Acknowledgments

Architecture inspired by [NousResearch/hermes-agent](https://github.com/NousResearch/hermes-agent).
