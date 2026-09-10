use anyhow::Result;
use clap::{Parser,Subcommand};
use hermes_lite::{agent::Agent,cron,mcp,store::Store,Config};

#[derive(Parser)] struct Cli { #[arg(long,default_value="config.yaml")] config:String, #[command(subcommand)] command:Command }
#[derive(Subcommand)] enum Command { Run{prompt:String}, Mcp, Cron{#[command(subcommand)] command:Cron}, Tick }
#[derive(Subcommand)] enum Cron { Add{name:String,schedule:String,command:String} }
fn main()->Result<()> { let cli=Cli::parse(); let cfg=Config::load(&cli.config).unwrap_or_default(); match cli.command { Command::Run{prompt}=>println!("{}",Agent::new(cfg)?.run(&prompt)?), Command::Mcp=>{let agent=Agent::new(cfg)?;mcp::serve(agent.tools())?},Command::Cron{command:Cron::Add{name,schedule,command}}=>{let store=Store::open(&cfg.db_path)?;let job=cron::add(&store,&name,&schedule,&command)?;println!("{}",job.id)},Command::Tick=>{let store=Store::open(&cfg.db_path)?;let agent=Agent::new(cfg)?;println!("{}",cron::tick(&store,agent.tools())?)} } Ok(()) }
