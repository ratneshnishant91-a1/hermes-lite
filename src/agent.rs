use crate::approval::approve;
use crate::config::Config;
use crate::context::Context;
use crate::learner::SelfLearner;
use crate::math;
use crate::model::Model;
use crate::skills::Skills;
use crate::store::Store;
use crate::tools::ToolRegistry;
use crate::workspace::Workspace;
use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;

const SYSTEM: &str = r#"You are Hermes-Lite v2.0, a self-learning autonomous agent.
- Maintain a goal stack; decompose tasks into steps.
- Use tools reliably; retry transient failures.
- Save durable facts and skills; reuse skills for similar tasks.
- Reflect on errors; revise plans when stuck.
- Observe user constraints and preferences.
- Be concise. Prefer cached/skill responses over LLM calls."#;

#[derive(Debug, Clone)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub status: String,
    pub plan: Vec<String>,
    pub current_step: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Constraints {
    pub prefer_rust: bool,
    pub avoid_network: bool,
    pub max_tool_calls_per_turn: usize,
}

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
    constraints: Constraints,
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
        let tools = ToolRegistry::new(cfg.clone(), workspace, Store::open(&cfg.db_path)?, skills.clone());
        let learner = SelfLearner::load(&cfg.workspace_root)?;
        let model = Model::new(&cfg.model.default_model);
        let mut constraints = Constraints::default();
        if let Some(v) = store.get_constraint("avoid_network")? {
            constraints.avoid_network = v == "true";
        }
        if let Some(v) = store.get_constraint("prefer_rust")? {
            constraints.prefer_rust = v == "true";
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

        // 1. Pattern match
        if let Some(response) = self.pattern_match(user_message) {
            return Ok(response);
        }

        // 2. Math
        if let Some(math_result) = math::evaluate_query(user_message) {
            return Ok(math_result);
        }

        // 3. Cache
        let cache_key = format!("{:x}", md5::compute(user_message.as_bytes()));
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(format!("[cached] {}", cached));
        }

        // 4. Skill retrieval
        if let Some(skill_response) = self.try_apply_skill(user_message)? {
            self.cache.insert(cache_key, skill_response.clone());
            return Ok(skill_response);
        }

        // 5. Direct tool execution
        if let Some(skill_response) = self.try_skill_execution(user_message)? {
            self.cache.insert(cache_key, skill_response.clone());
            return Ok(skill_response);
        }

        // 6. LLM with reflection & constraints
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
                augmented.push(json!({"role":"system","content":"Constraint: avoid network tools (fetch_url, web_search)."}));
            }

            let response = self.model.generate(&augmented, &self.tools.schemas())?;

            if response.tool_calls.is_empty() {
                final_text = response.text.unwrap_or_default();
                self.context.add(&self.store, json!({"role":"assistant","content":final_text}))?;
                success = true;
                break;
            }

            for call in response.tool_calls {
                let name = call["function"]["name"].as_str().unwrap_or("").to_string();
                let args: Value = match call["function"]["arguments"].as_str() {
                    Some(s) => serde_json::from_str(s).unwrap_or(json!({})),
                    None => call["function"]["arguments"].clone(),
                };

                if self.constraints.avoid_network && ["fetch_url","web_search"].contains(&name.as_str()) {
                    last_error = Some("Network tools disabled by constraints.".into());
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
        if Regex::new(r"^(hi|hello|hey|greetings)").unwrap().is_match(&q) { return Some("Hello! How can I help you today?".into()); }
        if Regex::new(r"(thank|thanks)").unwrap().is_match(&q) { return Some("You're welcome!".into()); }
        if Regex::new(r"^(help|what can you do)").unwrap().is_match(&q) { return Some("I can: execute shell commands, read/write files, fetch URLs, search web, save memories, create skills, evaluate math. Just ask!".into()); }
        if Regex::new(r"(status|health|are you ok)").unwrap().is_match(&q) { return Some("All systems operational.".into()); }
        if Regex::new(r"(what time|current time|date now)").unwrap().is_match(&q) { return Some(format!("Current time: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))); }
        None
    }

    fn try_apply_skill(&self, query: &str) -> Result<Option<String>> {
        let relevant = self.skills.find_relevant(query);
        if let Some(name) = relevant.first() {
            let content = self.skills.load(name)?;
            return Ok(Some(format!("[skill: {}]\n{}", name, content)));
        }
        Ok(None)
    }

    fn try_skill_execution(&self, query: &str) -> Result<Option<String>> {
        let q = query.to_lowercase();
        if let Some(path) = Regex::new(r"read file (\S+)").unwrap().captures(&q).and_then(|c| c.get(1)) {
            let result = self.tools.execute("read_file", &json!({"path": path.as_str()}))?;
            return Ok(Some(format!("File content: {}", result)));
        }
        if let Some(mem_query) = Regex::new(r"(remember|memory|recall) (.+)").unwrap().captures(&q).and_then(|c| c.get(2)) {
            let results = self.store.search_memories(mem_query.as_str(), 5)?;
            if results.is_empty() { return Ok(Some("No matching memories found.".into())); }
            return Ok(Some(format!("Memories: {}", results.join(" | "))));
        }
        Ok(None)
    }

    pub fn list_goals(&self) -> Result<Vec<crate::store::GoalRecord>> {
        self.store.list_goals()
    }

    pub fn set_constraint(&self, key: &str, value: &str) -> Result<()> {
        self.store.set_constraint(key, value)
    }
}
