use anyhow::Result;
use clap::{Parser, Subcommand};
use hermes_lite::{agent::Agent, cron, mcp, store::Store, Config};

#[derive(Parser)]
#[command(name = "hermes-lite", version = "2.0.0", about = "Small, secure, durable agent runtime")]
struct Cli {
    #[arg(long, default_value = "config.yaml")]
    config: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Run { prompt: String },
    Mcp,
    Cron { #[command(subcommand)] command: CronCommand },
    Tick,
    Status,
}

#[derive(Subcommand)]
enum CronCommand {
    Add { name: String, schedule: String, command: String },
    List,
    Remove { id: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = Config::load(&cli.config).unwrap_or_default();
    match cli.command {
        Command::Run { prompt } => {
            println!("{}", Agent::new(cfg)?.run(&prompt)?);
        }
        Command::Mcp => {
            let agent = Agent::new(cfg)?;
            mcp::serve(agent.tools())?;
        }
        Command::Cron { command } => {
            let store = Store::open(&cfg.db_path)?;
            match command {
                CronCommand::Add { name, schedule, command } => {
                    let job = cron::add(&store, &name, &schedule, &command)?;
                    println!("Created cron job {} ({})", job.id, job.name);
                }
                CronCommand::List => {
                    let jobs = store.list_cron()?;
                    if jobs.is_empty() {
                        println!("No cron jobs configured.");
                    } else {
                        for j in jobs {
                            println!("- id: {}\n  name: {}\n  every: {}s\n  cmd: {}\n  next_run: {}\n  enabled: {}", j.id, j.name, j.every_secs, j.command, j.next_run, j.enabled);
                        }
                    }
                }
                CronCommand::Remove { id } => {
                    if store.delete_cron(&id)? {
                        println!("Removed cron job {id}");
                    } else {
                        println!("Cron job {id} not found");
                    }
                }
            }
        }
        Command::Tick => {
            let store = Store::open(&cfg.db_path)?;
            let agent = Agent::new(cfg)?;
            let count = cron::tick(&store, agent.tools())?;
            println!("Ticked: {count} jobs executed");
        }
        Command::Status => {
            println!("Hermes-Lite v2.0.0 is operational.");
            println!("Workspace: {}", cfg.workspace_root);
            println!("Database:  {}", cfg.db_path);
        }
    }
    Ok(())
}
