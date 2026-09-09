from __future__ import annotations

import re
import socket
import urllib.request
import urllib.error
from typing import Optional
from dataclasses import dataclass
from ipaddress import ip_address, IPv4Address, IPv6Address


# Private/internal IP ranges (RFC 1918, RFC 4193)
PRIVATE_IP_PATTERNS = [
    re.compile(r'^10\.'),  # 10.0.0.0/8
    re.compile(r'^172\.(1[6-9]|2[0-9]|3[0-1])\.'),  # 172.16.0.0/12
    re.compile(r'^192\.168\.'),  # 192.168.0.0/16
    re.compile(r'^127\.'),  # localhost
    re.compile(r'^0\.0\.0\.'),  # 0.0.0.0
    re.compile(r'^::1$'),  # IPv6 localhost
    re.compile(r'^fc00:'),  # IPv6 unique local
    re.compile(r'^fd[0-9a-f]{2}:'),  # IPv6 unique local
]

# Blocked ports
BLOCKED_PORTS = {22, 23, 25, 53, 135, 139, 445, 3306, 3389, 5432, 6379, 27017}


@dataclass
class NetworkConfig:
    enabled: bool = False
    allowed_domains: list[str] = None
    blocked_ports: set[int] = None
    max_response_size: int = 10 * 1024 * 1024  # 10MB
    timeout: int = 30

    def __post_init__(self):
        if self.allowed_domains is None:
            self.allowed_domains = []
        if self.blocked_ports is None:
            self.blocked_ports = BLOCKED_PORTS


def is_private_ip(ip: str) -> bool:
    """Check if IP is private/internal (SSRF protection)."""
    for pattern in PRIVATE_IP_PATTERNS:
        if pattern.match(ip):
            return True
    return False


def resolve_hostname(hostname: str) -> list[str]:
    """Resolve hostname to IPs and check for private IPs."""
    try:
        addresses = socket.getaddrinfo(hostname, None)
        ips = []
        for addr in addresses:
            ip = addr[4][0]
            if is_private_ip(ip):
                raise ValueError(f"Private/internal IP blocked: {ip}")
            ips.append(ip)
        return ips
    except socket.gaierror:
        raise ValueError(f"Cannot resolve hostname: {hostname}")


def is_domain_allowed(domain: str, allowed_domains: list[str]) -> bool:
    """Check if domain is in allowlist."""
    if not allowed_domains:
        return True  # No allowlist = all domains allowed

    domain = domain.lower()
    for allowed in allowed_domains:
        allowed = allowed.lower()
        if domain == allowed or domain.endswith(f".{allowed}"):
            return True
    return False


def safe_fetch_url(url: str, config: NetworkConfig) -> dict:
    """
    Safely fetch URL with SSRF protection.
    """
    if not config.enabled:
        return {"error": "Network access disabled", "status": "blocked"}

    try:
        # Parse URL
        from urllib.parse import urlparse
        parsed = urlparse(url)

        if parsed.scheme not in ["http", "https"]:
            return {"error": "Only HTTP/HTTPS allowed", "status": "blocked"}

        # Check domain allowlist
        if not is_domain_allowed(parsed.hostname, config.allowed_domains):
            return {
                "error": f"Domain not in allowlist: {parsed.hostname}",
                "status": "blocked",
            }

        # Resolve and check for private IPs (SSRF protection)
        try:
            resolve_hostname(parsed.hostname)
        except ValueError as e:
            return {"error": str(e), "status": "blocked"}

        # Fetch with limits
        req = urllib.request.Request(
            url,
            headers={"User-Agent": "Mozilla/5.0 (Hermes-Lite/1.5; Security-Hardened)"},
        )

        with urllib.request.urlopen(req, timeout=config.timeout) as response:
            # Check content length
            content_length = response.headers.get("Content-Length")
            if content_length and int(content_length) > config.max_response_size:
                return {
                    "error": f"Response too large: {content_length} bytes",
                    "status": "blocked",
                }

            # Read response
            content = response.read(config.max_response_size)

            return {
                "url": url,
                "status": "success",
                "content_type": response.headers.get("Content-Type", "unknown"),
                "content_length": len(content),
                "content": content.decode("utf-8", errors="ignore")[:100000],  # Limit to 100KB
            }

    except urllib.error.HTTPError as e:
        return {"error": f"HTTP {e.code}: {e.reason}", "status": "error"}
    except urllib.error.URLError as e:
        return {"error": f"Network error: {e.reason}", "status": "error"}
    except Exception as e:
        return {"error": str(e), "status": "error"}


def safe_web_search(query: str, config: NetworkConfig) -> list[dict]:
    """
    Safe web search using DuckDuckGo.
    """
    if not config.enabled:
        return [{"error": "Network access disabled", "status": "blocked"}]

    # DuckDuckGo HTML search
    from urllib.parse import quote
    encoded = quote(query)
    url = f"https://html.duckduckgo.com/html/?q={encoded}"

    return [safe_fetch_url(url, config)]
