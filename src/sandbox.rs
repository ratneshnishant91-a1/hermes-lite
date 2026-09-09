use crate::config::SandboxConfig;
use anyhow::Result;
use serde::Serialize;
use std::collections::HashSet;
use std::process::Command;
use std::time::Duration;

const SAFE_ENV: &[&str] = &["PATH", "HOME", "USER", "LANG", "LC_ALL", "TERM", "SHELL", "TMPDIR", "PWD"];

#[derive(Debug, Serialize)]
pub struct SandboxResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timeout: bool,
}

pub fn run_shell(cfg: &SandboxConfig, command: &str, cwd: &str) -> Result<SandboxResult> {
    let timeout = Duration::from_secs(cfg.timeout.max(1));
    let mut cmd = if cfg.network_enabled {
        Command::new("bash")
    } else {
        Command::new("bash")
    };
    cmd.arg("-c").arg(command).current_dir(cwd);
    cmd.env_clear();
    let allow: HashSet<&str> = SAFE_ENV.iter().copied().collect();
    for (k, v) in std::env::vars() {
        if allow.contains(k.as_str()) {
            cmd.env(k, v);
        }
    }

    // Best-effort timeout using wait_timeout-like polling via spawn + try_wait.
    let mut child = cmd.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn()?;
    let start = std::time::Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let out = child.wait_with_output()?;
            return Ok(SandboxResult {
                exit_code: status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
                timeout: false,
            });
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(SandboxResult {
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("timed out after {}s", cfg.timeout),
                timeout: true,
            });
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
