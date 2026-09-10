use crate::{math, store::Store, tools::Tools, Config};
use anyhow::Result;

pub struct Agent { store: Store, tools: Tools }
impl Agent {
    pub fn new(config: Config) -> Result<Self> { Ok(Self { store: Store::open(&config.db_path)?, tools: Tools::new(&config)? }) }
    pub fn run(&self, prompt:&str)->Result<String>{
        crate::security::validate_prompt(prompt).map_err(anyhow::Error::msg)?;
        if let Some(answer)=math::evaluate(prompt){return Ok(answer);}
        let lower=prompt.to_lowercase();
        if let Some(value)=lower.strip_prefix("remember "){self.store.remember(value,value)?;return Ok("remembered".into());}
        if let Some(key)=lower.strip_prefix("recall "){return Ok(self.store.recall(key)?.unwrap_or_else(||"not found".into()));}
        if let Some(path)=lower.strip_prefix("read file "){return self.tools.read_file(path.trim());}
        Ok("This lightweight runtime handles local math, memory, and workspace files. Configure an external MCP client or an LLM adapter for open-ended reasoning.".into())
    }
    pub fn tools(&self)->&Tools{&self.tools}
}
