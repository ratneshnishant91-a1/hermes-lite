use crate::{workspace::Workspace, Config};
use anyhow::{bail, Result};
use std::process::Command;
use std::time::Duration;

pub struct Tools {
    workspace: Workspace,
    timeout: Duration,
}

impl Tools {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            workspace: Workspace::new(&config.workspace_root)?,
            timeout: Duration::from_secs(config.sandbox.timeout_secs.clamp(1, 30)),
        })
    }

    pub fn run_shell(&self, command: &str) -> Result<String> {
        let forbidden = ["rm -rf /", "mkfs", "shutdown", "reboot", "dd if="];
        if forbidden.iter().any(|v| command.contains(v)) {
            bail!("blocked dangerous command");
        }
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(self.workspace.resolve(".")?)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .output()?;

        let mut res = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !output.stderr.is_empty() {
            if !res.is_empty() {
                res.push('\n');
            }
            res.push_str(&String::from_utf8_lossy(&output.stderr).trim());
        }
        if res.is_empty() {
            res = format!("exit={}", output.status.code().unwrap_or(-1));
        }
        Ok(res)
    }

    pub fn read_file(&self, path: &str) -> Result<String> {
        self.workspace.read(path)
    }

    pub fn write_file(&self, path: &str, body: &str) -> Result<()> {
        self.workspace.write(path, body)
    }

    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}
