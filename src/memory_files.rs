//! Generate curated memory files (MEMORY.md, USER.md) from SQLite.

use crate::store::Store;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct MemoryFiles {
    workspace_root: String,
}

impl MemoryFiles {
    pub fn new(workspace_root: &str) -> Self {
        Self {
            workspace_root: workspace_root.to_string(),
        }
    }

    /// Generate MEMORY.md and USER.md from SQLite
    pub fn generate(&self, store: &Store) -> Result<()> {
        // Generate MEMORY.md (facts)
        let memories = store.search_memories("", 100)?;
        let mut memory_content = String::from("# Long-term Memory\n\n");
        for mem in memories {
            memory_content.push_str(&format!("- {}\n", mem));
        }

        let memory_path = Path::new(&self.workspace_root).join("MEMORY.md");
        fs::write(memory_path, memory_content)?;

        // Generate USER.md (preferences)
        let preferences = store.search_memories("User preference:", 50)?;
        let mut user_content = String::from("# User Preferences\n\n");
        for pref in preferences {
            if let Some(fact) = pref.strip_prefix("User preference: ") {
                user_content.push_str(&format!("- {}\n", fact));
            }
        }

        let user_path = Path::new(&self.workspace_root).join("USER.md");
        fs::write(user_path, user_content)?;

        Ok(())
    }

    /// Load MEMORY.md and USER.md into prompt
    pub fn load_into_prompt(&self) -> String {
        let mut prompt = String::new();

        let memory_path = Path::new(&self.workspace_root).join("MEMORY.md");
        if memory_path.exists() {
            if let Ok(content) = fs::read_to_string(&memory_path) {
                prompt.push_str("\n\n## Memory\n");
                prompt.push_str(&content);
            }
        }

        let user_path = Path::new(&self.workspace_root).join("USER.md");
        if user_path.exists() {
            if let Ok(content) = fs::read_to_string(&user_path) {
                prompt.push_str("\n\n## User Preferences\n");
                prompt.push_str(&content);
            }
        }

        prompt
    }
}
