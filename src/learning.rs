use crate::events::Event;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailureCluster {
    pub signature: String,
    pub count: usize,
    pub latest_at: i64,
    pub suggested_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LearningReport {
    pub events_examined: usize,
    pub failures: Vec<FailureCluster>,
}

pub fn analyze(events: &[Event]) -> LearningReport {
    let mut clusters: BTreeMap<String, FailureCluster> = BTreeMap::new();
    for event in events.iter().filter(|e| e.success == Some(false)) {
        let signature = signature(&event.message);
        let entry = clusters.entry(signature.clone()).or_insert_with(|| FailureCluster { suggested_action: suggestion(&signature), signature, count: 0, latest_at: event.at });
        entry.count += 1;
        entry.latest_at = entry.latest_at.max(event.at);
    }
    LearningReport { events_examined: events.len(), failures: clusters.into_values().collect() }
}

fn signature(message: &str) -> String {
    let normalized: String = message.to_lowercase().chars().map(|c| if c.is_ascii_digit() { '#' } else { c }).collect();
    normalized.split_whitespace().take(12).collect::<Vec<_>>().join(" ")
}

fn suggestion(signature: &str) -> String {
    if signature.contains("path") || signature.contains("workspace") { "Check workspace path validation and use a relative path.".into() }
    else if signature.contains("timeout") { "Reduce task scope or raise the explicit tool timeout after review.".into() }
    else if signature.contains("unknown tool") { "Register the tool explicitly or correct the tool name.".into() }
    else { "Add a focused regression test before changing the harness or policy.".into() }
}
