from __future__ import annotations

import json
import urllib.request
import urllib.error
import os
from dataclasses import dataclass, field
from typing import Any


@dataclass
class ProviderConfig:
    name: str
    base_url: str
    api_key_env: str
    models: list[str]
    priority: int = 0
    enabled: bool = True


@dataclass
class RouterStats:
    total_requests: int = 0
    successful_requests: int = 0
    fallback_count: int = 0
    last_provider: str = ""
    last_model: str = ""


class ModelRouter:
    """
    Multi-provider model router with automatic fallback.
    """

    DEFAULT_PROVIDERS = [
        ProviderConfig(
            name="openrouter",
            base_url="https://openrouter.ai/api/v1",
            api_key_env="OPENROUTER_API_KEY",
            models=["anthropic/claude-sonnet-4-5-20260514", "openai/gpt-4.1-mini"],
            priority=1,
        ),
        ProviderConfig(
            name="openai",
            base_url="https://api.openai.com/v1",
            api_key_env="OPENAI_API_KEY",
            models=["gpt-4.1-mini", "gpt-4o-mini"],
            priority=2,
        ),
        ProviderConfig(
            name="anthropic",
            base_url="https://api.anthropic.com",
            api_key_env="ANTHROPIC_API_KEY",
            models=["claude-sonnet-4-20260514", "claude-3-5-sonnet-20241022"],
            priority=3,
        ),
        ProviderConfig(
            name="gemini",
            base_url="https://generativelanguage.googleapis.com/v1beta",
            api_key_env="GEMINI_API_KEY",
            models=["gemini-2.5-flash", "gemini-2.5-pro"],
            priority=4,
        ),
        ProviderConfig(
            name="local",
            base_url="http://localhost:11434/v1",
            api_key_env="",
            models=["qwen2.5:7b", "llama3.2:3b"],
            priority=5,
        ),
    ]

    def __init__(self, providers: list[ProviderConfig] | None = None):
        self.providers = providers or self.DEFAULT_PROVIDERS
        self.stats = RouterStats()
        self._sort_by_priority()

    def _sort_by_priority(self) -> None:
        """Sort providers by priority."""
        self.providers.sort(key=lambda p: p.priority)

    def _get_api_key(self, provider: ProviderConfig) -> str | None:
        """Get API key from environment."""
        if not provider.api_key_env:
            return ""
        return os.getenv(provider.api_key_env)

    def _make_request(
        self,
        provider: ProviderConfig,
        model: str,
        messages: list[dict],
        tools: list[dict] | None = None,
    ) -> tuple[dict, bool]:
        """Make API request to provider."""
        api_key = self._get_api_key(provider)
        if not api_key and provider.api_key_env:
            return {}, False

        # Build request
        body = {"model": model, "messages": messages}
        if tools:
            body["tools"] = tools

        # Build headers
        headers = {"Content-Type": "application/json"}
        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"

        # Build URL
        if provider.name == "anthropic":
            url = f"{provider.base_url}/v1/messages"
            body["max_tokens"] = 4096
            headers["x-api-key"] = api_key
            headers["anthropic-version"] = "2023-06-01"
        elif provider.name == "gemini":
            url = f"{provider.base_url}/models/{model}:generateContent?key={api_key}"
            body = {"contents": [{"parts": [{"text": messages[-1]["content"]}]}]}
        else:
            url = f"{provider.base_url}/chat/completions"

        try:
            request = urllib.request.Request(
                url,
                data=json.dumps(body).encode(),
                headers=headers,
                method="POST",
            )

            with urllib.request.urlopen(request, timeout=60) as response:
                data = json.loads(response.read())

            # Normalize response
            if provider.name == "anthropic":
                content = data.get("content", [{}])[0].get("text", "")
                normalized = {"choices": [{"message": {"content": content, "tool_calls": []}}]}
            elif provider.name == "gemini":
                content = data.get("candidates", [{}])[0].get("content", {}).get("parts", [{}])[0].get("text", "")
                normalized = {"choices": [{"message": {"content": content, "tool_calls": []}}]}
            else:
                normalized = data

            return normalized, True

        except Exception:
            return {}, False

    def generate(
        self,
        messages: list[dict],
        tools: list[dict] | None = None,
    ) -> tuple[dict, str, str]:
        """
        Generate response with automatic fallback.
        Returns (response_data, provider_name, model_name).
        """
        self.stats.total_requests += 1

        # Try each provider in priority order
        for provider in self.providers:
            if not provider.enabled:
                continue

            for model in provider.models:
                response, success = self._make_request(provider, model, messages, tools)

                if success and "error" not in response:
                    self.stats.successful_requests += 1
                    self.stats.last_provider = provider.name
                    self.stats.last_model = model
                    return response, provider.name, model

                # Log fallback
                self.stats.fallback_count += 1

        # All providers failed
        raise RuntimeError(f"All providers failed. Stats: {self.stats}")

    def get_stats(self) -> dict:
        """Get router statistics."""
        return {
            "total_requests": self.stats.total_requests,
            "successful_requests": self.stats.successful_requests,
            "fallback_count": self.stats.fallback_count,
            "last_provider": self.stats.last_provider,
            "last_model": self.stats.last_model,
            "providers": [
                {"name": p.name, "enabled": p.enabled, "models": p.models}
                for p in self.providers
            ],
        }
