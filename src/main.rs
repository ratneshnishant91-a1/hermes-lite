use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use hermes_lite::{Agent, Config, SubAgentPool};
use hermes_lite::security::RateLimiter;
use hermes_lite::store::Store;
use std::io::{self, Write};
use std::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "hermes-lite", version, about = "Self-learning autonomous agent (Rust 2024 core)")]
struct Cli {
    #[arg(long, default_value = "config.yaml")]
    config: String,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Chat,
    Run { prompt: String },
    Stats,
    Memories { limit: Option<usize> },
    Skills,
    Sessions,
    Backup,
    Agents { #[command(subcommand)] action: AgentCommands },
    Goals,
    Constraint { key: String, value: String },
    /// Manage cron jobs
    Cron {
        #[command(subcommand)]
        action: CronCommands,
    },
    Gateway { #[arg(long, default_value = "127.0.0.1:8000", env = "HERMES_BIND")] bind: String },
    Mcp,
}

#[derive(Subcommand)]
enum AgentCommands {
    Spawn { task: String },
    List,
    Get { task_id: String },
}

#[derive(Subcommand)]
enum CronCommands {
    /// Add a cron job: cron add "backup" "every 1h" "shell" "rm -rf /tmp/*"
    Add { name: String, schedule: String, tool: String, args: String },
    /// List all cron jobs
    List,
    /// Remove a cron job
    Remove { job_id: String },
}

fn main() -> Result<()> {
    std::panic::set_hook(Box::new(|info| { tracing::error!("Panic: {}", info); }));
    let env_filter = tracing_subscriber::EnvFilter::from_default_env().add_directive("hermes_lite=info".parse()?);
    if std::env::var("RUST_LOG_JSON").unwrap_or_default() == "1" {
        tracing_subscriber::registry().with(env_filter).with(tracing_subscriber::fmt::layer().json()).init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }

    let cli = Cli::parse();
    let config = Config::load(&cli.config).unwrap_or_else(|_| Config::default());

    match cli.command.unwrap_or(Commands::Chat) {
        Commands::Chat => { let mut agent = Agent::new(config.clone())?; repl(&mut agent) }
        Commands::Run { prompt } => {
            if prompt == "healthcheck" { println!("ok"); return Ok(()); }
            let mut agent = Agent::new(config.clone())?;
            println!("{}", agent.run(&prompt)?);
            Ok(())
        }
        Commands::Stats => {
            let agent = Agent::new(config.clone())?;
            println!("{}", serde_json::to_string_pretty(&agent.learning_stats())?);
            Ok(())
        }
        Commands::Memories { limit } => {
            let store = Store::open(&config.db_path)?;
            for m in store.search_memories("", limit.unwrap_or(20) as i64)? { println!("{m}"); }
            Ok(())
        }
        Commands::Skills => {
            println!("{}", hermes_lite::skills::Skills::new(&config.skills_root).catalog_text());
            Ok(())
        }
        Commands::Sessions => { println!("Sessions stored in {}", config.db_path); Ok(()) }
        Commands::Backup => {
            let store = Store::open(&config.db_path)?;
            store.backup(&mut io::stdout())?;
            Ok(())
        }
        Commands::Agents { action } => {
            let pool = SubAgentPool::new(config.clone());
            match action {
                AgentCommands::Spawn { task } => println!("Spawned: {}", pool.spawn(task)),
                AgentCommands::List => { for t in pool.status() { println!("{}: {} - {}", t.id, t.status, t.description); } }
                AgentCommands::Get { task_id } => match pool.get_result(&task_id) { Some(t) => println!("{:?}", t), None => println!("Not found") }
            }
            Ok(())
        }
        Commands::Goals => {
            let agent = Agent::new(config.clone())?;
            for g in agent.list_goals()? { println!("[{}] {} (step {}/{})", g.status, g.description, g.current_step, serde_json::from_str::<Vec<String>>(&g.plan).map(|v| v.len()).unwrap_or(0)); }
            Ok(())
        }
        Commands::Constraint { key, value } => {
            let agent = Agent::new(config.clone())?;
            agent.set_constraint(&key, &value)?;
            println!("Constraint set: {key}={value}");
            Ok(())
        }
        Commands::Cron { action } => {
            let store = Store::open(&config.db_path)?;
            match action {
                CronCommands::Add { name, schedule, tool, args } => {
                    let args_json = serde_json::json!({"command": args});
                    let job = store.create_cron_job(&uuid::Uuid::new_v4().to_string(), &name, &schedule, &tool, &args_json.to_string())?;
                    println!("Cron job added: {name} ({schedule})");
                }
                CronCommands::List => {
                    for job in store.list_cron_jobs()? {
                        println!("[{}] {} - {} (next: {}, runs: {})", job.id, job.name, job.schedule, job.next_run, job.run_count);
                    }
                }
                CronCommands::Remove { job_id } => {
                    store.delete_cron_job(&job_id)?;
                    println!("Cron job removed: {job_id}");
                }
            }
            Ok(())
        }
        Commands::Gateway { bind } => { let mut agent = Agent::new(config.clone())?; gateway(&mut agent, &bind) }
        Commands::Mcp => { let agent = Agent::new(config.clone())?; hermes_lite::mcp::serve(agent.tools()) }
    }
}

fn repl(agent: &mut Agent) -> Result<()> {
    println!("Hermes-Lite v2.0 — Agentic mode");
    println!("session={}  cmds: exit | stats | goals | cron | constraint\n", agent.session_id());
    let stdin = io::stdin();
    loop {
        print!("You: "); io::stdout().flush()?;
        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 { break; }
        let line = line.trim();
        if line.is_empty() || matches!(line, "exit"|"quit"|"q") { break; }
        if line == "stats" { println!("{}", serde_json::to_string_pretty(&agent.learning_stats())?); continue; }
        if line == "goals" { for g in agent.list_goals()? { println!("[{}] {}", g.status, g.description); } continue; }
        match agent.run(line) { Ok(a) => println!("\nAgent: {a}\n"), Err(e) => eprintln!("Error: {e:#}") }
    }
    Ok(())
}

fn gateway(agent: &mut Agent, bind: &str) -> Result<()> {
    let listener = TcpListener::bind(bind).with_context(|| format!("bind {bind}"))?;
    let rate_limiter = RateLimiter::new(60, 60);
    tracing::info!("gateway on http://{bind}");
    for stream in listener.incoming() {
        let mut stream = match stream { Ok(s) => s, Err(_) => continue };
        let peer = stream.peer_addr().map(|a| a.ip().to_string()).unwrap_or_else(|_| "unknown".into());
        if !rate_limiter.allow(&peer) { let _ = stream.write_all(b"HTTP/1.1 429\r\nContent-Length: 0\r\n\r\n"); continue; }
        let mut buf = [0u8; 8192];
        let n = match std::io::Read::read(&mut stream, &mut buf) { Ok(n) => n, Err(_) => continue };
        let req = String::from_utf8_lossy(&buf[..n]);
        let (status, body) = if req.starts_with("GET /health") { ("200 OK", r#"{"status":"ok"}"#.into()) }
            else if req.starts_with("POST /chat") {
                let msg = req.split("\r\n\r\n").nth(1).and_then(|b| serde_json::from_str::<serde_json::Value>(b).ok()).and_then(|v| v.get("message").and_then(|m| m.as_str()).map(str::to_owned)).unwrap_or_default();
                match agent.run(&msg) { Ok(a) => ("200 OK", format!(r#"{{"response":"{}"}}"#, a)), Err(e) => ("500 Internal Server Error", format!(r#"{{"error":"{}"}}"#, e)) }
            } else { ("404 Not Found", r#"{"error":"not found"}"#.into()) };
        let resp = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{body}", body.len());
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    }
    Ok(())
}
