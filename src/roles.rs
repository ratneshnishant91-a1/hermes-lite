//! Role specialization for agents.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentRole {
    /// Research: web search, fetch URLs, read files
    Researcher,
    /// Implementation: shell, write files, execute code
    Implementer,
    /// Review: read files, verify artifacts
    Reviewer,
    /// QA: run tests, verify results
    QA,
}

impl AgentRole {
    /// Get allowed tools for this role
    pub fn allowed_tools(&self) -> Vec<&'static str> {
        match self {
            AgentRole::Researcher => vec!["web_search", "fetch_url", "read_file", "memory_search"],
            AgentRole::Implementer => vec!["shell", "write_file", "read_file", "skill_load"],
            AgentRole::Reviewer => vec!["read_file", "memory_search", "skill_load"],
            AgentRole::QA => vec!["shell", "read_file", "write_file"],
        }
    }

    /// Check if tool is allowed for this role
    pub fn allows(&self, tool: &str) -> bool {
        self.allowed_tools().contains(&tool)
    }
}

/// Worker agent with specific role
pub struct Worker {
    pub role: AgentRole,
    pub task: String,
    pub result: Option<String>,
}

impl Worker {
    pub fn new(role: AgentRole, task: String) -> Self {
        Self { role, task, result: None }
    }
}
