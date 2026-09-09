#!/usr/bin/env python3
"""
Hermes-Lite v1.5 - Main Entry Point

Run: python main.py
"""

import sys
import json
import argparse
from config import Config


def main():
    parser = argparse.ArgumentParser(description="Hermes-Lite v1.5")
    parser.add_argument("--config", default="config.yaml", help="Config file")
    parser.add_argument("--demo", action="store_true", help="Run demo")
    parser.add_argument("--test", action="store_true", help="Run tests")
    args = parser.parse_args()

    # Run demo
    if args.demo:
        from demo import main as demo_main
        demo_main()
        return

    # Run tests
    if args.test:
        from test_hermes import main as test_main
        sys.exit(test_main())
        return

    # Interactive CLI
    print("Hermes-Lite v1.5")
    print("=" * 40)
    print("Commands: 'exit' to quit, 'help' for help")
    print()

    # Load config
    config = Config.load(args.config)

    # Check API key
    import os
    if not os.getenv("OPENAI_API_KEY") and not os.getenv("OPENROUTER_API_KEY"):
        print("⚠️  Warning: No API key set. Set OPENAI_API_KEY or OPENROUTER_API_KEY")
        print()

    # Try to import agent
    try:
        from agent import Agent
    except Exception as e:
        print(f"❌ Error loading agent: {e}")
        print("\nMake sure all dependencies are installed:")
        print("  pip install -r requirements.txt")
        sys.exit(1)

    # Create agent
    try:
        agent = Agent(config=config)
        print(f"✅ Agent initialized (session {agent.session_id})")
        print()
    except Exception as e:
        print(f"❌ Error creating agent: {e}")
        sys.exit(1)

    # Interactive loop
    while True:
        try:
            user = input("You: ")
        except (KeyboardInterrupt, EOFError):
            print("\n\nGoodbye!")
            break

        if user.lower() in {"exit", "quit", "q"}:
            print("\nGoodbye!")
            break

        if user.lower() == "help":
            print("\nCommands:")
            print("  exit, quit, q - Exit the agent")
            print("  help - Show this help")
            print("  stats - Show learning stats")
            print("  skills - Show learned skills")
            print()
            continue

        if user.lower() == "stats":
            stats = agent.get_learning_stats()
            print(f"\n📊 Learning Stats:")
            print(f"  Total events: {stats['total_events']}")
            print(f"  Skills created: {stats['skills_created']}")
            print(f"  Memories consolidated: {stats['memories_consolidated']}")
            print(f"  Lessons learned: {stats['lessons_learned']}")
            print()
            continue

        if user.lower() == "skills":
            skills = agent.list_learned_skills()
            print(f"\n📚 Learned Skills ({len(skills)}):")
            for skill in skills:
                print(f"  - {skill['name']}")
                print(f"    Task: {skill['task'][:100]}...")
            print()
            continue

        # Run agent
        try:
            print("\n🤖 Agent: ", end="", flush=True)
            answer = agent.run(user)
            print(answer)
            print()
        except Exception as e:
            print(f"\n❌ Error: {e}")
            print()

    # Cleanup
    agent.shutdown()


if __name__ == "__main__":
    main()
