use crate::tools::Tools;
use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

pub fn process_message(raw: &str, tools: &Tools) -> Option<String> {
    let request: Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(e) => {
            return Some(json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":e.to_string()}}).to_string());
        }
    };
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    let result = match method {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "hermes-lite", "version": "2.0.0"}
        }),
        "tools/list" => json!({
            "tools": [
                {
                    "name": "read_file",
                    "description": "Read workspace file",
                    "inputSchema": {
                        "type": "object",
                        "properties": {"path": {"type": "string"}},
                        "required": ["path"]
                    }
                },
                {
                    "name": "write_file",
                    "description": "Write workspace file",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": {"type": "string"},
                            "body": {"type": "string"}
                        },
                        "required": ["path", "body"]
                    }
                }
            ]
        }),
        "tools/call" => {
            let p = &request["params"];
            let name = p.get("name").and_then(Value::as_str).unwrap_or("");
            let call_res = match name {
                "read_file" => {
                    let path = p["arguments"]["path"].as_str().unwrap_or("");
                    tools.read_file(path)
                }
                "write_file" => {
                    let path = p["arguments"]["path"].as_str().unwrap_or("");
                    let body = p["arguments"]["body"].as_str().unwrap_or("");
                    tools.write_file(path, body).map(|_| "ok".to_string())
                }
                _ => Err(anyhow::anyhow!("unknown tool: {name}")),
            };
            match call_res {
                Ok(v) => json!({"content": [{"type": "text", "text": v}]}),
                Err(e) => json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]}),
            }
        }
        _ => json!({"error": {"code": -32601, "message": format!("method not found: {method}")}}),
    };

    let response = if result.get("error").is_some() {
        json!({"jsonrpc": "2.0", "id": id, "error": result["error"]})
    } else {
        json!({"jsonrpc": "2.0", "id": id, "result": result})
    };
    Some(response.to_string())
}

pub fn serve(tools: &Tools) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(resp) = process_message(&line, tools) {
            writeln!(stdout, "{resp}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}
