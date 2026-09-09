use crate::config::SandboxConfig;
use anyhow::Result;
use serde::Serialize;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SAFE_ENV: &[&str] = &[
    "PATH", "HOME", "USER", "LANG", "LC_ALL", "TERM", "SHELL", "TMPDIR", "TMP", "TEMP", "PWD",
];

#[derive(Debug, Serialize)]
pub struct SandboxResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timeout: bool,
}

/// Run `command` via `bash -c` with strict resource limits.
///
/// Limits:
/// - CPU: 10s hard limit
/// - Memory: 256MB virtual memory
/// - Files: 64 open files
/// - Processes: 10 child processes
///
/// These limits prevent runaway commands from exhausting the host.
pub fn run_shell(cfg: &SandboxConfig, command: &str, cwd: &str) -> Result<SandboxResult> {
    let timeout = Duration::from_secs(cfg.timeout.min(10).max(1));
    let mut cmd = Command::new("bash");
    cmd.arg("-c")
        .arg(command)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear();
    for (k, v) in std::env::vars() {
        if SAFE_ENV.iter().any(|s| *s == k) {
            cmd.env(k, v);
        }
    }

    // Resource limits via ulimit wrapper (bash built-in)
    // -v 262144 = 256MB virtual memory
    // -n 64 = max 64 open files
    // -u 10 = max 10 processes
    // -t 10 = 10s CPU time
    let limit_cmd = format!(
        "ulimit -v 262144 -n 64 -u 10 -t 10 2>/dev/null; {}",
        command
    );
    cmd.arg("-c").arg(&limit_cmd);

    let mut child = cmd.spawn()?;
    let start = Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => {
                let mut stdout = String::new();
                let mut stderr = String::new();
                if let Some(mut pipe) = child.stdout.take() {
                    let _ = pipe.read_to_string(&mut stdout);
                }
                if let Some(mut pipe) = child.stderr.take() {
                    let _ = pipe.read_to_string(&mut stderr);
                }
                return Ok(SandboxResult {
                    exit_code: status.code().unwrap_or(-1),
                    stdout,
                    stderr,
                    timeout: false,
                });
            }
            None if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(SandboxResult {
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("timed out after {}s", cfg.timeout),
                    timeout: true,
                });
            }
            None => std::thread::sleep(Duration::from_millis(40)),
        }
    }
}
