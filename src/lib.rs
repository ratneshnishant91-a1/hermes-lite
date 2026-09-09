//! Hermes-Lite Rust core.

pub mod agent;
pub mod agents;
pub mod approval;
pub mod config;
pub mod context;
pub mod jobs;
pub mod learner;
pub mod math;
pub mod mcp;
pub mod model;
pub mod network;
pub mod sandbox;
pub mod security;
pub mod skills;
pub mod store;
pub mod tools;
pub mod workspace;

pub use agent::Agent;
pub use agents::SubAgentPool;
pub use config::Config;

#[cfg(test)]
mod tests {
    use crate::security::InputValidator;
    use crate::workspace::Workspace;

    #[test]
    fn rejects_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path()).unwrap();
        assert!(ws.resolve("../etc/passwd").is_err());
        assert!(ws.resolve("ok.txt").is_ok());
    }

    #[test]
    fn validates_user_input() {
        assert!(InputValidator::validate_message("hello").is_ok());
        assert!(InputValidator::validate_message("../secret").is_err());
        assert!(InputValidator::validate_message("").is_err());
    }

    #[test]
    fn blocks_private_hosts() {
        assert!(crate::network::host_blocked("127.0.0.1"));
        assert!(crate::network::host_blocked("10.0.0.5"));
        assert!(crate::network::host_blocked("192.168.0.1"));
        assert!(!crate::network::host_blocked("example.com"));
    }
}
