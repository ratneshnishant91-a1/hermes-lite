use crate::{workspace::Workspace, Config};
use anyhow::{bail, Result};
use std::{process::Command, time::{Duration, Instant}};

pub struct Tools { workspace: Workspace, timeout: Duration }
impl Tools {
    pub fn new(config: &Config) -> Result<Self> { Ok(Self { workspace: Workspace::new(&config.workspace_root)?, timeout: Duration::from_secs(config.sandbox.timeout_secs.clamp(1, 30)) }) }
    pub fn run_shell(&self, command: &str) -> Result<String> {
        let forbidden = ["rm -rf /", "mkfs", "shutdown", "reboot", "dd if="];
        if forbidden.iter().any(|v| command.contains(v)) { bail!("blocked dangerous command"); }
        let mut child = Command::new("sh").arg("-c").arg(command).current_dir(self.workspace.resolve(".")?).env_clear().env("PATH", "/usr/bin:/bin").spawn()?;
        let started=Instant::now();
        loop { if let Some(status)=child.try_wait()? { return Ok(format!("exit={}", status.code().unwrap_or(-1))); } if started.elapsed()>self.timeout { child.kill()?; return Ok("timed out".into()); } std::thread::sleep(Duration::from_millis(20)); }
    }
    pub fn read_file(&self, path:&str) -> Result<String> { self.workspace.read(path) }
    pub fn write_file(&self, path:&str, body:&str) -> Result<()> { self.workspace.write(path,body) }
}
