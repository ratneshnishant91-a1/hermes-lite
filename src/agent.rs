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
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;

const SYSTEM: &str = r#"You are Hermes-Lite v2.0, a self-learning autonomous agent.
Use tools when needed. Stay inside workspace. Save facts with memory_save.
Be concise. Prefer skills and cached responses over LLM calls."#;

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
            llm_call_count: 0,
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

    /// Smart routing: pattern → math → cache → skill → LLM
    pub fn run(&mut self, user_message: &str) -> Result<String> {
        crate::security::InputValidator::validate_message(user_message).map_err(anyhow::Error::msg)?;
        
        self.current_task = Some(user_message.chars().take(200).collect());
        self.task_start = self.context.messages.len();

        // 1. Pattern matching (greetings, help, time)
        if let Some(response) = self.pattern_match(user_message) {
            return Ok(response);
        }

        // 2. Math evaluation (arithmetic, algebra, functions)
        if let Some(math_result) = self.evaluate_math(user_message) {
            return Ok(math_result);
        }

        // 3. Cache lookup
        let cache_key = format!("{:x}", md5::compute(user_message.as_bytes()));
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(format!("[cached] {}", cached));
        }

        // 4. Skill execution (direct tool calls)
        if let Some(skill_response) = self.try_skill_execution(user_message)? {
            self.cache.insert(cache_key, skill_response.clone());
            return Ok(skill_response);
        }

        // 5. LLM fallback (increment counter)
        self.llm_call_count += 1;
        self.context
            .add(&self.store, json!({"role":"user","content":user_message}))?;

        let mut final_text = String::from("Agent stopped: maximum tool iterations reached.");
        let mut success = false;
        let mut tool_calls_made = 0;
        
        for _ in 0..15 {
            let compressed = self.context.prompt_messages(20);
            let response = self.model.generate(&compressed, &self.tools.schemas())?;
            
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
                
                let result = if !approve(&name, &args) {
                    json!("Tool execution rejected.")
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

    /// Pattern matching for common queries
    fn pattern_match(&self, query: &str) -> Option<String> {
        let q = query.to_lowercase();
        
        if Regex::new(r"^(hi|hello|hey|greetings)").unwrap().is_match(&q) {
            return Some("Hello! How can I help you today?".into());
        }
        if Regex::new(r"(thank|thanks)").unwrap().is_match(&q) {
            return Some("You're welcome!".into());
        }
        if Regex::new(r"^(help|what can you do)").unwrap().is_match(&q) {
            return Some("I can: execute shell commands, read/write files, fetch URLs, search web, save memories, create skills, evaluate math. Just ask!".into());
        }
        if Regex::new(r"(status|health|are you ok)").unwrap().is_match(&q) {
            return Some("All systems operational.".into());
        }
        if Regex::new(r"(what time|current time|date now)").unwrap().is_match(&q) {
            return Some(format!("Current time: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        }
        None
    }

    /// Evaluate math expressions locally (no LLM)
    fn evaluate_math(&self, query: &str) -> Option<String> {
        // Extract math expression from query
        let patterns = [
            Regex::new(r"calculate\s+(.+)").unwrap(),
            Regex::new(r"what is\s+(.+[+\-*/^].+)").unwrap(),
            Regex::new(r"solve\s+(.+)").unwrap(),
            Regex::new(r"eval(uate)?\s+(.+)").unwrap(),
        ];
        
        let expr = patterns.iter()
            .filter_map(|re| re.captures(query).and_then(|c| c.get(1).map(|m| m.as_str())))
            .next()
            .or_else(|| {
                // If query is purely mathematical (e.g., "2+2*3")
                if query.chars().any(|c| "+-*/^().".contains(c)) && query.chars().any(|c| c.is_digit(10)) {
                    Some(query)
                } else {
                    None
                }
            })?;
        
        // Evaluate with meval crate
        match meval::eval_str(expr) {
            Ok(result) => {
                // Format nicely (remove trailing zeros)
                let formatted = if result.fract() == 0.0 {
                    format!("{}", result as i64)
                } else {
                    format!("{:.6}", result).trim_end_matches('0').trim_end_matches('.').to_string()
                };
                Some(format!("{} = {}", expr, formatted))
            }
            Err(_) => None,
        }
    }

    /// Direct skill/tool execution
    fn try_skill_execution(&self, query: &str) -> Result<Option<String>> {
        let q = query.to_lowercase();
        
        if let Some(path) = Regex::new(r"read file (\S+)").unwrap().captures(&q).and_then(|c| c.get(1)) {
            let result = self.tools.execute("read_file", &json!({"path": path.as_str()}))?;
            return Ok(Some(format!("File content: {}", result)));
        }
        
        if let Some(mem_query) = Regex::new(r"(remember|memory|recall) (.+)").unwrap().captures(&q).and_then(|c| c.get(2)) {
            let results = self.store.search_memories(mem_query.as_str(), 5)?;
            if results.is_empty() {
                return Ok(Some("No matching memories found.".into()));
            }
            return Ok(Some(format!("Memories: {}", results.join(" | "))));
        }
        
        Ok(None)
    }
}
