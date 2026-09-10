use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "workspace_default")]
    pub workspace_root: String,
    #[serde(default = "db_default")]
    pub db_path: String,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub sandbox: SandboxConfig,
}

fn workspace_default() -> String { "workspace".into() }
fn db_default() -> String { "hermes.db".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self { Self { enabled: false, allowed_domains: Vec::new() } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    #[serde(default = "timeout_default")]
    pub timeout_secs: u64,
}

fn timeout_default() -> u64 { 10 }
impl Default for SandboxConfig { fn default() -> Self { Self { timeout_secs: timeout_default() } } }

impl Default for Config {
    fn default() -> Self {
        Self { workspace_root: workspace_default(), db_path: db_default(), network: NetworkConfig::default(), sandbox: SandboxConfig::default() }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let p = path.as_ref();
        let text = fs::read_to_string(p).with_context(|| format!("read {}", p.display()))?;
        Ok(serde_yaml::from_str(&text).context("parse config")?)
    }
}
