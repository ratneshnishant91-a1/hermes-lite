use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub gateway: GatewayConfig,
    #[serde(default)]
    pub sandbox: SandboxConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub model: ModelConfig,
    #[serde(default = "default_workspace")]
    pub workspace_root: String,
    #[serde(default = "default_db")]
    pub db_path: String,
    #[serde(default = "default_skills")]
    pub skills_root: String,
}

fn default_workspace() -> String {
    "workspace".into()
}
fn default_db() -> String {
    "hermes.db".into()
}
fn default_skills() -> String {
    "skills".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    #[serde(default = "default_bind")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub require_auth: bool,
}

fn default_bind() -> String {
    "127.0.0.1".into()
}
fn default_port() -> u16 {
    8000
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub network_enabled: bool,
}

fn default_mode() -> String {
    "subprocess".into()
}
fn default_timeout() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default = "default_max_bytes")]
    pub max_response_size: usize,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_max_bytes() -> usize {
    10 * 1024 * 1024
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    #[serde(default = "default_model")]
    pub default_model: String,
}

fn default_model() -> String {
    "gpt-4.1-mini".into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gateway: GatewayConfig::default(),
            sandbox: SandboxConfig::default(),
            network: NetworkConfig::default(),
            model: ModelConfig::default(),
            workspace_root: default_workspace(),
            db_path: default_db(),
            skills_root: default_skills(),
        }
    }
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            host: default_bind(),
            port: default_port(),
            require_auth: true,
        }
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            timeout: default_timeout(),
            network_enabled: false,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_domains: vec![],
            max_response_size: default_max_bytes(),
            timeout: default_timeout(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            default_model: default_model(),
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let cfg: Config = serde_yaml::from_str(&text).context("parse config.yaml")?;
        Ok(cfg)
    }
}
