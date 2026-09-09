from __future__ import annotations

import threading
import time
import json
from datetime import datetime, timezone, timedelta
from dataclasses import dataclass, field
from typing import Callable


@dataclass
class CronJob:
    id: str
    name: str
    schedule: str
    tool_name: str
    arguments: dict
    enabled: bool = True
    last_run: str = ""
    next_run: str = ""
    run_count: int = 0


def parse_schedule(schedule: str) -> int:
    """Parse simple schedule strings into seconds until next run."""
    import re
    schedule = schedule.lower().strip()

    # "every X minutes"
    match = re.match(r"every\s+(\d+)\s*m(?:in)?", schedule)
    if match:
        return int(match.group(1)) * 60

    # "every X hours"
    match = re.match(r"every\s+(\d+)\s*h", schedule)
    if match:
        return int(match.group(1)) * 3600

    # "at HH:MM"
    match = re.match(r"at\s+(\d{1,2}):(\d{2})", schedule)
    if match:
        hour, minute = int(match.group(1)), int(match.group(2))
        now = datetime.now(timezone.utc)
        target = now.replace(hour=hour, minute=minute, second=0, microsecond=0)
        if target <= now:
            target += timedelta(days=1)
        return int((target - now).total_seconds())

    # "daily"
    if schedule == "daily":
        now = datetime.now(timezone.utc)
        target = now.replace(hour=0, minute=0, second=0, microsecond=0) + timedelta(days=1)
        return int((target - now).total_seconds())

    raise ValueError(f"Unknown schedule format: {schedule}")


class CronScheduler:
    """Simple cron scheduler for recurring tasks."""

    def __init__(self, store, tool_execute_fn: Callable):
        self.store = store
        self.execute_tool = tool_execute_fn
        self.jobs: dict[str, CronJob] = {}
        self.running = False
        self.thread: threading.Thread | None = None
        self.lock = threading.Lock()
        self._load_jobs()

    def _load_jobs(self):
        """Load jobs from memory."""
        jobs_memory = self.store.search_memories("cron_job:", limit=100)
        for content in jobs_memory:
            try:
                data = json.loads(content.replace("cron_job:", ""))
                job = CronJob(**data)
                self.jobs[job.id] = job
            except:
                pass

    def _save_job(self, job: CronJob):
        """Persist job to memory."""
        self.store.remember(f"cron_job:{json.dumps(job.__dict__)}", source="cron")

    def add_job(self, name: str, schedule: str, tool_name: str, arguments: dict) -> CronJob:
        """Add a new cron job."""
        import uuid
        job = CronJob(id=str(uuid.uuid4()), name=name, schedule=schedule, tool_name=tool_name, arguments=arguments)
        seconds = parse_schedule(schedule)
        job.next_run = (datetime.now(timezone.utc) + timedelta(seconds=seconds)).isoformat()
        with self.lock:
            self.jobs[job.id] = job
            self._save_job(job)
        return job

    def remove_job(self, job_id: str):
        """Remove a job."""
        with self.lock:
            if job_id in self.jobs:
                del self.jobs[job_id]

    def _run_job(self, job: CronJob):
        """Execute a job."""
        try:
            result = self.execute_tool(job.tool_name, job.arguments)
            job.last_run = datetime.now(timezone.utc).isoformat()
            job.run_count += 1
            seconds = parse_schedule(job.schedule)
            job.next_run = (datetime.now(timezone.utc) + timedelta(seconds=seconds)).isoformat()
        except Exception:
            job.last_run = datetime.now(timezone.utc).isoformat()
        self._save_job(job)

    def _scheduler_loop(self):
        """Background scheduler thread."""
        while self.running:
            now = datetime.now(timezone.utc)
            with self.lock:
                for job in list(self.jobs.values()):
                    if not job.enabled:
                        continue
                    next_run = datetime.fromisoformat(job.next_run)
                    if now >= next_run:
                        threading.Thread(target=self._run_job, args=(job,), daemon=True).start()
            time.sleep(30)

    def start(self):
        """Start the scheduler."""
        if self.running:
            return
        self.running = True
        self.thread = threading.Thread(target=self._scheduler_loop, daemon=True)
        self.thread.start()

    def stop(self):
        """Stop the scheduler."""
        self.running = False
        if self.thread:
            self.thread.join(timeout=5)

    def list_jobs(self) -> list[dict]:
        """List all jobs."""
        with self.lock:
            return [
                {"id": job.id, "name": job.name, "schedule": job.schedule, "tool": job.tool_name, "enabled": job.enabled, "last_run": job.last_run, "next_run": job.next_run, "run_count": job.run_count}
                for job in self.jobs.values()
            ]
