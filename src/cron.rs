//! Persistent cron scheduler with SQLite backend.

use crate::store::Store;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::thread;
use std::time::Duration as StdDuration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub schedule: String, // "every 5m", "every 1h", "daily", "hourly"
    pub tool_name: String,
    pub arguments: Value,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: DateTime<Utc>,
    pub run_count: i64,
}

pub struct CronScheduler {
    store: Store,
    running: bool,
}

impl CronScheduler {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            running: false,
        }
    }

    /// Parse schedule string into Duration
    pub fn parse_schedule(schedule: &str) -> Result<Duration> {
        let s = schedule.trim().to_lowercase();

        if s == "hourly" {
            return Ok(Duration::hours(1));
        }
        if s == "daily" {
            return Ok(Duration::days(1));
        }

        // "every 5m", "every 1h", etc.
        if let Some(rest) = s.strip_prefix("every ") {
            let mut parts = rest.split_whitespace();
            let n: i64 = parts.next().unwrap_or("1").parse()?;
            let unit = parts.next().unwrap_or("m");
            return Ok(match unit.chars().next().unwrap_or('m') {
                'h' => Duration::hours(n),
                'd' => Duration::days(n),
                _ => Duration::minutes(n),
            });
        }

        anyhow::bail!("Unknown schedule format: {} (use 'every 5m', 'hourly', 'daily')", schedule)
    }

    /// Add a new cron job
    pub fn add_job(&self, name: &str, schedule: &str, tool_name: &str, arguments: &Value) -> Result<CronJob> {
        let interval = Self::parse_schedule(schedule)?;
        let next_run = Utc::now() + interval;
        let id = uuid::Uuid::new_v4().to_string();

        let job = CronJob {
            id: id.clone(),
            name: name.to_string(),
            schedule: schedule.to_string(),
            tool_name: tool_name.to_string(),
            arguments: arguments.clone(),
            enabled: true,
            last_run: None,
            next_run,
            run_count: 0,
        };

        // Persist to SQLite
        self.store.create_cron_job(&id, name, schedule, tool_name, &arguments.to_string())?;

        Ok(job)
    }

    /// List all cron jobs
    pub fn list_jobs(&self) -> Result<Vec<CronJob>> {
        self.store.list_cron_jobs()
    }

    /// Remove a cron job
    pub fn remove_job(&self, job_id: &str) -> Result<()> {
        self.store.delete_cron_job(job_id)
    }

    /// Start scheduler in background thread
    pub fn start(&mut self, tool_execute_fn: impl Fn(&str, &Value) -> Result<Value> + Send + 'static) {
        self.running = true;
        let store = self.store.clone();

        thread::spawn(move || {
            while store.is_scheduler_running() {
                let now = Utc::now();

                // Check all jobs
                if let Ok(jobs) = store.list_cron_jobs() {
                    for mut job in jobs {
                        if !job.enabled || job.next_run > now {
                            continue;
                        }

                        // Execute job
                        match tool_execute_fn(&job.tool_name, &job.arguments) {
                            Ok(_) => {
                                let interval = CronScheduler::parse_schedule(&job.schedule).unwrap_or(Duration::minutes(5));
                                job.last_run = Some(now);
                                job.next_run = now + interval;
                                job.run_count += 1;
                                let _ = store.update_cron_job(&job);
                            }
                            Err(e) => {
                                eprintln!("Cron job {} failed: {}", job.name, e);
                                // Retry in 5 minutes
                                job.next_run = now + Duration::minutes(5);
                                let _ = store.update_cron_job(&job);
                            }
                        }
                    }
                }

                thread::sleep(StdDuration::from_secs(30));
            }
        });
    }

    /// Stop scheduler
    pub fn stop(&mut self) {
        self.running = false;
    }
}
