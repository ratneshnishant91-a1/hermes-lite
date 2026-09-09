from __future__ import annotations

import json
import sqlite3
from datetime import datetime, timezone
from pathlib import Path


def _now() -> str:
    return datetime.now(timezone.utc).isoformat()


class Store:
    def __init__(self, path: str = "hermes.db"):
        Path(path).parent.mkdir(parents=True, exist_ok=True)
        self.db = sqlite3.connect(path)
        self.db.row_factory = sqlite3.Row
        self.db.execute("PRAGMA foreign_keys = ON")
        self._init()

    def _init(self) -> None:
        self.db.executescript(
            """
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY,
                session_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT,
                tool_call_id TEXT,
                name TEXT,
                tool_calls TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            );
            CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY,
                content TEXT NOT NULL,
                source TEXT,
                created_at TEXT NOT NULL
            );
            """
        )
        self.db.commit()

    def create_session(self) -> int:
        now = _now()
        cur = self.db.execute(
            "INSERT INTO sessions(created_at, updated_at) VALUES (?, ?)",
            (now, now),
        )
        self.db.commit()
        return int(cur.lastrowid)

    def latest_session(self) -> int | None:
        row = self.db.execute(
            "SELECT id FROM sessions ORDER BY updated_at DESC LIMIT 1"
        ).fetchone()
        return int(row["id"]) if row else None

    def touch_session(self, session_id: int) -> None:
        self.db.execute(
            "UPDATE sessions SET updated_at = ? WHERE id = ?",
            (_now(), session_id),
        )
        self.db.commit()

    def save_message(self, session_id: int, message: dict) -> None:
        self.db.execute(
            """
            INSERT INTO messages(
                session_id, role, content, tool_call_id, name, tool_calls, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            """,
            (
                session_id,
                message["role"],
                message.get("content"),
                message.get("tool_call_id"),
                message.get("name"),
                json.dumps(message["tool_calls"]) if message.get("tool_calls") else None,
                _now(),
            ),
        )
        self.db.commit()

    def load_messages(self, session_id: int) -> list[dict]:
        rows = self.db.execute(
            """
            SELECT role, content, tool_call_id, name, tool_calls
            FROM messages
            WHERE session_id = ?
            ORDER BY id
            """,
            (session_id,),
        ).fetchall()
        messages = []
        for row in rows:
            msg = {"role": row["role"]}
            if row["content"] is not None:
                msg["content"] = row["content"]
            if row["tool_call_id"]:
                msg["tool_call_id"] = row["tool_call_id"]
            if row["name"]:
                msg["name"] = row["name"]
            if row["tool_calls"]:
                msg["tool_calls"] = json.loads(row["tool_calls"])
            messages.append(msg)
        return messages

    def remember(self, content: str, source: str = "agent") -> str:
        self.db.execute(
            "INSERT INTO memories(content, source, created_at) VALUES (?, ?, ?)",
            (content, source, _now()),
        )
        self.db.commit()
        return f"Saved memory: {content}"

    def search_memories(self, query: str, limit: int = 5) -> list[str]:
        rows = self.db.execute(
            """
            SELECT content FROM memories
            WHERE content LIKE ?
            ORDER BY id DESC
            LIMIT ?
            """,
            (f"%{query}%", limit),
        ).fetchall()
        return [row["content"] for row in rows]
