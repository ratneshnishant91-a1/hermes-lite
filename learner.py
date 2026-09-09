from __future__ import annotations

import json
from datetime import datetime, timezone
from typing import Any
from pathlib import Path


class SelfLearner:
    """
    Self-learning module for Hermes-Lite.
    Automatically creates skills, consolidates memories, and improves from experience.
    
    Following real Hermes Agent's self-improving architecture.
    """

    def __init__(self, store, skills, memory, workspace):
        self.store = store
        self.skills = skills
        self.memory = memory
        self.workspace = workspace
        self.learning_log = workspace.files.parent / "learning_log.json"
        self._load_log()

    def _load_log(self):
        """Load learning history."""
        if self.learning_log.exists():
            with self.learning_log.open("r") as f:
                self.log = json.load(f)
        else:
            self.log = {
                "skills_created": [],
                "memories_consolidated": [],
                "lessons_learned": [],
                "total_learning_events": 0,
            }

    def _save_log(self):
        """Save learning history."""
        with self.learning_log.open("w") as f:
            json.dump(self.log, f, indent=2)

    def learn_from_task(self, task_description: str, conversation_messages: list[dict], result: str, success: bool):
        """
        Learn from a completed task.
        
        1. Extract important facts → memory
        2. Identify patterns → skill creation
        3. Record lesson learned
        """
        # 1. Consolidate memory
        self._consolidate_memory(conversation_messages, result)

        # 2. Create skill if task was successful and complex
        if success and len(conversation_messages) > 10:
            self._create_skill_from_task(task_description, conversation_messages, result)

        # 3. Record lesson
        self._record_lesson(task_description, result, success)

        self.log["total_learning_events"] += 1
        self._save_log()

    def _consolidate_memory(self, messages: list[dict], result: str):
        """
        Extract and save important facts from conversation.
        """
        # Extract user preferences, facts, decisions
        facts = []

        for msg in messages:
            content = msg.get("content", "")
            if msg.get("role") == "user":
                # Look for preference statements
                if any(kw in content.lower() for kw in ["i prefer", "i like", "always", "never", "my favorite"]):
                    facts.append(f"User preference: {content[:200]}")

        # Save consolidated facts
        for fact in facts:
            self.memory.remember(fact, source="learning")
            self.log["memories_consolidated"].append({
                "timestamp": datetime.now(timezone.utc).isoformat(),
                "fact": fact,
            })

    def _create_skill_from_task(self, task_description: str, messages: list[dict], result: str):
        """
        Create a new skill from successful task execution.
        """
        # Generate skill name from task
        skill_name = self._generate_skill_name(task_description)
        skill_dir = self.skills.root / skill_name

        # Check if skill already exists
        if skill_dir.exists():
            return

        # Extract workflow from conversation
        workflow = self._extract_workflow(messages, result)

        # Create skill directory
        skill_dir.mkdir(parents=True, exist_ok=True)

        # Write SKILL.md
        skill_content = f"""---
name: {skill_name}
description: Auto-generated skill for {task_description[:100]}
created: {datetime.now(timezone.utc).isoformat()}
---

# {skill_name}

Auto-generated skill from successful task execution.

## Workflow

{workflow}

## Notes

- This skill was automatically created by the self-learning system
- Review and refine the steps based on actual usage
- Add examples and edge cases as needed
"""

        (skill_dir / "SKILL.md").write_text(skill_content, encoding="utf-8")

        self.log["skills_created"].append({
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "name": skill_name,
            "task": task_description[:200],
        })

    def _generate_skill_name(self, task_description: str) -> str:
        """Generate skill name from task description."""
        # Simple: use first few words, sanitized
        words = task_description.lower().split()[:3]
        name = "_".join(w for w in words if w.isalnum())
        return f"auto_{name}"

    def _extract_workflow(self, messages: list[dict], result: str) -> str:
        """
        Extract workflow steps from conversation.
        """
        steps = []

        for i, msg in enumerate(messages):
            if msg.get("role") == "assistant" and msg.get("tool_calls"):
                for call in msg["tool_calls"]:
                    tool_name = call["function"]["name"]
                    args = call["function"]["arguments"]
                    steps.append(f"{len(steps)+1}. Used `{tool_name}` with {args[:100]}")

            elif msg.get("role") == "tool":
                result_preview = str(msg.get("content", ""))[:100]
                steps.append(f"   → Result: {result_preview}")

        if not steps:
            return "No tool calls detected. Manual workflow creation needed."

        return "\n".join(steps)

    def _record_lesson(self, task_description: str, result: str, success: bool):
        """Record lesson learned from task."""
        lesson = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "task": task_description[:200],
            "success": success,
            "result_preview": result[:200] if result else "None",
        }
        self.log["lessons_learned"].append(lesson)

    def get_learning_stats(self) -> dict:
        """Get learning statistics."""
        return {
            "total_events": self.log["total_learning_events"],
            "skills_created": len(self.log["skills_created"]),
            "memories_consolidated": len(self.log["memories_consolidated"]),
            "lessons_learned": len(self.log["lessons_learned"]),
        }

    def list_created_skills(self) -> list[dict]:
        """List all auto-created skills."""
        return self.log["skills_created"]

    def review_skill(self, skill_name: str) -> str | None:
        """Review and potentially improve a skill."""
        skill_path = self.skills.root / skill_name / "SKILL.md"
        if not skill_path.exists():
            return None

        content = skill_path.read_text(encoding="utf-8")

        # Add review timestamp
        review_note = f"\n\n## Reviewed\n\nLast reviewed: {datetime.now(timezone.utc).isoformat()}\n"
        content += review_note

        skill_path.write_text(content, encoding="utf-8")
        return content
