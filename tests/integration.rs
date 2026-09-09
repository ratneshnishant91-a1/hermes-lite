//! End-to-end integration tests

use hermes_lite::{Config, Agent};
use std::env;

#[test]
#[ignore] // Requires OPENAI_API_KEY
fn test_agent_basic_chat() {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let config = Config::default();
    let mut agent = Agent::new(config).expect("Failed to create agent");
    
    let response = agent.run("Hello, what is your name?").expect("Failed to run agent");
    assert!(!response.is_empty());
    assert!(response.len() > 10);
}

#[test]
#[ignore] // Requires OPENAI_API_KEY
fn test_agent_remembers_preferences() {
    let config = Config::default();
    let mut agent = Agent::new(config).expect("Failed to create agent");
    
    // Teach preference
    let _ = agent.run("Remember that I prefer Rust over Python");
    
    // Ask about preference
    let response = agent.run("What programming language do I prefer?").expect("Failed");
    assert!(response.to_lowercase().contains("rust"));
}

#[test]
fn test_config_load() {
    let config = Config::default();
    assert_eq!(config.workspace_root, "workspace");
    assert_eq!(config.db_path, "hermes.db");
}

#[test]
fn test_workspace_sandbox() {
    use hermes_lite::workspace::Workspace;
    use tempfile::tempdir;
    
    let dir = tempdir().unwrap();
    let ws = Workspace::new(dir.path()).unwrap();
    
    // Should reject path traversal
    assert!(ws.resolve("../etc/passwd").is_err());
    assert!(ws.resolve("../../secret").is_err());
    
    // Should accept valid paths
    assert!(ws.resolve("file.txt").is_ok());
    assert!(ws.resolve("subdir/file.txt").is_ok());
}

#[test]
fn test_security_input_validation() {
    use hermes_lite::security::InputValidator;
    
    // Valid inputs
    assert!(InputValidator::validate_message("Hello").is_ok());
    assert!(InputValidator::validate_message("What is 2+2?").is_ok());
    
    // Invalid inputs
    assert!(InputValidator::validate_message("").is_err());
    assert!(InputValidator::validate_message("   ").is_err());
    assert!(InputValidator::validate_message("../secret").is_err());
}

#[test]
fn test_network_ssrf_protection() {
    use hermes_lite::network::host_blocked;
    
    // Should block private/internal hosts
    assert!(host_blocked("127.0.0.1"));
    assert!(host_blocked("localhost"));
    assert!(host_blocked("10.0.0.1"));
    assert!(host_blocked("192.168.1.1"));
    assert!(host_blocked("169.254.169.254")); // AWS metadata
    
    // Should allow public hosts
    assert!(!host_blocked("example.com"));
    assert!(!host_blocked("google.com"));
    assert!(!host_blocked("api.openai.com"));
}
