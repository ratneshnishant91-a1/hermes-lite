use crate::Tools;
use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

pub fn serve(tools: &Tools) -> Result<()> {
    for line in io::stdin().lock().lines() {
        let request: Value = match serde_json::from_str(&line?) { Ok(v)=>v, Err(e)=>{ println!("{}",json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":e.to_string()}})); continue; } };
        let id=request.get("id").cloned().unwrap_or(Value::Null);
        let result=match request.get("method").and_then(Value::as_str).unwrap_or("") {
            "initialize"=>json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"hermes-lite","version":"2.0.0"}}),
            "tools/list"=>json!({"tools":[{"name":"read_file","description":"Read workspace file","inputSchema":{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}},{"name":"write_file","description":"Write workspace file","inputSchema":{"type":"object","properties":{"path":{"type":"string"},"body":{"type":"string"}},"required":["path","body"]}}]}),
            "tools/call"=>{let p=&request["params"];let r=match p["name"].as_str(){Some("read_file")=>tools.read_file(p["arguments"]["path"].as_str().unwrap_or("")),Some("write_file")=>tools.write_file(p["arguments"]["path"].as_str().unwrap_or(""),p["arguments"]["body"].as_str().unwrap_or("")).map(|_|"ok".into()),_=>Err(anyhow::anyhow!("unknown tool"))};match r{Ok(v)=>json!({"content":[{"type":"text","text":v}]}),Err(e)=>json!({"isError":true,"content":[{"type":"text","text":e.to_string()}]})}},
            _=>json!({"error":{"code":-32601,"message":"method not found"}}),
        };
        let response=if result.get("error").is_some(){json!({"jsonrpc":"2.0","id":id,"error":result["error"]})}else{json!({"jsonrpc":"2.0","id":id,"result":result})};
        writeln!(io::stdout(),"{response}")?;io::stdout().flush()?;
    } Ok(())
}
