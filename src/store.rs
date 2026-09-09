use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: Option<String>,
    pub tool_call_id: Option<String>,
    pub name: Option<String>,
    pub tool_calls: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct GoalRecord {
    pub id: String,
    pub description: String,
    pub status: String,
    pub plan: String,
    pub current_step: i64,
}

#[derive(Debug, Clone)]
pub struct CronJobRecord {
    pub id: String,
    pub name: String,
    pub schedule: String,
    pub tool_name: String,
    pub arguments: String,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: String,
    pub run_count: i64,
}

pub struct Store {
    db: Connection,
    scheduler_running: Arc<AtomicBool>,
}

impl Clone for Store {
    fn clone(&self) -> Self {
        Self {
            db: Connection::open(self.db.path()).unwrap(),
            scheduler_running: self.scheduler_running.clone(),
        }
    }
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db = Connection::open(path)?;
        db.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA temp_store=MEMORY;
             PRAGMA cache_size=-16384;
             PRAGMA mmap_size=268435456;
             PRAGMA foreign_keys = ON;
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
             CREATE TABLE IF NOT EXISTS goals (
                id TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                plan TEXT NOT NULL,
                current_step INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS constraints (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS cron_jobs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                schedule TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                arguments TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_run TEXT,
                next_run TEXT NOT NULL,
                run_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, id);
             CREATE INDEX IF NOT EXISTS idx_memories_content ON memories(content);
             CREATE INDEX IF NOT EXISTS idx_goals_status ON goals(status);
             CREATE INDEX IF NOT EXISTS idx_cron_jobs_next_run ON cron_jobs(next_run);",
        )?;
        Ok(Self {
            db,
            scheduler_running: Arc::new(AtomicBool::new(true)),
        })
    }

    pub fn is_scheduler_running(&self) -> bool {
        self.scheduler_running.load(Ordering::SeqCst)
    }

    pub fn create_session(&self) -> Result<i64> {
        let now = Utc::now().to_rfc3339();
        self.db.execute("INSERT INTO sessions(created_at, updated_at) VALUES (?1, ?2)", params![now, now])?;
        Ok(self.db.last_insert_rowid())
    }

    pub fn latest_session(&self) -> Result<Option<i64>> {
        Ok(self.db.query_row("SELECT id FROM sessions ORDER BY updated_at DESC LIMIT 1", [], |r| r.get(0)).optional()?)
    }

    pub fn save_message(&self, session_id: i64, msg: &Message) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let tool_calls = msg.tool_calls.as_ref().map(|v| v.to_string());
        self.db.execute("INSERT INTO messages(session_id, role, content, tool_call_id, name, tool_calls, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![session_id, msg.role, msg.content, msg.tool_call_id, msg.name, tool_calls, now])?;
        self.db.execute("UPDATE sessions SET updated_at = ?1 WHERE id = ?2", params![now, session_id])?;
        Ok(())
    }

    pub fn load_messages(&self, session_id: i64) -> Result<Vec<Message>> {
        let mut stmt = self.db.prepare("SELECT role, content, tool_call_id, name, tool_calls FROM messages WHERE session_id = ?1 ORDER BY id")?;
        let rows = stmt.query_map(params![session_id], |r| {
            let tool_calls: Option<String> = r.get(4)?;
            Ok(Message { role: r.get(0)?, content: r.get(1)?, tool_call_id: r.get(2)?, name: r.get(3)?, tool_calls: tool_calls.and_then(|s| serde_json::from_str(&s).ok()) })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn remember(&self, content: &str, source: &str) -> Result<String> {
        let now = Utc::now().to_rfc3339();
        self.db.execute("INSERT INTO memories(content, source, created_at) VALUES (?1, ?2, ?3)", params![content, source, now])?;
        Ok(format!("Saved memory: {content}"))
    }

    pub fn search_memories(&self, query: &str, limit: i64) -> Result<Vec<String>> {
        let mut stmt = self.db.prepare("SELECT content FROM memories WHERE content LIKE ?1 ORDER BY id DESC LIMIT ?2")?;
        let rows = stmt.query_map(params![format!("%{query}%"), limit], |r| r.get(0))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn create_goal(&self, id: &str, description: &str, plan: &[String]) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.db.execute("INSERT OR REPLACE INTO goals(id, description, status, plan, current_step, created_at, updated_at) VALUES (?1, ?2, 'running', ?3, 0, ?4, ?4)", params![id, description, serde_json::to_string(plan)?, now])?;
        Ok(())
    }

    pub fn update_goal(&self, id: &str, status: &str, step: usize) -> Result<()> {
        self.db.execute("UPDATE goals SET status=?1, current_step=?2, updated_at=?3 WHERE id=?4", params![status, step as i64, Utc::now().to_rfc3339(), id])?;
        Ok(())
    }

    pub fn list_goals(&self) -> Result<Vec<GoalRecord>> {
        let mut stmt = self.db.prepare("SELECT id, description, status, plan, current_step FROM goals ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |r| Ok(GoalRecord { id: r.get(0)?, description: r.get(1)?, status: r.get(2)?, plan: r.get(3)?, current_step: r.get(4)? }))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn set_constraint(&self, key: &str, value: &str) -> Result<()> {
        self.db.execute("INSERT OR REPLACE INTO constraints(key,value,updated_at) VALUES (?1,?2,?3)", params![key, value, Utc::now().to_rfc3339()])?;
        Ok(())
    }

    pub fn get_constraint(&self, key: &str) -> Result<Option<String>> {
        Ok(self.db.query_row("SELECT value FROM constraints WHERE key=?1", params![key], |r| r.get(0)).optional()?)
    }

    // Cron job methods
    pub fn create_cron_job(&self, id: &str, name: &str, schedule: &str, tool_name: &str, arguments: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let next_run = (Utc::now() + chrono::Duration::minutes(5)).to_rfc3339();
        self.db.execute(
            "INSERT INTO cron_jobs(id, name, schedule, tool_name, arguments, enabled, next_run, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?7)",
            params![id, name, schedule, tool_name, arguments, next_run, now],
        )?;
        Ok(())
    }

    pub fn update_cron_job(&self, job: &crate::cron::CronJob) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let last_run = job.last_run.map(|dt| dt.to_rfc3339());
        self.db.execute(
            "UPDATE cron_jobs SET last_run=?1, next_run=?2, run_count=?3, updated_at=?4 WHERE id=?5",
            params![last_run, job.next_run.to_rfc3339(), job.run_count, now, job.id],
        )?;
        Ok(())
    }

    pub fn list_cron_jobs(&self) -> Result<Vec<crate::cron::CronJob>> {
        let mut stmt = self.db.prepare("SELECT id, name, schedule, tool_name, arguments, enabled, last_run, next_run, run_count FROM cron_jobs ORDER BY next_run")?;
        let rows = stmt.query_map([], |r| {
            let last_run: Option<String> = r.get(6)?;
            Ok(crate::cron::CronJob {
                id: r.get(0)?,
                name: r.get(1)?,
                schedule: r.get(2)?,
                tool_name: r.get(3)?,
                arguments: serde_json::from_str(&r.get::<_, String>(4)?)?,
                enabled: r.get::<_, i64>(5)? == 1,
                last_run: last_run.and_then(|s| DateTime::parse_from_rfc3339(&s).ok()).map(|dt| dt.with_timezone(&Utc)),
                next_run: DateTime::parse_from_rfc3339(&r.get::<_, String>(7)?)?.with_timezone(&Utc),
                run_count: r.get(8)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete_cron_job(&self, job_id: &str) -> Result<()> {
        self.db.execute("DELETE FROM cron_jobs WHERE id=?1", params![job_id])?;
        Ok(())
    }

    pub fn backup(&self, out: &mut dyn Write) -> Result<()> {
        for table in ["sessions", "messages", "memories", "goals", "constraints", "cron_jobs"] {
            let mut stmt = self.db.prepare(&format!("SELECT * FROM {table}"))?;
            let rows = stmt.query_map([], |r| {
                let mut values = Vec::new();
                for i in 0..r.as_ref().column_count() {
                    let v: String = r.get(i)?;
                    values.push(format!("'{}'", v.replace("'", "''")));
                }
                Ok(values)
            })?;
            for row in rows.filter_map(|r| r.ok()) {
                writeln!(out, "INSERT INTO {} VALUES ({});", table, row.join(","))?;
            }
        }
        Ok(())
    }
}
