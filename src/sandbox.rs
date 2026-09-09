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

/// Run `command` via `bash -c` with a filtered environment and a hard timeout.
///
/// After `Child::try_wait` reaps the process we only read the pipes — we do **not**
/// call `wait_with_output` again (that would wait on an already-reaped child).
pub fn run_shell(cfg: &SandboxConfig, command: &str, cwd: &str) -> Result<SandboxResult> {
    let timeout = Duration::from_secs(cfg.timeout.max(1));
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
