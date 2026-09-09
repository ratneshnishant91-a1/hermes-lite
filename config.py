from __future__ import annotations

import os
import yaml
from dataclasses import dataclass, field
from typing import Any
from pathlib import Path


@dataclass
class GatewayConfig:
    enabled: bool = False
    host: str = "127.0.0.1"
    port: int = 8000
    telegram_token: str = ""
    telegram_allowed_users: list[int] = field(default_factory=list)
    telegram_allow_all: bool = False
    discord_token: str = ""
    discord_guild_id: str = ""
    discord_allowed_users: list[str] = field(default_factory=list)
    discord_allow_all: bool = False
    allowed_users: list[str] = field(default_factory=list)
    allow_all: bool = False
    require_auth: bool = True


@dataclass
class SandboxConfig:
    mode: str = "docker"
    timeout: int = 30
    memory_limit_mb: int = 512
    cpu_limit: float = 1.0
    network_enabled: bool = False
    workspace_only: bool = True
    docker_image: str = "python:3.11-slim"
    docker_forward_env: list[str] = field(default_factory=list)


@dataclass
class ObservabilityConfig:
    enabled: bool = True
    log_level: str = "INFO"
    log_file: str = "hermes-lite.log"
    metrics_enabled: bool = True
    tracing_enabled: bool = False
    retention_days: int = 7
    redact_secrets: bool = True


@dataclass
class ModelConfig:
    providers: list[dict] = field(default_factory=lambda: [
        {"name": "openrouter", "priority": 1, "enabled": True},
        {"name": "openai", "priority": 2, "enabled": True},
        {"name": "anthropic", "priority": 3, "enabled": True},
        {"name": "gemini", "priority": 4, "enabled": True},
        {"name": "local", "priority": 5, "enabled": False},
    ])
    default_model: str = "gpt-4.1-mini"
    max_tokens: int = 4096
    fallback_providers: list[dict] = field(default_factory=list)


@dataclass
class ApprovalsConfig:
    mode: str = "manual"
    hardline_patterns: list[str] = field(default_factory=lambda: [
        r'\brm\s+-[^\s]*r\s+/',
        r'\bmkfs\b',
        r'\bdd\s+if=',
        r'\b:()\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;',
        r'\bkill\s+-1\s+1\b',
        r'\bshutdown\b.*-h',
        r'\breboot\b',
    ])
    dangerous_patterns: list[str] = field(default_factory=lambda: [
        r'\brm\s+-[^\s]*r',
        r'\bchmod\s+(777|666|a\+w|o\+w)',
        r'\bDROP\s+(TABLE|DATABASE)\b',
        r'\bDELETE\s+FROM\b(?!.*\bWHERE\b)',
        r'\bTRUNCATE\s+(TABLE)?\s*\w',
        r'\b(curl|wget)\b.*\|\s*(ba)?sh\b',
        r'\bbash\s+-c\b',
        r'\bpython\s+-c\b',
        r'\bsudo\b',
        r'\bpasswd\b',
        r'\bvisudo\b',
        r'\bssh\b.*@',
        r'\bscp\b',
        r'\brsync\b.*@',
    ])


@dataclass
class Config:
    gateway: GatewayConfig = field(default_factory=GatewayConfig)
    sandbox: SandboxConfig = field(default_factory=SandboxConfig)
    observability: ObservabilityConfig = field(default_factory=ObservabilityConfig)
    model: ModelConfig = field(default_factory=ModelConfig)
    approvals: ApprovalsConfig = field(default_factory=ApprovalsConfig)
    workspace_root: str = "workspace"
    db_path: str = "hermes.db"
    skills_root: str = "skills"
    mcp_servers: dict[str, dict] = field(default_factory=dict)

    @classmethod
    def load(cls, path: str = "config.yaml") -> "Config":
        config_path = Path(path)
        if not config_path.exists():
            return cls()
        with config_path.open("r") as f:
            data = yaml.safe_load(f) or {}
        if "gateway" in data:
            gateway = GatewayConfig(**data["gateway"])
        else:
            gateway = GatewayConfig()
        if "sandbox" in data:
            sandbox = SandboxConfig(**data["sandbox"])
        else:
            sandbox = SandboxConfig()
        if "observability" in data:
            observability = ObservabilityConfig(**data["observability"])
        else:
            observability = ObservabilityConfig()
        if "model" in data:
            model = ModelConfig(**data["model"])
        else:
            model = ModelConfig()
        if "approvals" in data:
            approvals = ApprovalsConfig(**data["approvals"])
        else:
            approvals = ApprovalsConfig()
        return cls(
            gateway=gateway,
            sandbox=sandbox,
            observability=observability,
            model=model,
            approvals=approvals,
            workspace_root=data.get("workspace_root", "workspace"),
            db_path=data.get("db_path", "hermes.db"),
            skills_root=data.get("skills_root", "skills"),
            mcp_servers=data.get("mcp_servers", {}),
        )

    def save(self, path: str = "config.yaml") -> None:
        data = {
            "gateway": {
                "enabled": self.gateway.enabled,
                "host": self.gateway.host,
                "port": self.gateway.port,
                "telegram_token": self.gateway.telegram_token,
                "telegram_allowed_users": self.gateway.telegram_allowed_users,
                "telegram_allow_all": self.gateway.telegram_allow_all,
                "discord_token": self.gateway.discord_token,
                "discord_guild_id": self.gateway.discord_guild_id,
                "discord_allowed_users": self.gateway.discord_allowed_users,
                "discord_allow_all": self.gateway.discord_allow_all,
                "allowed_users": self.gateway.allowed_users,
                "allow_all": self.gateway.allow_all,
                "require_auth": self.gateway.require_auth,
            },
            "sandbox": {
                "mode": self.sandbox.mode,
                "timeout": self.sandbox.timeout,
                "memory_limit_mb": self.sandbox.memory_limit_mb,
                "cpu_limit": self.sandbox.cpu_limit,
                "network_enabled": self.sandbox.network_enabled,
                "workspace_only": self.sandbox.workspace_only,
                "docker_image": self.sandbox.docker_image,
                "docker_forward_env": self.sandbox.docker_forward_env,
            },
            "observability": {
                "enabled": self.observability.enabled,
                "log_level": self.observability.log_level,
                "log_file": self.observability.log_file,
                "metrics_enabled": self.observability.metrics_enabled,
                "tracing_enabled": self.observability.tracing_enabled,
                "retention_days": self.observability.retention_days,
                "redact_secrets": self.observability.redact_secrets,
            },
            "model": {
                "providers": self.model.providers,
                "default_model": self.model.default_model,
                "max_tokens": self.model.max_tokens,
                "fallback_providers": self.model.fallback_providers,
            },
            "approvals": {
                "mode": self.approvals.mode,
                "hardline_patterns": self.approvals.hardline_patterns,
                "dangerous_patterns": self.approvals.dangerous_patterns,
            },
            "workspace_root": self.workspace_root,
            "db_path": self.db_path,
            "skills_root": self.skills_root,
            "mcp_servers": self.mcp_servers,
        }
        with open(path, "w") as f:
            yaml.dump(data, f, default_flow_style=False)
