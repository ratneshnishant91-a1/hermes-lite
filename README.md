# Hermes-Lite v1.5 — Self-Learning AI Agent

🚀 **Production-ready autonomous AI agent with self-learning capability**

The key USP of Hermes Agent: **learns from every task** to become smarter over time.

## ✨ Key Features

### 🧠 Self-Learning (USP)
- **Automatic skill creation** from successful tasks
- **Memory consolidation** - extracts important facts from conversations
- **Performance feedback** - learns from successes and failures
- **Skill evolution** - improves skills based on usage patterns

### 🎨 Web Dashboard
- Real-time metrics and analytics
- Learning statistics visualization
- Session and task management
- API token management

### 🛡️ Production Security
- Docker sandboxing (default)
- 5-layer user authorization
- Dangerous command approval (regex patterns)
- Credential filtering
- Network isolation

## Quick Start

```bash
# Clone
git clone https://github.com/ratneshnishant91-a1/hermes-lite.git
cd hermes-lite

# Install
pip install -r requirements.txt

# Set API key
export OPENROUTER_API_KEY="sk-or-..."

# Run self-learning demo
python demo.py

# Run dashboard
python dashboard.py
# Open http://127.0.0.1:8080
```

## Self-Learning Example

```python
from agent import Agent

agent = Agent()

# Task 1: Create a file
agent.run("Create a Python file that prints Hello World")
# → Agent creates skill automatically

# Task 2: Similar task
agent.run("Create a test file with a function")
# → Agent uses learned skill from Task 1!

# Check what was learned
stats = agent.get_learning_stats()
print(stats)
# {'total_events': 2, 'skills_created': 1, ...}

skills = agent.list_learned_skills()
print(skills)
# [{'name': 'auto_create_python', 'task': 'Create a Python file...'}]
```

## How Self-Learning Works

```
Task Completion
    ↓
Extract Workflow (tool calls, results)
    ↓
Create Skill (SKILL.md)
    ↓
Consolidate Memory (important facts)
    ↓
Record Lesson (success/failure)
    ↓
Next Similar Task → Use Learned Skill
```

### Learning Log

Auto-saved to `workspace/learning_log.json`:

```json
{
  "total_learning_events": 5,
  "skills_created": [
    {"name": "auto_create_python", "task": "Create a Python file..."}
  ],
  "memories_consolidated": [
    {"fact": "User preference: I prefer Python over JavaScript"}
  ],
  "lessons_learned": [...]
}
```

## Dashboard

http://127.0.0.1:8080

- 📊 Real-time metrics
- 📝 Activity logs
- 📚 Learned skills
- 🎯 Task success rate

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Web Dashboard |
| `/api/metrics` | GET | Observability metrics |
| `/api/logs` | GET | Recent logs |
| `/api/learning/stats` | GET | Learning statistics |
| `/api/learning/skills` | GET | Auto-created skills |

## Configuration

`config.yaml`:

```yaml
sandbox:
  mode: "docker"  # Docker isolation
  network_enabled: false  # No network

approvals:
  mode: "manual"  # manual, smart, off
  dangerous_patterns: [...]
```

## License

MIT

## Acknowledgments

Self-learning architecture inspired by [NousResearch/hermes-agent](https://github.com/NousResearch/hermes-agent).
