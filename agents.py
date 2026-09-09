from __future__ import annotations

import threading
import queue
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone


@dataclass
class Task:
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    description: str = ""
    status: str = "pending"
    result: str = ""
    created_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    completed_at: str = ""


class SubAgent:
    """Lightweight sub-agent for task delegation."""

    def __init__(self, parent_agent, allowed_tools: list[str] | None = None):
        self.parent = parent_agent
        self.allowed_tools = allowed_tools
        self.task = Task()
        self.result_queue = queue.Queue()
        self.thread: threading.Thread | None = None

    def _run_task(self, description: str) -> None:
        """Run task in background."""
        try:
            self.task.status = "running"
            # Simple delegation: run as regular agent call
            result = self.parent.run(description)
            self.task.result = result
            self.task.status = "completed"
        except Exception as e:
            self.task.result = f"Error: {e}"
            self.task.status = "failed"
        finally:
            self.task.completed_at = datetime.now(timezone.utc).isoformat()
            self.result_queue.put(self.task)

    def delegate(self, description: str) -> Task:
        """Start sub-agent in background."""
        self.task = Task(description=description)
        self.thread = threading.Thread(target=self._run_task, args=(description,), daemon=True)
        self.thread.start()
        return self.task

    def wait(self, timeout: float = 60.0) -> Task:
        """Block until sub-agent completes."""
        if self.thread:
            self.thread.join(timeout=timeout)
        return self.task

    def poll(self) -> Task:
        """Check status without blocking."""
        try:
            return self.result_queue.get_nowait()
        except queue.Empty:
            return self.task


class AgentPool:
    """Manage multiple sub-agents for parallel task execution."""

    def __init__(self, parent_agent):
        self.parent = parent_agent
        self.agents: dict[str, SubAgent] = {}
        self.tasks: dict[str, Task] = {}

    def spawn(self, task_description: str, allowed_tools: list[str] | None = None) -> SubAgent:
        """Create and start a sub-agent."""
        agent = SubAgent(self.parent, allowed_tools)
        task = agent.delegate(task_description)
        self.agents[agent.task.id] = agent
        self.tasks[agent.task.id] = task
        return agent

    def status(self) -> list[dict]:
        """Get status of all active tasks."""
        return [
            {"id": task.id, "description": task.description, "status": task.status, "result": task.result}
            for task in self.tasks.values()
        ]

    def get_result(self, task_id: str) -> Task | None:
        """Get completed task result."""
        if task_id in self.agents:
            return self.agents[task_id].poll()
        return None
