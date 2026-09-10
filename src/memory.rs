use crate::store::Store;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryObject {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub accessed_at: i64,
    pub access_count: u64,
}

pub struct Memory {
    store: Store,
    cache: HashMap<String, MemoryObject>,
}

impl Memory {
    pub fn new(store: Store) -> Self {
        Self { store, cache: HashMap::new() }
    }

    pub fn add(&mut self, content: &str, tags: Vec<String>) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let obj = MemoryObject {
            id: id.clone(),
            content: content.to_string(),
            tags,
            created_at: now,
            accessed_at: now,
            access_count: 0,
        };
        self.cache.insert(id.clone(), obj.clone());
        self.store.remember(&id, &serde_json::to_string(&obj)?)?;
        Ok(id)
    }

    pub fn get(&mut self, id: &str) -> Result<Option<MemoryObject>> {
        if let Some(obj) = self.cache.get(id) {
            let mut obj = obj.clone();
            obj.accessed_at = chrono::Utc::now().timestamp();
            obj.access_count += 1;
            self.cache.insert(id.to_string(), obj.clone());
            self.store.remember(id, &serde_json::to_string(&obj)?)?;
            return Ok(Some(obj));
        }
        if let Some(serialized) = self.store.recall(id)? {
            let obj: MemoryObject = serde_json::from_str(&serialized)?;
            self.cache.insert(id.to_string(), obj.clone());
            return Ok(Some(obj));
        }
        Ok(None)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryObject>> {
        let mut results = Vec::new();
        for obj in self.cache.values() {
            if obj.content.contains(query) || obj.tags.iter().any(|t| t.contains(query)) {
                results.push(obj.clone());
                if results.len() >= limit {
                    break;
                }
            }
        }
        Ok(results)
    }

    pub fn consolidate(&mut self) -> Result<usize> {
        let now = chrono::Utc::now().timestamp();
        let mut removed = 0;
        let mut to_remove = Vec::new();
        for (id, obj) in &self.cache {
            if now - obj.accessed_at > 86400 * 7 && obj.access_count < 3 {
                to_remove.push(id.clone());
            }
        }
        for id in to_remove {
            self.cache.remove(&id);
            removed += 1;
        }
        Ok(removed)
    }
}
