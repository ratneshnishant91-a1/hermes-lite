use crate::approval::approve;
use crate::config::Config;
use crate::context::Context;
use crate::executor::Executor;
use crate::learner::SelfLearner;
use crate::math;
use crate::memory_files::MemoryFiles;
use crate::model::Model;
use crate::planner::Planner;
use crate::roles::AgentRole;
use crate::skills::Skills;
use crate::store::Store;
use crate::tools::ToolRegistry;
use crate::workspace::Workspace;
use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;

const SYSTEM: &str = r#"You are Hermes-Lite v2.0, a self-learning autonomous agent.
- Use planner/executor pattern for complex tasks.
- Verify artifacts before accepting results.
- Obey role restrictions.
- Save durable facts and skills.
- Be concise."#;

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
    llm_call_count: u64,
    constraints: super::agent::Constraints,
    role: AgentRole,
    planner: Option<Planner>,
    memory_files: MemoryFiles,
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

        // Generate memory files
        let memory_files = MemoryFiles::new(&cfg.workspace_root);
        let _ = memory_files.generate(&store);

        let mut prompt = format!("{SYSTEM}\n\n{}", skills.catalog_text());
        prompt.push_str(&memory_files.load_into_prompt());

        let context = Context::load(&store, session_id, &prompt)?;
        let tools = ToolRegistry::new(cfg.clone(), workspace, Store::open(&cfg.db_path)?, skills.clone());
        let learner = SelfLearner::load(&cfg.workspace_root)?;
        let model = Model::new(&cfg.model.default_model);
        let planner = Planner::new(model.clone());

        let mut constraints = super::agent::Constraints::default();
        if let Some(v) = store.get_constraint("avoid_network")? {
            constraints.avoid_network = v == "true";
        }

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
            llm_call_count: 0,
            constraints,
            role: AgentRole::Implementer, // Default role
            planner: Some(planner),
            memory_files,
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
            "cache_size": self.cache.len(),
            "llm_calls": self.llm_call_count
        })
    }

    pub fn run(&mut self, user_message: &str) -> Result<String> {
        crate::security::InputValidator::validate_message(user_message).map_err(anyhow::Error::msg)?;
        self.current_task = Some(user_message.chars().take(200).collect());
        self.task_start = self.context.messages.len();

        // Pattern match
        if let Some(response) = self.pattern_match(user_message) {
            return Ok(response);
        }

        // Math
        if let Some(math_result) = math::evaluate_query(user_message) {
            return Ok(math_result);
        }

        // Cache
        let cache_key = format!("{:x}", md5::compute(user_message.as_bytes()));
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(format!("[cached] {}", cached));
        }

        // For complex tasks, use planner/executor
        if user_message.len() > 50 && !self.cache.contains_key(&cache_key) {
            if let Some(ref planner) = self.planner {
                if let Ok(plan) = planner.create_plan(user_message, &self.context.prompt_messages(10)) {
                    // Execute plan
                    let executor = Executor::new(self.tools.clone());
                    let mut plan_mut = plan;
                    let outputs = executor.execute_plan(&mut plan_mut)?;

                    // Verify artifacts
                    let verified = plan_mut.artifacts.iter().all(|a| a.verified);
                    if verified {
                        let result = outputs.join("\n");
                        self.cache.insert(cache_key, result.clone());
                        return Ok(result);
                    }
                }
            }
        }

        // Fall back to direct execution
        self.llm_call_count += 1;
        self.context.add(&self.store, json!({"role":"user","content":user_message}))?;

        let mut final_text = String::from("Agent stopped: maximum tool iterations reached.");
        let mut success = false;
        let mut tool_calls_made = 0;
        let mut last_error: Option<String> = None;

        for _turn in 0..15 {
            let compressed = self.context.prompt_messages(20);
            let mut augmented = compressed.clone();
            if let Some(err) = &last_error {
                augmented.push(json!({"role":"system","content":format!("Last error: {}. Reflect and revise.", err)}));
            }
            if self.constraints.avoid_network {
                augmented.push(json!({"role":"system","content":"Constraint: avoid network tools."}));
            }
            augmented.push(json!({"role":"system","content":format!("Your role: {:?}. Allowed tools: {:?}", self.role, self.role.allowed_tools())}));

            let response = self.model.generate(&augmented, &self.tools.schemas())?;

            if response.tool_calls.is_empty() {
                final_text = response.text.unwrap_or_default();
                self.context.add(&self.store, json!({"role":"assistant","content":final_text}))?;
                success = true;
                break;
            }

            for call in response.tool_calls {
                let name = call["function"]["name"].as_str().unwrap_or("").to_string();

                // Enforce role restrictions
                if !self.role.allows(&name) {
                    last_error = Some(format!("Tool {} not allowed for role {:?}", name, self.role));
                    continue;
                }

                let args: Value = match call["function"]["arguments"].as_str() {
                    Some(s) => serde_json::from_str(s).unwrap_or(json!({})),
                    None => call["function"]["arguments"].clone(),
                };

                if self.constraints.avoid_network && ["fetch_url","web_search"].contains(&name.as_str()) {
                    last_error = Some("Network tools disabled.".into());
                    continue;
                }

                let result = if !approve(&name, &args) {
                    json!("Tool execution rejected.")
                } else {
                    match self.tools.execute(&name, &args) {
                        Ok(v) => { tool_calls_made += 1; v }
                        Err(err) => { last_error = Some(err.to_string()); json!({"error": err.to_string()}) }
                    }
                };
                self.context.add(&self.store, json!({"role":"assistant","tool_calls":[call.clone()]}))?;
                self.context.add(&self.store, json!({"role":"tool","tool_call_id":call.get("id").cloned().unwrap_or(json!("call")),"name":name,"content":result.to_string()}))?;
            }
        }

        self.cache.insert(cache_key, final_text.clone());

        if tool_calls_made > 0 {
            if let Some(task) = self.current_task.clone() {
                let start = self.task_start.min(self.context.messages.len());
                let msgs = self.context.messages[start..].to_vec();
                let _ = self.learner.learn(&self.store, &self.skills, &task, &msgs, &final_text, success);
            }
        }

        self.current_task = None;
        Ok(final_text)
    }

    fn pattern_match(&self, query: &str) -> Option<String> {
        let q = query.to_lowercase();
        if Regex::new(r"^(hi|hello|hey|greetings)").unwrap().is_match(&q) { return Some("Hello!".into()); }
        if Regex::new(r"(thank|thanks)").unwrap().is_match(&q) { return Some("You're welcome!".into()); }
        if Regex::new(r"^(help|what can you do)").unwrap().is_match(&q) { return Some("I can: shell, files, fetch, search, memory, skills, math.".into()); }
        if Regex::new(r"(status|health|are you ok)").unwrap().is_match(&q) { return Some("All systems operational.".into()); }
        if Regex::new(r"(what time|current time|date now)").unwrap().is_match(&q) { return Some(format!("Current time: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))); }
        None
    }

    pub fn set_role(&mut self, role: AgentRole) {
        self.role = role;
    }

    pub fn list_goals(&self) -> Result<Vec<crate::store::GoalRecord>> {
        self.store.list_goals()
    }

    pub fn set_constraint(&self, key: &str, value: &str) -> Result<()> {
        self.store.set_constraint(key, value)
    }
}
