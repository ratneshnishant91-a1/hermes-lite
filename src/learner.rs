use crate::skills::Skills;
use crate::store::Store;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize)]
pub struct LearningLog {
    pub skills_created: Vec<Value>,
    pub memories_consolidated: Vec<Value>,
    pub lessons_learned: Vec<Value>,
    pub total_learning_events: u64,
}

pub struct SelfLearner {
    path: PathBuf,
    pub log: LearningLog,
}

impl SelfLearner {
    pub fn load(workspace_root: &str) -> Result<Self> {
        let path = PathBuf::from(workspace_root).join("learning_log.json");
        let log = if path.exists() {
            serde_json::from_str(&fs::read_to_string(&path)?)?
        } else {
            LearningLog::default()
        };
        Ok(Self { path, log })
    }

    fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, serde_json::to_string_pretty(&self.log)?)?;
        Ok(())
    }

    pub fn learn(
        &mut self,
        store: &Store,
        skills: &Skills,
        task: &str,
        messages: &[Value],
        result: &str,
        success: bool,
    ) -> Result<()> {
        for msg in messages {
            if msg.get("role").and_then(|r| r.as_str()) == Some("user") {
                if let Some(c) = msg.get("content").and_then(|c| c.as_str()) {
                    let lower = c.to_lowercase();
                    if ["i prefer", "i like", "always", "never"].iter().any(|k| lower.contains(k)) {
                        store.remember(&format!("User preference: {}", &c[..c.len().min(200)]), "learning")?;
                        self.log.memories_consolidated.push(serde_json::json!({
                            "timestamp": Utc::now().to_rfc3339(),
                            "fact": c
                        }));
                    }
                }
            }
        }
        if success && messages.len() > 8 {
            let name = format!(
                "auto_{}",
                task.split_whitespace()
                    .take(3)
                    .filter(|w| w.chars().all(|c| c.is_ascii_alphanumeric()))
                    .collect::<Vec<_>>()
                    .join("_")
                    .to_lowercase()
            );
            if !name.ends_with('_') && name.len() > 5 {
                let body = format!(
                    "---\nname: {name}\ndescription: Auto-generated from {task}\n---\n\n# {name}\n\n{result}\n"
                );
                skills.write_skill(&name, &body)?;
                self.log.skills_created.push(serde_json::json!({
                    "timestamp": Utc::now().to_rfc3339(),
                    "name": name,
                    "task": task
                }));
            }
        }
        self.log.lessons_learned.push(serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "task": task,
            "success": success
        }));
        self.log.total_learning_events += 1;
        self.save()
    }
}
