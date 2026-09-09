//! Executor that runs plan steps with tool calls.

use crate::tools::ToolRegistry;
use crate::planner::{Plan, Step, Artifact};
use serde_json::{json, Value};
use anyhow::Result;

pub struct Executor {
    tools: ToolRegistry,
}

impl Executor {
    pub fn new(tools: ToolRegistry) -> Self {
        Self { tools }
    }

    /// Execute a plan step by step
    pub fn execute_plan(&self, plan: &mut Plan) -> Result<Vec<String>> {
        let mut outputs = Vec::new();

        for step in &mut plan.steps {
            if step.status == "completed" {
                continue;
            }

            step.status = "running".into();

            // Execute step based on tool
            let output = if let Some(tool_name) = &step.tool {
                // Simple execution: assume args are in description
                // In real impl, would parse args from step
                let args = json!({"command": step.description});
                match self.tools.execute(tool_name, &args) {
                    Ok(result) => {
                        step.status = "completed".into();
                        format!("Step {}: {}", step.id, result)
                    }
                    Err(e) => {
                        step.status = "failed".into();
                        format!("Step {} failed: {}", step.id, e)
                    }
                }
            } else {
                step.status = "completed".into();
                format!("Step {}: {}", step.id, step.description)
            };

            outputs.push(output);
        }

        // Verify artifacts
        for artifact in &mut plan.artifacts {
            artifact.verified = self.verify_artifact(artifact).unwrap_or(false);
        }

        Ok(outputs)
    }

    fn verify_artifact(&self, artifact: &Artifact) -> Result<bool> {
        match artifact.kind.as_str() {
            "file" => Ok(std::path::Path::new(&artifact.path_or_url).exists()),
            "url" => Ok(ureq::head(&artifact.path_or_url).call().is_ok()),
            _ => Ok(true),
        }
    }
}
