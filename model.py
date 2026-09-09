from dataclasses import dataclass
from typing import Any
import os
import json
import urllib.request
from router import ModelRouter


@dataclass
class ModelResponse:
    text: str | None = None
    tool_calls: list[dict[str, Any]] | None = None
    provider: str = ""
    model: str = ""


class Model:
    def __init__(self, config=None):
        self.router = ModelRouter()
        self.config = config

    def generate(self, messages, tools):
        """Generate response with multi-provider fallback."""
        try:
            response_data, provider, model = self.router.generate(messages, tools)
            message = response_data["choices"][0]["message"]
            calls = message.get("tool_calls") or []
            return ModelResponse(
                text=message.get("content"),
                tool_calls=calls,
                provider=provider,
                model=model,
            )
        except Exception as e:
            # Fallback to simple OpenAI if router fails
            return self._generate_simple(messages, tools)

    def _generate_simple(self, messages, tools):
        """Simple OpenAI API call (fallback)."""
        api_key = os.getenv("OPENAI_API_KEY") or os.getenv("OPENROUTER_API_KEY")
        if not api_key:
            raise RuntimeError("Set OPENAI_API_KEY or OPENROUTER_API_KEY")

        body = {
            "model": os.getenv("HERMES_MODEL", "gpt-4.1-mini"),
            "messages": messages,
            "tools": tools,
        }

        request = urllib.request.Request(
            "https://api.openai.com/v1/chat/completions",
            data=json.dumps(body).encode(),
            headers={
                "Content-Type": "application/json",
                "Authorization": f"Bearer {api_key}",
            },
            method="POST",
        )

        with urllib.request.urlopen(request) as response:
            data = json.loads(response.read())

        message = data["choices"][0]["message"]
        calls = message.get("tool_calls") or []

        return ModelResponse(
            text=message.get("content"),
            tool_calls=calls,
            provider="openai",
            model=os.getenv("HERMES_MODEL", "gpt-4.1-mini"),
        )

    def get_stats(self) -> dict:
        """Get router statistics."""
        return self.router.get_stats()
