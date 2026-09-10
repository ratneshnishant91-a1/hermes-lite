use std::io::{Read,Write};
use std::process::{Command,Stdio};
use serde_json::{json,Value};

fn mcp_roundtrip() -> std::io::Result<()> {
    let mut child = Command::new("cargo").arg("run").arg("--quiet").arg("--").arg("mcp")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn()?;
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut buf = String::new();
    fn req(id:i32,method:&str,params:Value)->String{json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}).to_string()}
    fn read_resp<R:Read>(r:&mut R)->Value{let mut s=String::new();r.read_to_string(&mut s).unwrap();serde_json::from_str(&s).unwrap()}
    stdin.write_all(req(1,"initialize",json!({})).as_bytes())?;stdin.flush()?;let _resp:Value=read_resp(&mut stdout);
    stdin.write_all(req(2,"tools/list",json!({})).as_bytes())?;stdin.flush()?;let tools:Value=read_resp(&mut stdout);
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(),2);
    stdin.write_all(req(3,"tools/call",json!({"name":"write_file","arguments":{"path":"mcp_test.txt","body":"hello"}})).as_bytes())?;stdin.flush()?;let _resp:Value=read_resp(&mut stdout);
    let _ = child.kill(); Ok(())
}

#[test] fn mcp_stdio_works(){let _=mcp_roundtrip();}
