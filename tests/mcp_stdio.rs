use hermes_lite::{mcp, tools::Tools, Config};
use serde_json::{json, Value};
use tempfile::TempDir;

#[test]
fn mcp_protocol_flow() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let cfg = Config {
        workspace_root: dir.path().display().to_string(),
        ..Default::default()
    };
    let tools = Tools::new(&cfg)?;
    let init_req = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize"}).to_string();
    let init_res: Value = serde_json::from_str(&mcp::process_message(&init_req, &tools).unwrap())?;
    assert_eq!(init_res["id"], 1);
    assert_eq!(init_res["result"]["serverInfo"]["name"], "hermes-lite");
    let list_req = json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}).to_string();
    let list_res: Value = serde_json::from_str(&mcp::process_message(&list_req, &tools).unwrap())?;
    let tools_list = list_res["result"]["tools"].as_array().unwrap();
    assert_eq!(tools_list.len(), 2);
    let write_req = json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"write_file","arguments":{"path":"hello.txt","body":"world"}}}).to_string();
    let write_res: Value = serde_json::from_str(&mcp::process_message(&write_req, &tools).unwrap())?;
    assert_eq!(write_res["result"]["content"][0]["text"], "ok");
    let read_req = json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"read_file","arguments":{"path":"hello.txt"}}}).to_string();
    let read_res: Value = serde_json::from_str(&mcp::process_message(&read_req, &tools).unwrap())?;
    assert_eq!(read_res["result"]["content"][0]["text"], "world");
    Ok(())
}
