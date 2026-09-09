//! Sub-agent delegation system.

use crate::config::Config;
use crate::store::Store;
use crate::tools::ToolRegistry;
use crate::workspace::Workspace;
use crate::skills::Skills;
use crate::model::Model;
use crate::context::Context;
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SubAgentTask {
    pub id: String,
    pub description: String,
    pub status: String,
    pub result: Option<String>,
}

pub struct SubAgentPool {
    tasks: Arc<Mutex<Vec<SubAgentTask>>>,
    config: Config,
}

impl SubAgentPool {
    pub fn new(config: Config) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    /// Spawn a sub-agent to execute a task in background
    pub fn spawn(&self, description: String) -> String {
        let task_id = Uuid::new_v4().to_string();
        let task = SubAgentTask {
            id: task_id.clone(),
            description: description.clone(),
            status: "running".into(),
            result: None,
        };
        self.tasks.lock().expect("lock").push(task);

        let tasks = self.tasks.clone();
        let config = self.config.clone();
        let task_id_clone = task_id.clone();
        let desc_clone = description.clone();

        thread::spawn(move || {
            let result = execute_sub_task(&config, &desc_clone);
            if let Ok(mut guard) = tasks.lock() {
                if let Some(task) = guard.iter_mut().find(|t| t.id == task_id_clone) {
                    task.status = "completed".into();
                    task.result = result.ok();
                }
            }
        });

        task_id
    }

    /// Get status of all tasks
    pub fn status(&self) -> Vec<SubAgentTask> {
        self.tasks.lock().expect("lock").clone()
    }

    /// Get result of a specific task
    pub fn get_result(&self, task_id: &str) -> Option<SubAgentTask> {
        self.tasks
            .lock()
            .expect("lock")
            .iter()
            .find(|t| t.id == task_id)
            .cloned()
    }

    /// Merge results from completed sub-agents
    pub fn merge_results(&self) -> String {
        let guard = self.tasks.lock().expect("lock");
        let completed: Vec<_> = guard.iter().filter(|t| t.status == "completed").collect();
        if completed.is_empty() {
            return "No completed tasks.".into();
        }
        let mut merged = String::from("Sub-agent results:\n");
        for task in completed {
            merged.push_str(&format!("- {}: {}\n", task.description, task.result.as_deref().unwrap_or("(no result)")));
        }
        merged
    }
}

fn execute_sub_task(config: &Config, description: &str) -> Result<String> {
    // Simplified sub-agent: single-turn execution
    let store = Store::open(&config.db_path)?;
    let session_id = store.create_session()?;
    let workspace = Workspace::new(&config.workspace_root)?;
    let skills = Skills::new(&config.skills_root);
    let context = Context::load(&store, session_id, "You are a helpful sub-agent.")?;
    let tools = ToolRegistry::new(config.clone(), workspace, Store::open(&config.db_path)?, skills);
    let model = Model::new(&config.model.default_model);

    let messages = vec![json!({"role":"user","content":description})];
    let response = model.generate(&messages, &tools.schemas())?;

    if let Some(text) = response.text {
        Ok(text)
    } else {
        Ok("Sub-agent completed (no text response)".into())
    }
}
