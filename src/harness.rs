use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HarnessManifest {
    pub version: u32,
    pub created_at: i64,
    pub prompt_policy: String,
    pub allowed_tools: Vec<String>,
    pub memory_policy: String,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct HarnessStore { root: PathBuf }

impl HarnessStore {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().join("harness");
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn initialize(&self) -> Result<HarnessManifest> {
        if let Some(existing) = self.current()? { return Ok(existing); }
        let manifest = HarnessManifest { version: 1, created_at: Utc::now().timestamp(), prompt_policy: "Validate input; use deterministic local capabilities first; require explicit integration for external effects.".into(), allowed_tools: vec!["read_file".into(), "write_file".into()], memory_policy: "Retain durable, user-approved facts; do not infer sensitive facts.".into(), notes: "Initial v2.5.1 harness.".into() };
        self.save(&manifest)?;
        Ok(manifest)
    }

    pub fn current(&self) -> Result<Option<HarnessManifest>> {
        let path = self.root.join("current.json");
        if !path.exists() { return Ok(None); }
        Ok(Some(serde_json::from_str(&fs::read_to_string(path).context("read harness manifest")?)?))
    }

    pub fn revise(&self, mut manifest: HarnessManifest) -> Result<HarnessManifest> {
        let current = self.initialize()?;
        if manifest.version <= current.version { manifest.version = current.version + 1; }
        manifest.created_at = Utc::now().timestamp();
        self.save(&manifest)?;
        Ok(manifest)
    }

    pub fn rollback(&self, version: u32) -> Result<HarnessManifest> {
        let archived = self.root.join(format!("v{version}.json"));
        if !archived.exists() { bail!("harness version {version} does not exist"); }
        let manifest: HarnessManifest = serde_json::from_str(&fs::read_to_string(archived)?)?;
        fs::write(self.root.join("current.json"), serde_json::to_string_pretty(&manifest)?)?;
        Ok(manifest)
    }

    fn save(&self, manifest: &HarnessManifest) -> Result<()> {
        let data = serde_json::to_string_pretty(manifest)?;
        fs::write(self.root.join(format!("v{}.json", manifest.version)), &data)?;
        fs::write(self.root.join("current.json"), data)?;
        Ok(())
    }
}
