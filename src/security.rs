use regex::Regex;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct InputValidator;

impl InputValidator {
    pub fn validate_message(msg: &str) -> Result<(), String> {
        if msg.trim().is_empty() {
            return Err("Empty message".into());
        }
        if msg.len() > 20_000 {
            return Err("Message too long".into());
        }
        if msg.contains("../") || msg.contains("..\\") {
            return Err("Path traversal detected".into());
        }
        Ok(())
    }

    pub fn validate_filename(name: &str) -> Result<(), String> {
        let re = Regex::new(r"^[A-Za-z0-9._\-/]+$").unwrap();
        if name.contains("..") || !re.is_match(name) {
            return Err("Invalid filename".into());
        }
        Ok(())
    }
}

pub struct RateLimiter {
    max: usize,
    window: Duration,
    hits: HashMap<String, Vec<Instant>>,
}

impl RateLimiter {
    pub fn new(max: usize, window_secs: u64) -> Self {
        Self {
            max,
            window: Duration::from_secs(window_secs),
            hits: HashMap::new(),
        }
    }

    pub fn allow(&mut self, user: &str) -> bool {
        let now = Instant::now();
        let hits = self.hits.entry(user.to_string()).or_default();
        hits.retain(|t| now.duration_since(*t) < self.window);
        if hits.len() >= self.max {
            return false;
        }
        hits.push(now);
        true
    }
}
