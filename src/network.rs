use crate::config::NetworkConfig;
use anyhow::Result;
use serde_json::{json, Value};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};

/// Hosts we never fetch, even if DNS later maps them to a public A record.
pub fn host_blocked(host: &str) -> bool {
    let host = host.trim().trim_matches('[').trim_matches(']').to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        return true;
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return ip_blocked(ip);
    }
    false
}

fn ip_blocked(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => v4_blocked(v),
        IpAddr::V6(v) => v6_blocked(v),
    }
}

fn v4_blocked(v: Ipv4Addr) -> bool {
    // `Ipv4Addr::is_private` has been stable for years; do not call `IpAddr::is_private`
    // (that helper is newer and not what we want for mixed v4/v6 anyway).
    v.is_loopback() || v.is_private() || v.is_link_local() || v.is_unspecified() || v.is_broadcast()
}

fn v6_blocked(v: Ipv6Addr) -> bool {
    v.is_loopback()
        || v.is_unspecified()
        || v.is_multicast()
        || v.to_ipv4_mapped().is_some_and(v4_blocked)
        || (v.octets()[0] & 0xfe) == 0xfc // unique local fc00::/7
}

fn domain_allowed(host: &str, allow: &[String]) -> bool {
    if allow.is_empty() {
        return true;
    }
    let host = host.to_ascii_lowercase();
    allow.iter().any(|d| {
        let d = d.to_ascii_lowercase();
        host == d || host.ends_with(&format!(".{d}"))
    })
}

pub fn safe_fetch(url: &str, cfg: &NetworkConfig) -> Result<Value> {
    if !cfg.enabled {
        return Ok(json!({"status":"blocked","error":"Network access disabled"}));
    }
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Ok(json!({"status":"blocked","error":"Only HTTP/HTTPS allowed"}));
    }
    let host = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split(['/', ':', '?', '#'])
        .next()
        .unwrap_or("");
    if host_blocked(host) {
        return Ok(json!({"status":"blocked","error":"Private/internal host blocked"}));
    }
    if !domain_allowed(host, &cfg.allowed_domains) {
        return Ok(json!({"status":"blocked","error":format!("Domain not allowed: {host}")}));
    }
    if let Ok(addrs) = (host, 443).to_socket_addrs() {
        for addr in addrs {
            if ip_blocked(addr.ip()) {
                return Ok(json!({"status":"blocked","error":"Resolved to a blocked IP"}));
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
            Ok(json!({"url": url, "status": status, "text": clipped}))
        }
        Err(err) => Ok(json!({"url": url, "status": "error", "error": err.to_string()})),
    }
}
