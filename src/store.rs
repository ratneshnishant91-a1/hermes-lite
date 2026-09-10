use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob { pub id: String, pub name: String, pub every_secs: i64, pub command: String, pub next_run: i64, pub last_run: Option<i64>, pub enabled: bool }

pub struct Store { db: Connection }
impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db = Connection::open(path)?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; CREATE TABLE IF NOT EXISTS memory (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS cron_jobs (id TEXT PRIMARY KEY, name TEXT NOT NULL, every_secs INTEGER NOT NULL, command TEXT NOT NULL, next_run INTEGER NOT NULL, last_run INTEGER, enabled INTEGER NOT NULL);")?;
        Ok(Self { db })
    }
    pub fn remember(&self, key: &str, value: &str) -> Result<()> { self.db.execute("INSERT INTO memory(key,value,updated_at) VALUES(?1,?2,?3) ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at", params![key,value,Utc::now().timestamp()])?; Ok(()) }
    pub fn recall(&self, key: &str) -> Result<Option<String>> { Ok(self.db.query_row("SELECT value FROM memory WHERE key=?1", [key], |r| r.get(0)).ok()) }
    pub fn save_cron(&self, job: &CronJob) -> Result<()> { self.db.execute("INSERT INTO cron_jobs(id,name,every_secs,command,next_run,last_run,enabled) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET next_run=excluded.next_run,last_run=excluded.last_run,enabled=excluded.enabled", params![job.id,job.name,job.every_secs,job.command,job.next_run,job.last_run,job.enabled as i32])?; Ok(()) }
    pub fn due_cron(&self, now: i64) -> Result<Vec<CronJob>> { let mut s=self.db.prepare("SELECT id,name,every_secs,command,next_run,last_run,enabled FROM cron_jobs WHERE enabled=1 AND next_run<=?1")?; let rows=s.query_map([now], |r| Ok(CronJob { id:r.get(0)?,name:r.get(1)?,every_secs:r.get(2)?,command:r.get(3)?,next_run:r.get(4)?,last_run:r.get(5)?,enabled:r.get::<_,i32>(6)? != 0 }))?; Ok(rows.filter_map(Result::ok).collect()) }
}
