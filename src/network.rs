use crate::config::NetworkConfig;
use anyhow::Result;
use serde_json::{json, Value};
use std::net::ToSocketAddrs;

fn is_private_ip(host: &str) -> bool {
    host == "localhost"
        || host == "127.0.0.1"
        || host == "::1"
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.starts_with("169.254.")
        || host.split('.').next().and_then(|p| p.parse::<u8>().ok()).map(|o| o == 172).unwrap_or(false)
}

fn domain_allowed(host: &str, allow: &[String]) -> bool {
    if allow.is_empty() {
        return true;
    }
    let host = host.to_lowercase();
    allow.iter().any(|d| host == *d || host.ends_with(&format!(".{d}")))
}

pub fn safe_fetch(url: &str, cfg: &NetworkConfig) -> Result<Value> {
    if !cfg.enabled {
        return Ok(json!({"status":"blocked","error":"Network access disabled"}));
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Ok(json!({"status":"blocked","error":"Only HTTP/HTTPS allowed"}));
    }
    let host = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split(['/', ':', '?'])
        .next()
        .unwrap_or("");
    if is_private_ip(host) {
        return Ok(json!({"status":"blocked","error":"Private/internal host blocked"}));
    }
    if !domain_allowed(host, &cfg.allowed_domains) {
        return Ok(json!({"status":"blocked","error":format!("Domain not allowed: {host}")}));
    }
    if let Ok(addrs) = (host, 80).to_socket_addrs() {
        for addr in addrs {
            if addr.ip().is_loopback() || addr.ip().is_private() {
                return Ok(json!({"status":"blocked","error":"Resolved to private IP"}));
            }
        }
    }
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(cfg.timeout.max(1)))
        .build();
    match agent.get(url).call() {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.into_string().unwrap_or_default();
            let clipped = if body.len() > 100_000 {
                format!("{}\n[truncated]", &body[..100_000])
            } else {
                body
            };
            Ok(json!({"url":url,"status":status,"text":clipped}))
        }
        Err(err) => Ok(json!({"url":url,"status":"error","error":err.to_string()})),
    }
}
