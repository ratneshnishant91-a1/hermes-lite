use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fs::{self, OpenOptions}, io::{BufRead, BufReader, Write}, path::{Path, PathBuf}};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub run_id: String,
    pub at: i64,
    pub kind: String,
    pub message: String,
    pub success: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunSummary {
    pub run_id: String,
    pub started_at: i64,
    pub finished_at: i64,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EventLog {
    root: PathBuf,
}

impl EventLog {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().join("runs");
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn start(&self, input: &str) -> Result<String> {
        let run_id = Uuid::new_v4().to_string();
        self.append(Event { run_id: run_id.clone(), at: Utc::now().timestamp(), kind: "run_started".into(), message: input.into(), success: None })?;
        Ok(run_id)
    }

    pub fn record(&self, run_id: &str, kind: &str, message: &str, success: Option<bool>) -> Result<()> {
        self.append(Event { run_id: run_id.into(), at: Utc::now().timestamp(), kind: kind.into(), message: message.into(), success })
    }

    pub fn finish(&self, run_id: &str, output: &str, success: bool, error: Option<&str>) -> Result<RunSummary> {
        let summary = RunSummary { run_id: run_id.into(), started_at: self.started_at(run_id)?.unwrap_or_else(|| Utc::now().timestamp()), finished_at: Utc::now().timestamp(), success, output: output.into(), error: error.map(str::to_owned) };
        self.record(run_id, "run_finished", &serde_json::to_string(&summary)?, Some(success))?;
        Ok(summary)
    }

    pub fn recent(&self, limit: usize) -> Result<Vec<Event>> {
        let file = OpenOptions::new().read(true).create(true).truncate(false).open(self.path())?;
        let mut items: Vec<Event> = BufReader::new(file).lines().filter_map(|line| line.ok()).filter_map(|line| serde_json::from_str(&line).ok()).collect();
        items.reverse();
        items.truncate(limit);
        Ok(items)
    }

    pub fn path(&self) -> PathBuf { self.root.join("events.jsonl") }

    fn append(&self, event: Event) -> Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(self.path()).context("open event log")?;
        writeln!(file, "{}", serde_json::to_string(&event)?)?;
        file.sync_data()?;
        Ok(())
    }

    fn started_at(&self, run_id: &str) -> Result<Option<i64>> {
        Ok(self.recent(usize::MAX)?.into_iter().find(|e| e.run_id == run_id && e.kind == "run_started").map(|e| e.at))
    }
}
