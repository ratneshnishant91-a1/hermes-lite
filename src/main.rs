use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use hermes_lite::{Agent, Config};
use hermes_lite::security::RateLimiter;
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
    Gateway {
        #[arg(long, default_value = "127.0.0.1:8000", env = "HERMES_BIND")]
        bind: String,
    },
    Mcp,
}

fn main() -> Result<()> {
    // Install panic hook to log without leaking secrets
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("Panic: {}", info);
    }));

    // Structured logging (JSON if RUST_LOG_JSON=1)
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
    let mut agent = Agent::new(config)?;

    match cli.command.unwrap_or(Commands::Chat) {
        Commands::Chat => repl(&mut agent),
        Commands::Run { prompt } => {
            if prompt == "healthcheck" {
                println!("ok");
                return Ok(());
            }
            println!("{}", agent.run(&prompt)?);
            Ok(())
        }
        Commands::Stats => {
            println!("{}", serde_json::to_string_pretty(&agent.learning_stats())?);
            Ok(())
        }
        Commands::Gateway { bind } => gateway(&mut agent, &bind),
        Commands::Mcp => hermes_lite::mcp::serve(agent.tools()),
    }
}

fn repl(agent: &mut Agent) -> Result<()> {
    println!("Hermes-Lite v2.0 (Rust 2024 / rustc 1.98)");
    println!("session={}  commands: exit | stats\n", agent.session_id());
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
        match agent.run(line) {
            Ok(answer) => println!("\nAgent: {answer}\n"),
            Err(err) => eprintln!("Error: {err:#}"),
        }
    }
    Ok(())
}

fn gateway(agent: &mut Agent, bind: &str) -> Result<()> {
    let listener = TcpListener::bind(bind).with_context(|| format!("bind {bind}"))?;
    let rate_limiter = RateLimiter::new(60, 60); // 60 req/min per IP
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
