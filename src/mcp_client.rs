//! MCP client for connecting to external tools via Model Context Protocol.

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

pub struct MCPClient {
    name: String,
    process: std::process::Child,
    reader: BufReader<std::process::ChildStdout>,
    request_id: u64,
}

impl MCPClient {
    /// Connect to MCP server (stdio transport)
    pub fn connect(name: &str, command: &str, args: &[&str]) -> Result<Self> {
        let mut process = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to spawn MCP server: {} {:?}", command, args))?;

        let reader = BufReader::new(process.stdout.take().unwrap());
        let mut client = Self {
            name: name.to_string(),
            process,
            reader,
            request_id: 0,
        };

        // Initialize connection
        client.initialize()?;

        Ok(client)
    }

    /// Initialize MCP connection
    fn initialize(&mut self) -> Result<()> {
        self.request_id += 1;
        let init_request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "hermes-lite",
                    "version": "2.0.0"
                }
            }
        });

        self.send_request(&init_request)?;
        let response = self.read_response()?;

        if response.get("error").is_some() {
            anyhow::bail!("MCP initialization failed: {}", response["error"]);
        }

        Ok(())
    }

    /// List available tools
    pub fn list_tools(&mut self) -> Result<Vec<Value>> {
        self.request_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": "tools/list",
            "params": {}
        });

        self.send_request(&request)?;
        let response = self.read_response()?;

        let tools = response
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(tools)
    }

    /// Call a tool
    pub fn call_tool(&mut self, name: &str, arguments: &Value) -> Result<Value> {
        self.request_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });

        self.send_request(&request)?;
        let response = self.read_response()?;

        if let Some(error) = response.get("error") {
            anyhow::bail!("Tool call failed: {}", error);
        }

        let result = response
            .get("result")
            .and_then(|r| r.get("content"))
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("(no result)");

        Ok(json!(result))
    }

    fn send_request(&mut self, request: &Value) -> Result<()> {
        let stdin = self.process.stdin.as_mut().unwrap();
        writeln!(stdin, "{}", request)?;
        stdin.flush()?;
        Ok(())
    }

    fn read_response(&mut self) -> Result<Value> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        let response: Value = serde_json::from_str(&line)?;
        Ok(response)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Registry of MCP clients
pub struct MCPRegistry {
    clients: Vec<MCPClient>,
}

impl MCPRegistry {
    pub fn new() -> Self {
        Self { clients: Vec::new() }
    }

    /// Add MCP server (e.g., Google Drive, Dropbox, GitHub)
    pub fn add(&mut self, name: &str, command: &str, args: &[&str]) -> Result<()> {
        let client = MCPClient::connect(name, command, args)?;
        self.clients.push(client);
        Ok(())
    }

    /// List all tools from all connected MCP servers
    pub fn list_all_tools(&mut self) -> Result<Vec<(String, Value)>> {
        let mut all_tools = Vec::new();
        for client in &mut self.clients {
            let tools = client.list_tools()?;
            for tool in tools {
                all_tools.push((client.name().to_string(), tool));
            }
        }
        Ok(all_tools)
    }

    /// Call tool by name (searches all connected MCP servers)
    pub fn call_tool(&mut self, name: &str, arguments: &Value) -> Result<Value> {
        for client in &mut self.clients {
            let tools = client.list_tools()?;
            if tools.iter().any(|t| t.get("function").and_then(|f| f.get("name")).as_ref().map(|n| n.as_str() == Some(name)).unwrap_or(false)) {
                return client.call_tool(name, arguments);
            }
        }
        anyhow::bail!("Tool not found: {}", name)
    }
}

impl Default for MCPRegistry {
    fn default() -> Self {
        Self::new()
    }
}
