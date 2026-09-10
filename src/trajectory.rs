use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub step_id: u64,
    pub pattern: String,
    pub input: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    pub steps: Vec<Step>,
    pub final_output: Option<String>,
    pub metadata: HashMap<String, Value>,
}

impl Trajectory {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            final_output: None,
            metadata: HashMap::new(),
        }
    }

    pub fn log_step(&mut self, step_id: u64, pattern: &str, input: &str) {
        self.steps.push(Step {
            step_id,
            pattern: pattern.to_string(),
            input: input.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        });
    }

    pub fn log_final(&mut self, output: &str) {
        self.final_output = Some(output.to_string());
    }

    pub fn set_metadata(&mut self, key: &str, value: Value) {
        self.metadata.insert(key.to_string(), value);
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
