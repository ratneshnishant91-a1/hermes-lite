//! Background tool execution and a tiny cron loop.
//!
//! Uses OS threads (not async). Edition 2024 does not require Tokio for this.

use anyhow::Result;
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub status: String,
    pub detail: String,
}

#[derive(Clone, Default)]
pub struct JobHub {
    inner: Arc<Mutex<Vec<Job>>>,
}

impl JobHub {
    pub fn spawn(&self, name: impl Into<String>, work: impl FnOnce() -> String + Send + 'static) -> String {
        let id = Uuid::new_v4().to_string();
        let job = Job {
            id: id.clone(),
            name: name.into(),
            status: "running".into(),
            detail: String::new(),
        };
        self.inner.lock().expect("job lock").push(job);
        let hub = self.clone();
        let id2 = id.clone();
        thread::spawn(move || {
            let detail = work();
            if let Ok(mut jobs) = hub.inner.lock() {
                if let Some(j) = jobs.iter_mut().find(|j| j.id == id2) {
                    j.status = "completed".into();
                    j.detail = detail;
                }
            }
        });
        id
    }

    pub fn list(&self) -> Vec<Job> {
        self.inner.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CronSpec {
    pub id: String,
    pub name: String,
    pub every_secs: i64,
    pub next_run: String,
}

pub fn parse_every(schedule: &str) -> Result<i64> {
    let s = schedule.trim().to_ascii_lowercase();
    if s == "daily" {
        return Ok(86_400);
    }
    if let Some(rest) = s.strip_prefix("every ") {
        let mut parts = rest.split_whitespace();
        let n: i64 = parts.next().unwrap_or("0").parse()?;
        let unit = parts.next().unwrap_or("s");
        return Ok(match unit.chars().next().unwrap_or('s') {
            'h' => n * 3600,
            'm' => n * 60,
            _ => n,
        });
    }
    anyhow::bail!("unsupported schedule: {schedule} (use 'every 5m' or 'daily')")
}

impl JobHub {
    pub fn schedule(&self, name: &str, schedule: &str) -> Result<CronSpec> {
        let every = parse_every(schedule)?;
        let id = Uuid::new_v4().to_string();
        let next = (Utc::now() + ChronoDuration::seconds(every)).to_rfc3339();
        Ok(CronSpec {
            id,
            name: name.into(),
            every_secs: every,
            next_run: next,
        })
    }
}
