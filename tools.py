from dataclasses import dataclass
from typing import Callable
import subprocess
import json
import os


@dataclass
class Tool:
    name: str
    description: str
    parameters: dict
    handler: Callable


class ToolRegistry:
    def __init__(self):
        self.tools: dict[str, Tool] = {}

    def register(self, tool: Tool):
        self.tools[tool.name] = tool

    def schemas(self):
        return [
            {
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters,
                },
            }
            for tool in self.tools.values()
        ]

    def execute(self, name, arguments):
        if name not in self.tools:
            raise ValueError(f"Unknown tool: {name}")
        tool = self.tools[name]
        return tool.handler(**arguments)


def create_default_registry(workspace, memory, skills):
    registry = ToolRegistry()

    def shell_command(command: str):
        result = subprocess.run(
            command,
            shell=True,
            capture_output=True,
            text=True,
            timeout=30,
            cwd=str(workspace.files),
        )
        return {
            "exit_code": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr,
        }

    def read_file(path: str):
        target = workspace.resolve(path)
        return target.read_text(encoding="utf-8")

    def write_file(path: str, content: str):
        target = workspace.resolve(path)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content, encoding="utf-8")
        return f"Wrote {target.relative_to(workspace.files)}"

    def fetch_web(url: str):
        from browser import fetch_url
        return fetch_url(url)

    def search_web(query: str, num_results: int = 5):
        from search import web_search
        return web_search(query, num_results)

    registry.register(
        Tool(
            name="shell",
            description="Execute a shell command in the workspace.",
            parameters={
                "type": "object",
                "properties": {"command": {"type": "string"}},
                "required": ["command"],
            },
            handler=shell_command,
        )
    )
    registry.register(
        Tool(
            name="read_file",
            description="Read a text file from the workspace.",
            parameters={
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"],
            },
            handler=read_file,
        )
    )
    registry.register(
        Tool(
            name="write_file",
            description="Write a text file to the workspace.",
            parameters={
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"},
                },
                "required": ["path", "content"],
            },
            handler=write_file,
        )
    )
    registry.register(
        Tool(
            name="fetch_url",
            description="Fetch a web page and extract its text content.",
            parameters={
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "Full URL starting with http:// or https://"}
                },
                "required": ["url"],
            },
            handler=fetch_web,
        )
    )
    registry.register(
        Tool(
            name="web_search",
            description="Search the web for information.",
            parameters={
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"},
                    "num_results": {"type": "integer", "description": "Number of results (default 5)"},
                },
                "required": ["query"],
            },
            handler=search_web,
        )
    )
    registry.register(
        Tool(
            name="memory_save",
            description="Save an important long-term fact.",
            parameters={
                "type": "object",
                "properties": {"content": {"type": "string"}},
                "required": ["content"],
            },
            handler=lambda content: memory.remember(content),
        )
    )
    registry.register(
        Tool(
            name="memory_search",
            description="Search long-term memory.",
            parameters={
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer"},
                },
                "required": ["query"],
            },
            handler=lambda query, limit=5: memory.search(query, limit),
        )
    )
    registry.register(
        Tool(
            name="skill_load",
            description="Load the full instructions for a named skill.",
            parameters={
                "type": "object",
                "properties": {"name": {"type": "string"}},
                "required": ["name"],
            },
            handler=lambda name: skills.load(name),
        )
    )
    return registry
