use crate::config::Config;
use crate::network::safe_fetch;
use crate::sandbox;
use crate::skills::Skills;
use crate::store::Store;
use crate::workspace::Workspace;
use anyhow::{bail, Result};
use serde_json::{json, Value};

pub struct ToolRegistry {
    cfg: Config,
    workspace: Workspace,
    store: Store,
    skills: Skills,
}

impl ToolRegistry {
    pub fn new(cfg: Config, workspace: Workspace, store: Store, skills: Skills) -> Self {
        Self {
            cfg,
            workspace,
            store,
            skills,
        }
    }

    pub fn schemas(&self) -> Vec<Value> {
        [
            ("shell", "Execute a shell command in the workspace."),
            ("read_file", "Read a text file from the workspace."),
            ("write_file", "Write a text file to the workspace."),
            ("fetch_url", "Fetch a web page over HTTPS."),
            ("web_search", "Search the web via DuckDuckGo."),
            ("memory_save", "Save a long-term fact."),
            ("memory_search", "Search long-term memory."),
            ("skill_load", "Load a SKILL.md by name."),
            ("skill_create", "Create a new skill from a task."),
        ]
        .into_iter()
        .map(|(name, description)| {
            json!({
                "type": "function",
                "function": {
                    "name": name,
                    "description": description,
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "command": {"type": "string"},
                            "path": {"type": "string"},
                            "content": {"type": "string"},
                            "url": {"type": "string"},
                            "query": {"type": "string"},
                            "name": {"type": "string"},
                            "limit": {"type": "integer"},
                            "task": {"type": "string"},
                            "result": {"type": "string"}
                        }
                    }
                }
            })
        })
        .collect()
    }

    pub fn execute(&self, name: &str, args: &Value) -> Result<Value> {
        match name {
            "shell" => {
                let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
                let cwd = self.workspace.files.to_string_lossy().into_owned();
                let result = sandbox::run_shell(&self.cfg.sandbox, command, &cwd)?;
                Ok(serde_json::to_value(result)?)
            }
            "read_file" => {
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                Ok(json!(self.workspace.read(path)?))
            }
            "write_file" => {
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
                Ok(json!(self.workspace.write(path, content)?))
            }
            "fetch_url" => {
                let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
                Ok(safe_fetch(url, &self.cfg.network)?)
            }
            "web_search" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let encoded = encode(query);
                let url = format!("https://html.duckduckgo.com/html/?q={encoded}");
                Ok(safe_fetch(&url, &self.cfg.network)?)
            }
            "memory_save" => {
                let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
                Ok(json!(self.store.remember(content, "agent")?))
            }
            "memory_search" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(5);
                Ok(json!(self.store.search_memories(query, limit)?))
            }
            "skill_load" => {
                let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
                Ok(json!(self.skills.load(name)?))
            }
            "skill_create" => {
                let task = args.get("task").and_then(|v| v.as_str()).unwrap_or("");
                let result = args.get("result").and_then(|v| v.as_str()).unwrap_or("");
                let name = format!(
                    "auto_{}",
                    task.split_whitespace()
                        .take(3)
                        .filter(|w| w.chars().all(|c| c.is_ascii_alphanumeric()))
                        .collect::<Vec<_>>()
                        .join("_")
                        .to_lowercase()
                );
                let body = format!(
                    "---\nname: {name}\ndescription: Auto-generated from {task}\n---\n\n# {name}\n\n{result}\n"
                );
                self.skills.write_skill(&name, &body)?;
                Ok(json!({"skill": name, "task": task}))
            }
            other => bail!("Unknown tool: {other}"),
        }
    }
}

fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
