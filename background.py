from __future__ import annotations

import threading
import time
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any, Callable
from concurrent.futures import ThreadPoolExecutor, Future


@dataclass
class BackgroundTask:
    id: str
    description: str
    status: str = "pending"
    result: Any = None
    error: str = ""
    created_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    started_at: str = ""
    completed_at: str = ""


class BackgroundExecutor:
    """Execute tool calls asynchronously without blocking."""

    def __init__(self, max_workers: int = 3):
        self.executor = ThreadPoolExecutor(max_workers=max_workers)
        self.tasks: dict[str, BackgroundTask] = {}
        self.futures: dict[str, Future] = {}
        self.lock = threading.Lock()

    def submit(self, tool_name: str, arguments: dict, tool_execute_fn: Callable, description: str = "") -> BackgroundTask:
        """Submit a tool call for async execution."""
        import uuid
        task_id = str(uuid.uuid4())
        task = BackgroundTask(id=task_id, description=description or f"{tool_name}({arguments})")

        def run():
            task.status = "running"
            task.started_at = datetime.now(timezone.utc).isoformat()
            try:
                result = tool_execute_fn(tool_name, arguments)
                task.result = result
                task.status = "completed"
            except Exception as e:
                task.error = str(e)
                task.status = "failed"
            finally:
                task.completed_at = datetime.now(timezone.utc).isoformat()
                with self.lock:
                    self.tasks[task_id] = task
            return task

        future = self.executor.submit(run)
        with self.lock:
            self.tasks[task_id] = task
            self.futures[task_id] = future
        return task

    def poll(self, task_id: str) -> BackgroundTask | None:
        """Check task status without blocking."""
        with self.lock:
            return self.tasks.get(task_id)

    def wait(self, task_id: str, timeout: float = 60.0) -> BackgroundTask | None:
        """Block until task completes."""
        with self.lock:
            future = self.futures.get(task_id)
            task = self.tasks.get(task_id)
        if future and task:
            future.result(timeout=timeout)
            return task
        return None

    def list_tasks(self) -> list[BackgroundTask]:
        """Get all tasks (recent first)."""
        with self.lock:
            return sorted(self.tasks.values(), key=lambda t: t.created_at, reverse=True)

    def shutdown(self):
        """Clean shutdown."""
        self.executor.shutdown(wait=True)
