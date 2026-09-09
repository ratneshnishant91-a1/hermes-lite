from __future__ import annotations

import os
import sys
import subprocess
from datetime import datetime, timezone
from pathlib import Path


class HealthChecker:
    """
    System health checks for liveness and readiness probes.
    """

    def __init__(self, config):
        self.config = config
        self.start_time = datetime.now(timezone.utc)
        self.last_health_check = None
        self.health_status = "unknown"

    def check_liveness(self) -> dict:
        """
        Check if service is alive.
        """
        return {
            "status": "alive",
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "uptime_seconds": (datetime.now(timezone.utc) - self.start_time).total_seconds(),
            "pid": os.getpid(),
        }

    def check_readiness(self) -> dict:
        """
        Check if service is ready to handle requests.
        """
        checks = {}
        all_healthy = True

        # Check database
        try:
            from store import Store
            store = Store()
            store.db.execute("SELECT 1")
            checks["database"] = {"status": "healthy", "message": "Database accessible"}
        except Exception as e:
            checks["database"] = {"status": "unhealthy", "message": str(e)}
            all_healthy = False

        # Check workspace
        try:
            from workspace import Workspace
            workspace = Workspace()
            if workspace.files.exists() and workspace.files.is_dir():
                checks["workspace"] = {"status": "healthy", "message": "Workspace accessible"}
            else:
                checks["workspace"] = {"status": "unhealthy", "message": "Workspace not found"}
                all_healthy = False
        except Exception as e:
            checks["workspace"] = {"status": "unhealthy", "message": str(e)}
            all_healthy = False

        # Check Docker (if configured)
        if self.config.sandbox.mode == "docker":
            try:
                result = subprocess.run(
                    ["docker", "ps"],
                    capture_output=True,
                    text=True,
                    timeout=5,
                )
                if result.returncode == 0:
                    checks["docker"] = {"status": "healthy", "message": "Docker daemon running"}
                else:
                    checks["docker"] = {"status": "unhealthy", "message": "Docker daemon error"}
                    all_healthy = False
            except Exception as e:
                checks["docker"] = {"status": "unhealthy", "message": str(e)}
                all_healthy = False

        # Check API keys
        if not os.getenv("OPENAI_API_KEY") and not os.getenv("OPENROUTER_API_KEY"):
            checks["api_keys"] = {"status": "warning", "message": "No API keys configured"}
        else:
            checks["api_keys"] = {"status": "healthy", "message": "API keys configured"}

        self.last_health_check = datetime.now(timezone.utc)
        self.health_status = "healthy" if all_healthy else "degraded"

        return {
            "status": self.health_status,
            "timestamp": self.last_health_check.isoformat(),
            "checks": checks,
            "all_healthy": all_healthy,
        }

    def get_metrics(self) -> dict:
        """
        Get system metrics.
        """
        import psutil

        return {
            "cpu_percent": psutil.cpu_percent(interval=1),
            "memory_percent": psutil.virtual_memory().percent,
            "disk_percent": psutil.disk_usage(str(Path.cwd())).percent,
            "process_count": len(psutil.pids()),
            "uptime_seconds": (datetime.now(timezone.utc) - self.start_time).total_seconds(),
        }
