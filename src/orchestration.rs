use crate::tools::Tools;
use crate::trajectory::Trajectory;
use anyhow::Result;
use serde_json::Value;

pub struct Sequential {
    agents: Vec<Box<dyn AgentTrait>>,
}

pub trait AgentTrait {
    fn execute(&self, input: &str, tools: &Tools, trajectory: &mut Trajectory) -> Result<String>;
}

impl Sequential {
    pub fn new(agents: Vec<Box<dyn AgentTrait>>) -> Self {
        Self { agents }
    }

    pub fn run(&self, input: &str, tools: &Tools) -> Result<String> {
        let mut trajectory = Trajectory::new();
        let mut current = input.to_string();
        for (i, agent) in self.agents.iter().enumerate() {
            trajectory.log_step(i as u64, "sequential", &current);
            current = agent.execute(&current, tools, &mut trajectory)?;
        }
        trajectory.log_final(&current);
        Ok(current)
    }
}

pub struct Concurrent {
    agents: Vec<Box<dyn AgentTrait>>,
}

impl Concurrent {
    pub fn new(agents: Vec<Box<dyn AgentTrait>>) -> Self {
        Self { agents }
    }

    pub fn run(&self, input: &str, tools: &Tools) -> Result<Vec<String>> {
        let mut results = Vec::new();
        for (i, agent) in self.agents.iter().enumerate() {
            let mut trajectory = Trajectory::new();
            trajectory.log_step(i as u64, "concurrent", input);
            let result = agent.execute(input, tools, &mut trajectory)?;
            trajectory.log_final(&result);
            results.push(result);
        }
        Ok(results)
    }
}

pub struct Handoff {
    agents: Vec<Box<dyn AgentTrait>>,
}

impl Handoff {
    pub fn new(agents: Vec<Box<dyn AgentTrait>>) -> Self {
        Self { agents }
    }

    pub fn run(&self, input: &str, tools: &Tools) -> Result<String> {
        let mut trajectory = Trajectory::new();
        let mut current = input.to_string();
        let mut agent_idx = 0;
        while agent_idx < self.agents.len() {
            trajectory.log_step(agent_idx as u64, "handoff", &current);
            let result = self.agents[agent_idx].execute(&current, tools, &mut trajectory)?;
            if result.starts_with("HANDOFF:") {
                let parts: Vec<&str> = result.split(':').collect();
                if parts.len() > 1 {
                    agent_idx = parts[1].parse().unwrap_or(agent_idx + 1);
                    current = parts.get(2).map(|s| s.to_string()).unwrap_or(current);
                    continue;
                }
            }
            return Ok(result);
        }
        Ok(current)
    }
}
