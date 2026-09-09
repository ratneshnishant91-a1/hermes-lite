use regex::Regex;
use serde_json::Value;
use std::io::{self, Write};

fn hardline() -> Vec<Regex> {
    [
        r"\brm\s+-[^\s]*r\s+/",
        r"\bmkfs\b",
        r"\bdd\s+if=",
        r"\breboot\b",
        r"\bshutdown\b",
    ]
    .into_iter()
    .map(|p| Regex::new(p).unwrap())
    .collect()
}

fn dangerous() -> Vec<Regex> {
    [
        r"\brm\s+-[^\s]*r",
        r"\bsudo\b",
        r"\bchmod\s+(777|666)",
        r"\b(curl|wget)\b.*\|\s*(ba)?sh\b",
    ]
    .into_iter()
    .map(|p| Regex::new(p).unwrap())
    .collect()
}

pub fn requires_approval(tool: &str, args: &Value) -> Option<String> {
    if tool != "shell" {
        return None;
    }
    let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
    for re in hardline() {
        if re.is_match(cmd) {
            return Some(format!("BLOCKED hardline pattern: {cmd}"));
        }
    }
    for re in dangerous() {
        if re.is_match(cmd) {
            return Some(format!("approval required: {cmd}"));
        }
    }
    None
}

pub fn approve(tool: &str, args: &Value) -> bool {
    match requires_approval(tool, args) {
        None => true,
        Some(reason) if reason.starts_with("BLOCKED") => {
            eprintln!("{reason}");
            false
        }
        Some(reason) => {
            print!("APPROVAL REQUIRED\n{reason}\nAllow? [y/N] ");
            let _ = io::stdout().flush();
            let mut line = String::new();
            if io::stdin().read_line(&mut line).is_err() {
                return false;
            }
            line.trim().eq_ignore_ascii_case("y")
        }
    }
}
