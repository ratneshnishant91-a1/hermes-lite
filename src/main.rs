use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use hermes_lite::{Agent, Config, SubAgentPool};
use hermes_lite::security::RateLimiter;
use hermes_lite::store::Store;
use secrecy::Secret;
use std::io::{self, Write};
use std::net::TcpListener;
use std::sync::Arc;
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
    Agents {
        #[command(subcommand)]
        action: AgentCommands,
    },
    /// List active goals
    Goals,
    Gateway {
        #[arg(long, default_value = "127.0.0.1:8000", env = "HERMES_BIND")]
        bind: String,
    },
    Mcp,
}

#[derive(Subcommand)]
enum AgentCommands {
    Spawn { task: String },
    List,
    Get { task_id: String },
}

fn main() -> Result<()> {
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("Panic: {}", info);
    }));

    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("hermes_lite=info".parse()?);
    if std::env::var("RUST_LOG_JSON").unwrap_or_default() == "1" {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }

    let cli = Cli::parse();
    let config = Config::load(&cli.config).unwrap_or_else(|_| Config::default());

    match cli.command.unwrap_or(Commands::Chat) {
        Commands::Chat => {
            let mut agent = Agent::new(config.clone())?;
            repl(&mut agent)
        }
        Commands::Run { prompt } => {
            if prompt == "healthcheck" {
                println!("ok");
                return Ok(());
            }
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
            let limit = limit.unwrap_or(20);
            let memories = store.search_memories("", limit as i64)?;
            for m in memories {
                println!("{m}");
            }
            Ok(())
        }
        Commands::Skills => {
            let skills = hermes_lite::skills::Skills::new(&config.skills_root);
            println!("{}", skills.catalog_text());
            Ok(())
        }
        Commands::Sessions => {
            let store = Store::open(&config.db_path)?;
            println!("Sessions stored in {}", config.db_path);
            Ok(())
        }
        Commands::Backup => {
            let store = Store::open(&config.db_path)?;
            store.backup(&mut io::stdout())?;
            Ok(())
        }
        Commands::Agents { action } => {
            let pool = SubAgentPool::new(config.clone());
            match action {
                AgentCommands::Spawn { task } => {
                    let task_id = pool.spawn(task);
                    println!("Spawned sub-agent: {task_id}");
                }
                AgentCommands::List => {
                    let tasks = pool.status();
                    for task in tasks {
                        println!("{}: {} - {}", task.id, task.status, task.description);
                    }
                }
                AgentCommands::Get { task_id } => {
                    match pool.get_result(&task_id) {
                        Some(task) => println!("{:?}", task),
                        None => println!("Task not found: {task_id}"),
                    }
                }
            }
            Ok(())
        }
        Commands::Goals => {
            let mut agent = Agent::new(config.clone())?;
            let goals = agent.list_goals();
            if goals.is_empty() {
                println!("No active goals.");
            } else {
                for g in goals {
                    println!("[{}] {} (step {}/{})", g.status, g.description, g.current_step, g.plan.len());
                }
            }
            Ok(())
        }
        Commands::Gateway { bind } => {
            let mut agent = Agent::new(config.clone())?;
            gateway(&mut agent, &bind)
        }
        Commands::Mcp => {
            let agent = Agent::new(config.clone())?;
            hermes_lite::mcp::serve(agent.tools())
        }
    }
}

fn repl(agent: &mut Agent) -> Result<()> {
    println!("Hermes-Lite v2.0 (Rust 2024 / rustc 1.98) — Agentic mode");
    println!("session={}  commands: exit | stats | goals | memories | skills\n", agent.session_id());
    let stdin = io::stdin();
    loop {
        print!("You: ");
        io::stdout().flush()?;
        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if matches!(line, "exit" | "quit" | "q") {
            break;
        }
        if line == "stats" {
            println!("{}", serde_json::to_string_pretty(&agent.learning_stats())?);
            continue;
        }
        if line == "goals" {
            for g in agent.list_goals() {
                println!("[{}] {} (step {}/{})", g.status, g.description, g.current_step, g.plan.len());
            }
            continue;
        }
        match agent.run(line) {
            Ok(answer) => println!("\nAgent: {answer}\n"),
            Err(err) => eprintln!("Error: {err:#}"),
        }
    }
    Ok(())
}

fn gateway(agent: &mut Agent, bind: &str) -> Result<()> {
    let listener = TcpListener::bind(bind).with_context(|| format!("bind {bind}"))?;
    let rate_limiter = RateLimiter::new(60, 60);
    tracing::info!("gateway on http://{bind}");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        let peer = stream.peer_addr().map(|a| a.ip().to_string()).unwrap_or_else(|_| "unknown".into());
        if !rate_limiter.allow(&peer) {
            let resp = "HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes());
            continue;
        }

        let mut buf = [0u8; 8192];
        let n = match std::io::Read::read(&mut stream, &mut buf) {
            Ok(n) => n,
            Err(_) => continue,
        };
        let req = String::from_utf8_lossy(&buf[..n]);
        let (status, body) = if req.starts_with("GET /health") {
            ("200 OK", serde_json::json!({"status":"ok"}).to_string())
        } else if req.starts_with("POST /chat") {
            let msg = req
                .split("\r\n\r\n")
                .nth(1)
                .and_then(|b| serde_json::from_str::<serde_json::Value>(b).ok())
                .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(str::to_owned))
                .unwrap_or_default();
            match agent.run(&msg) {
                Ok(answer) => ("200 OK", serde_json::json!({"response":answer}).to_string()),
                Err(err) => ("500 Internal Server Error", serde_json::json!({"error":err.to_string()}).to_string()),
            }
        } else {
            ("404 Not Found", serde_json::json!({"error":"not found"}).to_string())
        };
        let resp = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
    }
    Ok(())
}
