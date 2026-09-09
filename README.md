# Hermes-Lite v1.0 — Production-Ready AI Agent

Complete, secure, production-ready autonomous AI agent.

## Features

- ✅ Sessions & Memory (SQLite persistence)
- ✅ Web browsing & search
- ✅ Multi-provider fallback (OpenRouter, OpenAI, Anthropic, Gemini, local)
- ✅ Gateway API (REST, Telegram, Discord)
- ✅ MCP server support
- ✅ Streaming responses (SSE)
- ✅ Docker sandboxing
- ✅ 5-layer authorization
- ✅ Observability & metrics
- ✅ Evaluation suite

## Quick Start

```bash
git clone https://github.com/ratneshnishant91-a1/hermes-lite.git
cd hermes-lite
pip install PyYAML docker
export OPENROUTER_API_KEY="sk-or-..."
python main.py
```

## Security

- Docker isolation (default)
- Credential filtering
- Dangerous command approval (regex patterns)
- 5-layer user authorization (deny-by-default)
- API authentication required
- Network disabled in sandbox

See SECURITY.md for details.

## API

```bash
curl http://127.0.0.1:8000/token -H "Content-Type: application/json" -d '{"user_id":"default"}'
curl http://127.0.0.1:8000/chat -H "Authorization: Bearer TOKEN" -H "Content-Type: application/json" -d '{"message":"Hello"}'
```

## License

MIT
