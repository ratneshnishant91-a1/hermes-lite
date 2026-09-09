#!/usr/bin/env python3
"""
Hermes-Lite v1.5 - Self-Learning Demo

This demonstrates the self-learning capability:
1. Complete a task
2. Agent automatically creates a skill
3. Next time, uses the learned skill
"""

from agent import Agent
from store import Store


def main():
    print("🚀 Hermes-Lite v1.5 - Self-Learning Demo\n")

    # Create agent
    agent = Agent()
    print(f"Session: {agent.session_id}\n")

    # Task 1: Create a file
    print("📝 Task 1: Creating a Python file...")
    response = agent.run("Create a Python file called hello.py that prints 'Hello, World!'")
    print(f"Agent: {response}\n")

    # Check learning stats
    stats = agent.get_learning_stats()
    print(f"📊 Learning Stats: {stats}\n")

    # Task 2: Similar task (should use learned skill)
    print("📝 Task 2: Creating another file (should learn from Task 1)...")
    response = agent.run("Create a file called test.py with a simple test function")
    print(f"Agent: {response}\n")

    # Check skills created
    skills = agent.list_learned_skills()
    print(f"📚 Learned Skills: {len(skills)}")
    for skill in skills:
        print(f"  - {skill['name']}: {skill['task'][:50]}...\n")

    # Show learning log
    print("📖 Learning Log:")
    log_path = agent.workspace.files.parent / "learning_log.json"
    if log_path.exists():
        import json
        with log_path.open() as f:
            log = json.load(f)
        print(f"  Total events: {log['total_learning_events']}")
        print(f"  Skills created: {len(log['skills_created'])}")
        print(f"  Memories consolidated: {len(log['memories_consolidated'])}")

    print("\n✅ Demo complete! The agent has learned from experience.")


if __name__ == "__main__":
    main()
