from __future__ import annotations

import subprocess
import os
import re
from pathlib import Path
from dataclasses import dataclass
from typing import Any


@dataclass
class SandboxResult:
    exit_code: int
    stdout: str
    stderr: str
    timeout: bool = False
    memory_exceeded: bool = False
    command_blocked: bool = False
    block_reason: str = ""


# Safe environment variables to forward
SAFE_ENV_VARS = {
    "PATH", "HOME", "USER", "LANG", "LC_ALL", "TERM", "SHELL",
    "TMPDIR", "TMP", "TEMP", "PWD", "OLDPWD",
}


class Sandbox:
    """
    Sandboxed execution for tool commands.
    Supports subprocess and Docker modes.
    """

    def __init__(self, config):
        self.config = config
        self.mode = config.sandbox.mode if config else "subprocess"

    def _filter_environment(self, extra_env: dict[str, str] | None = None) -> dict[str, str]:
        """Filter environment variables (credential filtering)."""
        filtered = {}

        # Add explicitly forwarded vars
        if self.config and self.config.sandbox.docker_forward_env:
            for var_name in self.config.sandbox.docker_forward_env:
                if var_name in os.environ:
                    filtered[var_name] = os.environ[var_name]

        # Add safe system vars
        for var_name in SAFE_ENV_VARS:
            if var_name in os.environ and var_name not in filtered:
                filtered[var_name] = os.environ[var_name]

        # Add explicitly provided vars
        if extra_env:
            filtered.update(extra_env)

        return filtered

    def execute(
        self,
        command: str,
        cwd: str | None = None,
        env: dict[str, str] | None = None,
        timeout: int | None = None,
    ) -> SandboxResult:
        """Execute command in sandbox."""
        timeout = timeout or (self.config.sandbox.timeout if self.config else 30)

        if self.mode == "docker":
            return self._execute_docker(command, cwd, env, timeout)
        else:
            return self._execute_subprocess(command, cwd, env, timeout)

    def _execute_subprocess(
        self,
        command: str,
        cwd: str | None,
        env: dict[str, str] | None,
        timeout: int,
    ) -> SandboxResult:
        """Execute with subprocess."""
        try:
            exec_env = self._filter_environment(env)

            result = subprocess.run(
                command,
                shell=True,
                capture_output=True,
                text=True,
                timeout=timeout,
                cwd=cwd,
                env=exec_env,
            )

            return SandboxResult(
                exit_code=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
            )

        except subprocess.TimeoutExpired:
            return SandboxResult(
                exit_code=-1,
                stdout="",
                stderr=f"Command timed out after {timeout}s",
                timeout=True,
            )
        except Exception as e:
            return SandboxResult(
                exit_code=-1,
                stdout="",
                stderr=str(e),
            )

    def _execute_docker(
        self,
        command: str,
        cwd: str | None,
        env: dict[str, str] | None,
        timeout: int,
    ) -> SandboxResult:
        """Execute in Docker container."""
        try:
            import docker
            client = docker.from_env()

            # Filter environment
            exec_env = self._filter_environment(env)

            # Prepare container config
            container_config = {
                "image": self.config.sandbox.docker_image if self.config else "python:3.11-slim",
                "command": f"bash -c '{command}'",
                "working_dir": "/workspace",
                "environment": exec_env,
                "mem_limit": f"{self.config.sandbox.memory_limit_mb if self.config else 512}m",
                "cpu_quota": int((self.config.sandbox.cpu_limit if self.config else 1.0) * 100000),
                "network_disabled": not (self.config.sandbox.network_enabled if self.config else False),
                "remove": True,
                "detach": False,
                "security_opt": ["no-new-privileges:true"],
                "cap_drop": ["ALL"],
            }

            # Mount workspace
            volumes = {}
            workspace_path = os.path.join(os.getcwd(), "workspace", "files")
            os.makedirs(workspace_path, exist_ok=True)
            volumes[workspace_path] = {"bind": "/workspace", "mode": "rw"}
            container_config["volumes"] = volumes

            # Run container
            result = client.containers.run(**container_config)

            return SandboxResult(
                exit_code=0,
                stdout=result.decode() if isinstance(result, bytes) else result,
                stderr="",
            )

        except Exception as e:
            return SandboxResult(
                exit_code=-1,
                stdout="",
                stderr=str(e),
            )
