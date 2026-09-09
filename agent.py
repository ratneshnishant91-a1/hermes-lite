from __future__ import annotations

import json
import time
from typing import Any

from approval import approve
from context import Context
from memory import Memory
from model import Model
from skills import Skills
from store import Store
from tools import create_default_registry
from workspace import Workspace
from learner import SelfLearner
from compressor import ContextCompressor
from context_selector import ContextSelector


SYSTEM_PROMPT = """
You are Hermes-Lite v1.5, a self-learning autonomous AI agent.

You automatically learn from every task:
- Extract important facts to memory
- Create skills from successful complex tasks
- Record lessons for continuous improvement

Use tools when helpful. Stay in workspace. Be concise.
"""


class Agent:
    def __init__(self, session_id: int | None = None, config=None):
        self.config = config
        self.store = Store()
        self.workspace = Workspace()
        self.skills = Skills()
        self.memory = Memory(self.store)
        self.model = Model()
        self.tools = create_default_registry(self.workspace, self.memory, self.skills)

        self.session_id = session_id or self.store.latest_session() or self.store.create_session()

        prompt = SYSTEM_PROMPT + "\n\n" + self.skills.catalog_text()
        facts = self.memory.search("", limit=8)
        if facts:
            prompt += "\n\nKnown memories:\n" + "\n".join(f"- {f}" for f in facts)

        self.context = Context(prompt, self.store, self.session_id, self.model.generate)
        self.context_selector = ContextSelector(self.memory, self.skills)

        # v1.5: Self-learning
        self.learner = SelfLearner(self.store, self.skills, self.memory, self.workspace)

        # Learning state
        self.current_task = None
        self.task_start_messages = len(self.context.messages)

    def run(self, user_message: str) -> str:
        start_time = time.perf_counter()
        tool_calls = []
        error = ""
        response = ""

        try:
            self.context.add_user(user_message)

            # Track task start
            if not self.current_task:
                self.current_task = {
                    "description": user_message[:200],
                    "start_messages": len(self.context.messages),
                }

            for _ in range(20):
                messages, selection_stats = self.context_selector.select_context(
                    user_message, self.context.prompt_messages()
                )

                response_obj = self.model.generate(messages, self.tools.schemas())

                if not response_obj.tool_calls:
                    response = response_obj.text or ""
                    self.context.add_assistant(response)

                    # Learn from completed task
                    if self.current_task:
                        self._complete_task(response, True)

                    break

                for call in response_obj.tool_calls:
                    name = call["function"]["name"]
                    arguments = json.loads(call["function"]["arguments"] or "{}")
                    tool_calls.append({"name": name, "arguments": arguments})

                    if not approve(name, arguments):
                        result = "Tool execution rejected by user."
                    else:
                        try:
                            result = self.tools.execute(name, arguments)
                        except Exception as e:
                            result = {"error": str(e)}
                            error = str(e)

                    self.context.add_tool_call(call)
                    self.context.add_tool_result(call["id"], name, result)

        except Exception as e:
            error = str(e)
            response = f"Error: {e}"
            if self.current_task:
                self._complete_task(response, False)

        return response

    def _complete_task(self, result: str, success: bool):
        """Mark task complete and learn from it."""
        if not self.current_task:
            return

        # Get conversation messages for this task
        messages = self.context.messages[self.current_task["start_messages"]:]

        # Learn from task
        self.learner.learn_from_task(
            task_description=self.current_task["description"],
            conversation_messages=messages,
            result=result,
            success=success,
        )

        # Reset task tracking
        self.current_task = None
        self.task_start_messages = len(self.context.messages)

    def get_learning_stats(self) -> dict:
        """Get self-learning statistics."""
        return self.learner.get_learning_stats()

    def list_learned_skills(self) -> list[dict]:
        """List auto-created skills."""
        return self.learner.list_created_skills()

    def get_model_stats(self) -> dict:
        """Get model router statistics."""
        return self.model.get_stats() if hasattr(self.model, "get_stats") else {}

    def shutdown(self):
        """Clean shutdown."""
        pass
