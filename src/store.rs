use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: Option<String>,
    pub tool_call_id: Option<String>,
    pub name: Option<String>,
    pub tool_calls: Option<Value>,
}

pub struct Store {
    db: Connection,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db = Connection::open(path)?;
        db.execute_batch(
            "PRAGMA foreign_keys = ON;
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
             );",
        )?;
        Ok(Self { db })
    }

    pub fn create_session(&self) -> Result<i64> {
        let now = Utc::now().to_rfc3339();
        self.db
            .execute(
                "INSERT INTO sessions(created_at, updated_at) VALUES (?1, ?2)",
                params![now, now],
            )?;
        Ok(self.db.last_insert_rowid())
    }

    pub fn latest_session(&self) -> Result<Option<i64>> {
        let id = self
            .db
            .query_row(
                "SELECT id FROM sessions ORDER BY updated_at DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Ok(id)
    }

    pub fn save_message(&self, session_id: i64, msg: &Message) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let tool_calls = msg.tool_calls.as_ref().map(|v| v.to_string());
        self.db.execute(
            "INSERT INTO messages(session_id, role, content, tool_call_id, name, tool_calls, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                session_id,
                msg.role,
                msg.content,
                msg.tool_call_id,
                msg.name,
                tool_calls,
                now
            ],
        )?;
        self.db.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![now, session_id],
        )?;
        Ok(())
    }

    pub fn load_messages(&self, session_id: i64) -> Result<Vec<Message>> {
        let mut stmt = self.db.prepare(
            "SELECT role, content, tool_call_id, name, tool_calls FROM messages WHERE session_id = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![session_id], |r| {
            let tool_calls: Option<String> = r.get(4)?;
            Ok(Message {
                role: r.get(0)?,
                content: r.get(1)?,
                tool_call_id: r.get(2)?,
                name: r.get(3)?,
                tool_calls: tool_calls.and_then(|s| serde_json::from_str(&s).ok()),
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn remember(&self, content: &str, source: &str) -> Result<String> {
        let now = Utc::now().to_rfc3339();
        self.db.execute(
            "INSERT INTO memories(content, source, created_at) VALUES (?1, ?2, ?3)",
            params![content, source, now],
        )?;
        Ok(format!("Saved memory: {content}"))
    }

    pub fn search_memories(&self, query: &str, limit: i64) -> Result<Vec<String>> {
        let mut stmt = self.db.prepare(
            "SELECT content FROM memories WHERE content LIKE ?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![format!("%{query}%"), limit], |r| r.get(0))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}
