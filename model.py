from dataclasses import dataclass
from typing import Any
import os
import json
import urllib.request


@dataclass
class ModelResponse:
    text: str | None = None
    tool_calls: list[dict[str, Any]] | None = None


class Model:
    def __init__(self, model: str | None = None):
        self.model = model or os.getenv("HERMES_MODEL", "gpt-4.1-mini")
        self.api_key = os.getenv("OPENAI_API_KEY")
        if not self.api_key:
            raise RuntimeError("Set OPENAI_API_KEY before starting Hermes-Lite.")

    def generate(self, messages, tools):
        body = {"model": self.model, "messages": messages, "tools": tools}
        request = urllib.request.Request(
            "https://api.openai.com/v1/chat/completions",
            data=json.dumps(body).encode(),
            headers={"Content-Type": "application/json", "Authorization": f"Bearer {self.api_key}"},
            method="POST",
        )
        with urllib.request.urlopen(request) as response:
            data = json.loads(response.read())
        message = data["choices"][0]["message"]
        calls = message.get("tool_calls") or []
        return ModelResponse(text=message.get("content"), tool_calls=calls)

    def get_stats(self) -> dict:
        return {"total_requests": 0, "providers": [{"name": "openai", "enabled": True, "models": [self.model]}]}
