from __future__ import annotations

import json
import threading
import time
import secrets
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
from typing import Any
import urllib.request
import urllib.error


class APIHandler(BaseHTTPRequestHandler):
    """HTTP request handler for gateway API."""

    agent = None
    config = None
    api_tokens: dict[str, dict] = {}  # token -> {user_id, created_at, expires_at}

    def log_message(self, format, *args):
        """Override to use our logger."""
        if self.agent and self.agent.observability:
            self.agent.observability.logger.info(f"Gateway: {args[0]}")

    def send_json(self, data: dict, status: int = 200) -> None:
        """Send JSON response."""
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

    def _is_user_authorized(self, user_id: str, platform: str = "api") -> bool:
        """Check if user is authorized (5-layer authz)."""
        # Layer 1: Per-platform allow-all
        if platform == "telegram" and self.config.gateway.telegram_allow_all:
            return True
        if platform == "discord" and self.config.gateway.discord_allow_all:
            return True
        if platform == "api" and self.config.gateway.allow_all:
            return True

        # Layer 3: Platform-specific allowlists
        if platform == "telegram" and self.config.gateway.telegram_allowed_users:
            try:
                return int(user_id) in self.config.gateway.telegram_allowed_users
            except ValueError:
                return user_id in [str(u) for u in self.config.gateway.telegram_allowed_users]

        if platform == "discord" and self.config.gateway.discord_allowed_users:
            return user_id in self.config.gateway.discord_allowed_users

        # Layer 4: Global allowlist
        if self.config.gateway.allowed_users:
            return user_id in self.config.gateway.allowed_users

        # Layer 5: Default DENY
        return False

    def _authenticate_request(self) -> str | None:
        """Authenticate API request."""
        # Check API token
        auth_header = self.headers.get("Authorization", "")
        if auth_header.startswith("Bearer "):
            token = auth_header[7:]
            if token in self.api_tokens:
                token_data = self.api_tokens[token]
                if time.time() < token_data["expires_at"]:
                    return token_data["user_id"]
                else:
                    del self.api_tokens[token]

        # Check query param token
        parsed = urlparse(self.path)
        params = parse_qs(parsed.query)
        if "token" in params:
            token = params["token"][0]
            if token in self.api_tokens:
                token_data = self.api_tokens[token]
                if time.time() < token_data["expires_at"]:
                    return token_data["user_id"]

        return None

    def do_GET(self) -> None:
        """Handle GET requests."""
        parsed = urlparse(self.path)

        if parsed.path == "/health":
            self.send_json({"status": "ok", "session": self.agent.session_id})
        elif parsed.path == "/metrics":
            if self.agent.observability:
                self.send_json(self.agent.observability.get_metrics())
            else:
                self.send_json({"status": "ok"})
        else:
            self.send_json({"error": "Not found"}, 404)

    def do_POST(self) -> None:
        """Handle POST requests."""
        parsed = urlparse(self.path)

        if parsed.path == "/chat":
            # Authenticate
            user_id = self._authenticate_request()
            if not user_id:
                self.send_json({"error": "Unauthorized"}, 401)
                return

            # Check authorization
            if not self._is_user_authorized(user_id, platform="api"):
                self.send_json({"error": "User not authorized"}, 403)
                return

            # Read body
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length).decode()

            try:
                data = json.loads(body)
            except json.JSONDecodeError:
                self.send_json({"error": "Invalid JSON"}, 400)
                return

            user_message = data.get("message", "")

            # Run agent
            try:
                response = self.agent.run(user_message)
                self.send_json({
                    "response": response,
                    "session": self.agent.session_id,
                    "user_id": user_id,
                })
            except Exception as e:
                self.send_json({"error": str(e)}, 500)

        elif parsed.path == "/token":
            # Create new API token
            user_id = self._authenticate_request()
            if not user_id:
                self.send_json({"error": "Unauthorized"}, 401)
                return

            data = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))).decode())
            expires_hours = data.get("expires_hours", 24)

            token = secrets.token_urlsafe(32)
            self.api_tokens[token] = {
                "user_id": user_id,
                "created_at": time.time(),
                "expires_at": time.time() + (expires_hours * 3600),
            }

            self.send_json({"token": token, "expires_hours": expires_hours})

        else:
            self.send_json({"error": "Not found"}, 404)


class TelegramBot:
    """Telegram bot integration."""

    def __init__(self, token: str, agent, config):
        self.token = token
        self.agent = agent
        self.config = config
        self.base_url = f"https://api.telegram.org/bot{token}"
        self.running = False
        self.thread: threading.Thread | None = None

    def _api(self, method: str, params: dict | None = None) -> dict:
        """Call Telegram API."""
        url = f"{self.base_url}/{method}"
        data = json.dumps(params or {}).encode() if params else None

        request = urllib.request.Request(
            url,
            data=data,
            headers={"Content-Type": "application/json"},
            method="POST" if data else "GET",
        )

        with urllib.request.urlopen(request, timeout=10) as response:
            return json.loads(response.read())

    def _is_user_authorized(self, user_id: int) -> bool:
        """Check Telegram user authorization."""
        if self.config.gateway.telegram_allow_all:
            return True
        if self.config.gateway.telegram_allowed_users:
            return user_id in self.config.gateway.telegram_allowed_users
        if self.config.gateway.allowed_users:
            try:
                return str(user_id) in self.config.gateway.allowed_users
            except ValueError:
                return str(user_id) in self.config.gateway.allowed_users
        return False

    def _handle_update(self, update: dict) -> None:
        """Handle incoming message."""
        message = update.get("message", {})
        chat_id = message.get("chat", {}).get("id")
        user_id = message.get("from", {}).get("id")
        text = message.get("text", "")

        # Check authorization
        if not self._is_user_authorized(user_id):
            return

        # Run agent
        response = self.agent.run(text)

        # Send response
        self._api("sendMessage", {"chat_id": chat_id, "text": response})

    def _poll(self) -> None:
        """Poll for updates."""
        offset = 0
        while self.running:
            try:
                updates = self._api("getUpdates", {"offset": offset, "timeout": 30})
                for update in updates.get("result", []):
                    offset = update.get("update_id", 0) + 1
                    self._handle_update(update)
            except Exception as e:
                if self.agent.observability:
                    self.agent.observability.logger.error(f"Telegram poll error: {e}")
                time.sleep(5)

    def start(self) -> None:
        """Start bot in background thread."""
        self.running = True
        self.thread = threading.Thread(target=self._poll, daemon=True)
        self.thread.start()

    def stop(self) -> None:
        """Stop bot."""
        self.running = False
        if self.thread:
            self.thread.join(timeout=5)


class Gateway:
    """REST API gateway with Telegram/Discord integrations."""

    def __init__(self, agent, config):
        self.agent = agent
        self.config = config
        self.server: HTTPServer | None = None
        self.telegram: TelegramBot | None = None

    def start(self) -> None:
        """Start gateway server and bots."""
        # Setup handler
        APIHandler.agent = self.agent
        APIHandler.config = self.config

        # Start HTTP server
        self.server = HTTPServer((self.config.gateway.host, self.config.gateway.port), APIHandler)

        server_thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        server_thread.start()

        if self.agent.observability:
            self.agent.observability.logger.info(f"Gateway started on http://{self.config.gateway.host}:{self.config.gateway.port}")

        # Start Telegram bot
        if self.config.gateway.telegram_token:
            self.telegram = TelegramBot(self.config.gateway.telegram_token, self.agent, self.config)
            self.telegram.start()
            if self.agent.observability:
                self.agent.observability.logger.info("Telegram bot started")

    def stop(self) -> None:
        """Stop gateway and bots."""
        if self.server:
            self.server.shutdown()
        if self.telegram:
            self.telegram.stop()
