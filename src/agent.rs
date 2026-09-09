use crate::approval::approve;
use crate::config::Config;
use crate::context::Context;
use crate::learner::SelfLearner;
use crate::model::Model;
use crate::skills::Skills;
use crate::store::Store;
use crate::tools::ToolRegistry;
use crate::workspace::Workspace;
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;

const SYSTEM: &str = r#"You are Hermes-Lite v2.0, a self-learning autonomous agent running on a Rust core.
Use tools when they help. Stay inside the workspace. Save durable facts with memory_save.
If a listed skill matches the task, skill_load it first. When finished, give a concise final answer.
Prefer using cached responses and skills over calling the LLM repeatedly."#;

pub struct Agent {
    store: Store,
    skills: Skills,
    model: Model,
    tools: ToolRegistry,
    context: Context,
    learner: SelfLearner,
    task_start: usize,
    current_task: Option<String>,
    cache: HashMap<String, String>,
}

impl Agent {
    pub fn new(cfg: Config) -> Result<Self> {
        let store = Store::open(&cfg.db_path)?;
        let workspace = Workspace::new(&cfg.workspace_root)?;
        let skills = Skills::new(&cfg.skills_root);
        let session_id = match store.latest_session()? {
            Some(id) if id > 0 => id,
            _ => store.create_session()?,
        };
        let mut prompt = format!("{SYSTEM}\n\n{}", skills.catalog_text());
        if let Ok(facts) = store.search_memories("", 8) {
            if !facts.is_empty() {
                prompt.push_str("\n\nKnown memories:\n");
                for f in facts {
                    prompt.push_str(&format!("- {f}\n"));
                }
            }
        }
        let context = Context::load(&store, session_id, &prompt)?;
        let tools = ToolRegistry::new(
            cfg.clone(),
            workspace,
            Store::open(&cfg.db_path)?,
            skills.clone(),
        );
        let learner = SelfLearner::load(&cfg.workspace_root)?;
        let model = Model::new(&cfg.model.default_model);
        Ok(Self {
            store,
            skills,
            model,
            tools,
            context,
            learner,
            task_start: 1,
            current_task: None,
            cache: HashMap::new(),
        })
    }

    pub fn session_id(&self) -> i64 {
        self.context.session_id
    }

    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    pub fn learning_stats(&self) -> Value {
        json!({
            "total_events": self.learner.log.total_learning_events,
            "skills_created": self.learner.log.skills_created.len(),
            "memories_consolidated": self.learner.log.memories_consolidated.len(),
            "lessons_learned": self.learner.log.lessons_learned.len(),
            "cache_size": self.cache.len()
        })
    }

    /// Check cache first, then skills, then LLM
    pub fn run(&mut self, user_message: &str) -> Result<String> {
        crate::security::InputValidator::validate_message(user_message).map_err(anyhow::Error::msg)?;
        
        // Check cache for similar queries (simple hash-based)
        let cache_key = format!("{:x}", md5::compute(user_message.as_bytes()));
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(format!("[cached] {}", cached));
        }

        self.current_task = Some(user_message.chars().take(200).collect());
        self.task_start = self.context.messages.len();
        self.context
            .add(&self.store, json!({"role":"user","content":user_message}))?;

        let mut final_text = String::from("Agent stopped: maximum tool iterations reached.");
        let mut success = false;
        let mut tool_calls_made = 0;
        
        for _ in 0..20 {
            let response = self
                .model
                .generate(&self.context.prompt_messages(30), &self.tools.schemas())?;
            
            if response.tool_calls.is_empty() {
                final_text = response.text.unwrap_or_default();
                self.context.add(
                    &self.store,
                    json!({"role":"assistant","content":final_text}),
                )?;
                success = true;
                break;
            }
            
            for call in response.tool_calls {
                let name = call["function"]["name"].as_str().unwrap_or("").to_string();
                let args: Value = match call["function"]["arguments"].as_str() {
                    Some(s) => serde_json::from_str(s).unwrap_or(json!({})),
                    None => call["function"]["arguments"].clone(),
                };
                
                // Auto-load skills if tool is skill_load
                if name == "skill_load" {
                    if let Some(skill_name) = args.get("name").and_then(|v| v.as_str()) {
                        // Skill will be loaded by tool execution
                    }
                }
                
                let result = if !approve(&name, &args) {
                    json!("Tool execution rejected by user.")
                } else {
                    match self.tools.execute(&name, &args) {
                        Ok(v) => {
                            tool_calls_made += 1;
                            v
                        }
                        Err(err) => json!({"error": err.to_string()}),
                    }
                };
                self.context.add(
                    &self.store,
                    json!({"role":"assistant","tool_calls":[call.clone()]}),
                )?;
                self.context.add(
                    &self.store,
                    json!({
                        "role":"tool",
                        "tool_call_id": call.get("id").cloned().unwrap_or(json!("call")),
                        "name": name,
                        "content": result.to_string()
                    }),
                )?;
            }
        }
        
        // Cache the response
        self.cache.insert(cache_key, final_text.clone());
        
        // Learn only if we made tool calls (meaningful interaction)
        if tool_calls_made > 0 {
            if let Some(task) = self.current_task.clone() {
                let start = self.task_start.min(self.context.messages.len());
                let msgs = self.context.messages[start..].to_vec();
                let _ = self
                    .learner
                    .learn(&self.store, &self.skills, &task, &msgs, &final_text, success);
            }
        }
        
        self.current_task = None;
        Ok(final_text)
    }
}
