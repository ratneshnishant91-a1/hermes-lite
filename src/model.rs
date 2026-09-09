use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::env;

#[derive(Debug, Clone, Default)]
pub struct ModelResponse {
    pub text: Option<String>,
    pub tool_calls: Vec<Value>,
    pub provider: String,
    pub model: String,
}

pub struct Model {
    model: String,
}

impl Model {
    pub fn new(default_model: &str) -> Self {
        Self {
            model: env::var("HERMES_MODEL")
                .unwrap_or_else(|_| default_model.to_string()),
        }
    }

    pub fn generate(&self, messages: &[Value], tools: &[Value]) -> Result<ModelResponse> {
        let (url, key, provider, model) = resolve_endpoint(&self.model)?;
        let mut body = json!({
            "model": model,
            "messages": messages,
        });
        if !tools.is_empty() {
            body["tools"] = json!(tools);
        }
        let resp: ChatResponse = ureq::post(&url)
            .set("Authorization", &format!("Bearer {key}"))
            .set("Content-Type", "application/json")
            .send_json(&body)
            .context("LLM request failed")?
            .into_json()
            .context("LLM JSON parse failed")?;
        let msg = resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message)
            .unwrap_or_default();
        Ok(ModelResponse {
            text: msg.content,
            tool_calls: msg.tool_calls.unwrap_or_default(),
            provider,
            model: model.to_string(),
        })
    }
}

fn resolve_endpoint(model: &str) -> Result<(String, String, String, String)> {
    if let Ok(key) = env::var("OPENROUTER_API_KEY") {
        return Ok((
            "https://openrouter.ai/api/v1/chat/completions".into(),
            key,
            "openrouter".into(),
            model.to_string(),
        ));
    }
    if let Ok(key) = env::var("OPENAI_API_KEY") {
        return Ok((
            "https://api.openai.com/v1/chat/completions".into(),
            key,
            "openai".into(),
            model.to_string(),
        ));
    }
    bail!("Set OPENAI_API_KEY or OPENROUTER_API_KEY")
}

#[derive(serde::Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}
#[derive(serde::Deserialize, Default)]
struct Choice {
    message: Msg,
}
#[derive(serde::Deserialize, Default)]
struct Msg {
    content: Option<String>,
    tool_calls: Option<Vec<Value>>,
}
