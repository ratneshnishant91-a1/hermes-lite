from __future__ import annotations

import re
from urllib.request import urlopen, Request
from urllib.error import URLError, HTTPError
from html.parser import HTMLParser


class HTMLExtractor(HTMLParser):
    def __init__(self):
        super().__init__()
        self.text_parts = []
        self.skip_tags = {"script", "style", "noscript", "meta", "link"}
        self.in_skip = False

    def handle_starttag(self, tag, attrs):
        if tag in self.skip_tags:
            self.in_skip = True

    def handle_endtag(self, tag):
        if tag in self.skip_tags:
            self.in_skip = False

    def handle_data(self, data):
        if not self.in_skip:
            text = data.strip()
            if text:
                self.text_parts.append(text)

    def get_text(self) -> str:
        return "\n".join(self.text_parts)


def fetch_url(url: str, max_length: int = 8000) -> dict:
    if not url.startswith(("http://", "https://")):
        return {"url": url, "status": "error", "error": "URL must start with http:// or https://"}
    try:
        req = Request(url, headers={"User-Agent": "Mozilla/5.0 (compatible; Hermes-Lite/1.0)"})
        with urlopen(req, timeout=10) as response:
            html = response.read().decode("utf-8", errors="ignore")
        title_match = re.search(r"<title[^>]*>([^<]+)</title>", html, re.IGNORECASE)
        title = title_match.group(1).strip() if title_match else "Untitled"
        parser = HTMLExtractor()
        parser.feed(html)
        text = parser.get_text()
        if len(text) > max_length:
            text = text[:max_length] + "\n\n[Content truncated]"
        return {"url": url, "title": title, "text": text, "status": "success", "char_count": len(text)}
    except HTTPError as e:
        return {"url": url, "status": "error", "error": f"HTTP {e.code}: {e.reason}"}
    except URLError as e:
        return {"url": url, "status": "error", "error": f"Network error: {e.reason}"}
    except Exception as e:
        return {"url": url, "status": "error", "error": str(e)}
