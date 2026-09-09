use crate::store::{Message, Store};
use anyhow::Result;
use serde_json::{json, Value};

pub struct Context {
    pub session_id: i64,
    pub messages: Vec<Value>,
}

impl Context {
    pub fn load(store: &Store, session_id: i64, system: &str) -> Result<Self> {
        let saved = store.load_messages(session_id)?;
        let mut messages = if saved.is_empty() {
            let sys = json!({"role":"system","content":system});
            store.save_message(
                session_id,
                &Message {
                    role: "system".into(),
                    content: Some(system.into()),
                    tool_call_id: None,
                    name: None,
                    tool_calls: None,
                },
            )?;
            vec![sys]
        } else {
            saved.into_iter().map(message_to_json).collect()
        };
        if messages.first().and_then(|m| m.get("role")).and_then(|r| r.as_str()) != Some("system") {
            messages.insert(0, json!({"role":"system","content":system}));
        }
        Ok(Self {
            session_id,
            messages,
        })
    }

    pub fn add(&mut self, store: &Store, msg: Value) -> Result<()> {
        store.save_message(self.session_id, &json_to_message(&msg))?;
        self.messages.push(msg);
        Ok(())
    }

    pub fn prompt_messages(&self, maximum: usize) -> Vec<Value> {
        if self.messages.len() <= maximum {
            return self.messages.clone();
        }
        let mut out = vec![self.messages[0].clone()];
        out.push(json!({
            "role":"system",
            "content":"Earlier conversation was compressed. Use recent context."
        }));
        let start = self.messages.len().saturating_sub(maximum);
        out.extend(self.messages[start..].iter().cloned());
        out
    }
}

fn message_to_json(m: Message) -> Value {
    let mut v = json!({"role": m.role});
    if let Some(c) = m.content {
        v["content"] = json!(c);
    }
    if let Some(id) = m.tool_call_id {
        v["tool_call_id"] = json!(id);
    }
    if let Some(n) = m.name {
        v["name"] = json!(n);
    }
    if let Some(tc) = m.tool_calls {
        v["tool_calls"] = tc;
    }
    v
}

fn json_to_message(v: &Value) -> Message {
    Message {
        role: v.get("role").and_then(|r| r.as_str()).unwrap_or("user").into(),
        content: v.get("content").and_then(|c| c.as_str()).map(|s| s.to_string()),
        tool_call_id: v.get("tool_call_id").and_then(|c| c.as_str()).map(|s| s.to_string()),
        name: v.get("name").and_then(|c| c.as_str()).map(|s| s.to_string()),
        tool_calls: v.get("tool_calls").cloned(),
    }
}
