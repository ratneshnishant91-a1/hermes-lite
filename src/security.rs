use std::{collections::HashMap, time::{Duration, Instant}};

pub fn validate_prompt(input: &str) -> Result<(), &'static str> {
    if input.trim().is_empty() { return Err("prompt is empty"); }
    if input.len() > 20_000 { return Err("prompt exceeds 20KB"); }
    if input.chars().any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t')) { return Err("prompt contains control characters"); }
    Ok(())
}

pub struct RateLimiter { maximum: usize, window: Duration, hits: HashMap<String, Vec<Instant>> }
impl RateLimiter {
    pub fn new(maximum: usize, window: Duration) -> Self { Self { maximum, window, hits: HashMap::new() } }
    pub fn allow(&mut self, key: &str) -> bool {
        let now = Instant::now();
        let values = self.hits.entry(key.to_owned()).or_default();
        values.retain(|v| now.duration_since(*v) < self.window);
        if values.len() >= self.maximum { return false; }
        values.push(now); true
    }
}
