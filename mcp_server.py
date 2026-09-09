from __future__ import annotations

import json
import sys
from typing import Any
from dataclasses import dataclass


@dataclass
class MCPRequest:
    jsonrpc: str
    id: int | str
    method: str
    params: dict | None = None


@dataclass
class MCPResponse:
    jsonrpc: str
    id: int | str
    result: Any | None = None
    error: dict | None = None


class MCPServer:
    """Model Context Protocol server for Hermes-Lite tools."""

    def __init__(self, tool_registry):
        self.tools = tool_registry

    def _send_response(self, response: MCPResponse) -> None:
        """Send JSON-RPC response via stdout."""
        output = {"jsonrpc": response.jsonrpc, "id": response.id}
        if response.result is not None:
            output["result"] = response.result
        if response.error is not None:
            output["error"] = response.error
        sys.stdout.write(json.dumps(output) + "\n")
        sys.stdout.flush()

    def _handle_initialize(self, req: MCPRequest) -> dict:
        """Handle initialize request."""
        return {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "hermes-lite", "version": "2.0.0"},
        }

    def _handle_tools_list(self, req: MCPRequest) -> dict:
        """Handle tools/list request."""
        return {"tools": self.tools.schemas()}

    def _handle_tools_call(self, req: MCPRequest) -> dict:
        """Handle tools/call request."""
        if not req.params:
            return {"error": {"code": -32602, "message": "Missing params"}}

        name = req.params.get("name")
        arguments = req.params.get("arguments", {})

        if not name:
            return {"error": {"code": -32602, "message": "Missing tool name"}}

        try:
            result = self.tools.execute(name, arguments)
            return {
                "content": [{"type": "text", "text": json.dumps(result, indent=2)}],
            }
        except Exception as e:
            return {"error": {"code": -32000, "message": f"Tool execution failed: {e}"}}

    def handle_request(self, raw: str) -> None:
        """Process a single JSON-RPC request."""
        try:
            data = json.loads(raw)
            req = MCPRequest(
                jsonrpc=data.get("jsonrpc", "2.0"),
                id=data.get("id", 0),
                method=data.get("method", ""),
                params=data.get("params"),
            )

            # Route methods
            if req.method == "initialize":
                result = self._handle_initialize(req)
            elif req.method == "tools/list":
                result = self._handle_tools_list(req)
            elif req.method == "tools/call":
                result = self._handle_tools_call(req)
            else:
                result = {"error": {"code": -32601, "message": f"Method not found: {req.method}"}}

            response = MCPResponse(jsonrpc="2.0", id=req.id, result=result if "error" not in result else None, error=result.get("error"))
            self._send_response(response)

        except json.JSONDecodeError as e:
            response = MCPResponse(jsonrpc="2.0", id=0, error={"code": -32700, "message": f"Parse error: {e}"})
            self._send_response(response)

    def run_stdio(self) -> None:
        """Run MCP server over stdio."""
        for line in sys.stdin:
            line = line.strip()
            if line:
                self.handle_request(line)
