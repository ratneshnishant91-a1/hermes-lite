//! Explicit planner that creates structured plans before execution.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::model::Model;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub goal: String,
    pub steps: Vec<Step>,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: usize,
    pub description: String,
    pub tool: Option<String>,
    pub status: String, // pending, running, completed, failed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub kind: String, // file, url, test_result
    pub path_or_url: String,
    pub verified: bool,
}

pub struct Planner {
    model: Model,
}

impl Planner {
    pub fn new(model: Model) -> Self {
        Self { model }
    }

    /// Create a structured plan for a goal
    pub fn create_plan(&self, goal: &str, context: &[Value]) -> Result<Plan, anyhow::Error> {
        let system = r#"You are a planner. Create a structured plan with:
- Clear goal
- Sequential steps (each with optional tool)
- Expected artifacts (files, URLs, test results)

Respond in JSON: {"goal": "...", "steps": [{"id": 1, "description": "...", "tool": "shell"}], "artifacts": [{"kind": "file", "path_or_url": "report.txt", "verified": false}]}"#;

        let mut messages = vec![json!({"role":"system","content":system})];
        messages.extend(context.iter().cloned());
        messages.push(json!({"role":"user","content":goal}));

        let response = self.model.generate(&messages, &[])?;
        let text = response.text.ok_or_else(|| anyhow::anyhow!("No plan generated"))?;

        // Extract JSON from response
        let json_start = text.find('{').unwrap_or(0);
        let json_end = text.rfind('}').map(|i| i + 1).unwrap_or(text.len());
        let json_str = &text[json_start..json_end];

        let plan: Plan = serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse plan: {}", e))?;

        Ok(plan)
    }

    /// Verify an artifact exists and is valid
    pub fn verify_artifact(&self, artifact: &Artifact) -> Result<bool, anyhow::Error> {
        match artifact.kind.as_str() {
            "file" => {
                // Check file exists
                let exists = std::path::Path::new(&artifact.path_or_url).exists();
                Ok(exists)
            }
            "url" => {
                // Check URL is reachable (simple HEAD request)
                let result = ureq::head(&artifact.path_or_url).call();
                Ok(result.is_ok())
            }
            "test_result" => {
                // Assume verified if present
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
