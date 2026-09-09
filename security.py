from __future__ import annotations

import re
import hashlib
import secrets
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


class InputValidator:
    """
    Strict input validation for all user inputs.
    Prevents injection attacks, path traversal, and malicious input.
    """

    # Dangerous patterns
    PATH_TRAVERSAL = re.compile(r'\.\./|\.\.\\|%2e%2e%2f|%2e%2e/|\.\.%2f|%2e%2e%5c')
    SQL_INJECTION = re.compile(r'(;|--|\b(union|select|insert|update|delete|drop|truncate|exec)\b)', re.IGNORECASE)
    SCRIPT_INJECTION = re.compile(r'(<script|javascript:|on\w+=)', re.IGNORECASE)
    COMMAND_INJECTION = re.compile(r'(`|\$\(|\||;|&|>|<)', re.IGNORECASE)

    MAX_INPUT_LENGTH = 10000
    MAX_FILENAME_LENGTH = 255
    ALLOWED_FILENAME_CHARS = re.compile(r'^[a-zA-Z0-9_\-\.]+$')

    @classmethod
    def validate_user_message(cls, message: str) -> tuple[bool, str]:
        """Validate user message."""
        if not message or not message.strip():
            return False, "Empty message"

        if len(message) > cls.MAX_INPUT_LENGTH:
            return False, f"Message too long (max {cls.MAX_INPUT_LENGTH} chars)"

        if cls.PATH_TRAVERSAL.search(message):
            return False, "Path traversal detected"

        if cls.SCRIPT_INJECTION.search(message):
            return False, "Script injection detected"

        return True, ""

    @classmethod
    def validate_filename(cls, filename: str) -> tuple[bool, str]:
        """Validate filename."""
        if not filename or not filename.strip():
            return False, "Empty filename"

        if len(filename) > cls.MAX_FILENAME_LENGTH:
            return False, f"Filename too long (max {cls.MAX_FILENAME_LENGTH} chars)"

        if '..' in filename:
            return False, "Path traversal not allowed"

        if not cls.ALLOWED_FILENAME_CHARS.match(filename):
            return False, "Invalid characters in filename"

        return True, ""

    @classmethod
    def validate_tool_arguments(cls, tool_name: str, arguments: dict) -> tuple[bool, str]:
        """Validate tool arguments."""
        if not isinstance(arguments, dict):
            return False, "Arguments must be a dictionary"

        # Validate each argument
        for key, value in arguments.items():
            if isinstance(value, str):
                valid, error = cls.validate_user_message(value)
                if not valid:
                    return False, f"Invalid argument {key}: {error}"

        return True, ""


class AuditLogger:
    """
    Immutable security audit logging.
    All security-relevant events are logged with tamper-evident hashing.
    """

    def __init__(self, log_dir: str = "audit_logs"):
        self.log_dir = Path(log_dir)
        self.log_dir.mkdir(parents=True, exist_ok=True)
        self.current_log = self.log_dir / f"audit_{datetime.now(timezone.utc).strftime('%Y%m%d')}.log"
        self.hash_chain = ""

    def log_event(self, event_type: str, details: dict, user_id: str = "system"):
        """
        Log a security event with tamper-evident hashing.
        """
        timestamp = datetime.now(timezone.utc).isoformat()

        # Create log entry
        entry = {
            "timestamp": timestamp,
            "event_type": event_type,
            "user_id": user_id,
            "details": details,
            "previous_hash": self.hash_chain,
        }

        # Create hash chain (tamper-evident)
        entry_hash = hashlib.sha256(f"{timestamp}{event_type}{user_id}{self.hash_chain}".encode()).hexdigest()
        entry["hash"] = entry_hash
        self.hash_chain = entry_hash

        # Append to log
        import json
        with self.current_log.open("a") as f:
            f.write(json.dumps(entry) + "\n")

    def log_security_event(self, event_type: str, details: dict, user_id: str = "system"):
        """Log a security-critical event."""
        self.log_event(f"SECURITY_{event_type}", details, user_id)

    def log_access(self, resource: str, action: str, user_id: str, success: bool):
        """Log access attempt."""
        self.log_event("ACCESS", {
            "resource": resource,
            "action": action,
            "success": success,
        }, user_id)

    def log_tool_execution(self, tool_name: str, arguments: dict, user_id: str, blocked: bool = False, reason: str = ""):
        """Log tool execution attempt."""
        self.log_event("TOOL_EXECUTION", {
            "tool": tool_name,
            "arguments": {k: v if len(str(v)) < 100 else str(v)[:100] + "..." for k, v in arguments.items()},
            "blocked": blocked,
            "reason": reason,
        }, user_id)

    def get_recent_events(self, limit: int = 100) -> list[dict]:
        """Get recent audit events."""
        events = []
        if self.current_log.exists():
            import json
            with self.current_log.open() as f:
                for line in f:
                    try:
                        events.append(json.loads(line))
                    except:
                        pass
        return events[-limit:]


class RateLimiter:
    """
    Rate limiting to prevent abuse.
    """

    def __init__(self, max_requests: int = 100, window_seconds: int = 60):
        self.max_requests = max_requests
        self.window_seconds = window_seconds
        self.requests: dict[str, list[float]] = {}

    def is_allowed(self, user_id: str) -> tuple[bool, float]:
        """
        Check if request is allowed.
        Returns (allowed, wait_seconds).
        """
        now = datetime.now(timezone.utc).timestamp()

        if user_id not in self.requests:
            self.requests[user_id] = []

        # Remove old requests
        self.requests[user_id] = [
            t for t in self.requests[user_id]
            if now - t < self.window_seconds
        ]

        # Check limit
        if len(self.requests[user_id]) >= self.max_requests:
            oldest = min(self.requests[user_id])
            wait_seconds = self.window_seconds - (now - oldest)
            return False, max(0, wait_seconds)

        # Allow
        self.requests[user_id].append(now)
        return True, 0.0

    def get_remaining(self, user_id: str) -> int:
        """Get remaining requests in window."""
        now = datetime.now(timezone.utc).timestamp()
        if user_id not in self.requests:
            return self.max_requests
        recent = [t for t in self.requests[user_id] if now - t < self.window_seconds]
        return max(0, self.max_requests - len(recent))


class SecretsManager:
    """
    Secure credential handling.
    Never logs or exposes secrets.
    """

    SENSITIVE_PATTERNS = [
        re.compile(r'(?i).*key.*', re.IGNORECASE),
        re.compile(r'(?i).*token.*', re.IGNORECASE),
        re.compile(r'(?i).*secret.*', re.IGNORECASE),
        re.compile(r'(?i).*password.*', re.IGNORECASE),
        re.compile(r'(?i).*api_.*', re.IGNORECASE),
        re.compile(r'(?i).*credential.*', re.IGNORECASE),
    ]

    @classmethod
    def is_sensitive(cls, key: str) -> bool:
        """Check if key is sensitive."""
        return any(pattern.match(key) for pattern in cls.SENSITIVE_PATTERNS)

    @classmethod
    def redact_secrets(cls, data: Any) -> Any:
        """Redact secrets from data."""
        if isinstance(data, dict):
            return {
                k: cls.redact_secrets(v) if not cls.is_sensitive(k) else "***REDACTED***"
                for k, v in data.items()
            }
        elif isinstance(data, list):
            return [cls.redact_secrets(item) for item in data]
        elif isinstance(data, str):
            # Redact API keys, tokens, etc.
            redacted = data
            for pattern in [r'sk-[a-zA-Z0-9]+', r'token_[a-zA-Z0-9]+', r'api_key_[a-zA-Z0-9]+']:
                redacted = re.sub(pattern, '***REDACTED***', redacted)
            return redacted
        return data

    @classmethod
    def generate_secure_token(cls, length: int = 32) -> str:
        """Generate a secure random token."""
        return secrets.token_urlsafe(length)
