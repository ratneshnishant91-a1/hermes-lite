//! Minimal MCP-style JSON-RPC 2.0 over stdin/stdout.

use crate::tools::ToolRegistry;
use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

pub fn serve(tools: &ToolRegistry) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(err) => {
                writeln!(
                    stdout,
                    "{}",
                    json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":err.to_string()}})
                )?;
                continue;
            }
        };
        let id = req.get("id").cloned().unwrap_or(json!(null));
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let result = match method {
            "initialize" => json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "hermes-lite", "version": "2.0.0"}
            }),
            "tools/list" => json!({"tools": tools.schemas()}),
            "tools/call" => {
                let params = req.get("params").cloned().unwrap_or(json!({}));
                let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                match tools.execute(name, &args) {
                    Ok(v) => json!({"content":[{"type":"text","text": v.to_string()}] }),
                    Err(err) => json!({"error": {"code": -32000, "message": err.to_string()}}),
                }
            }
            _ => json!({"error": {"code": -32601, "message": format!("unknown method {method}")}}),
        };
        let resp = if result.get("error").is_some() {
            json!({"jsonrpc":"2.0","id": id, "error": result.get("error").cloned()})
        } else {
            json!({"jsonrpc":"2.0","id": id, "result": result})
        };
        writeln!(stdout, "{resp}")?;
        stdout.flush()?;
    }
    Ok(())
}
