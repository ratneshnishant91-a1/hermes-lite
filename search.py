from __future__ import annotations

import re
import urllib.request


def web_search(query: str, num_results: int = 5) -> list[dict]:
    encoded_query = urllib.request.quote(query)
    url = f"https://html.duckduckgo.com/html/?q={encoded_query}"
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0 (compatible; Hermes-Lite/1.0)"})
        with urllib.request.urlopen(req, timeout=10) as response:
            html = response.read().decode("utf-8", errors="ignore")
        results = []
        result_pattern = r'<a class="result__a" href="([^"]+)">([^<]+)</a>'
        snippet_pattern = r'<a class="result__snippet" href="[^"]*">([^<]+)</a>'
        titles = re.findall(result_pattern, html)
        snippets = re.findall(snippet_pattern, html)
        for i, (href, title) in enumerate(titles[:num_results]):
            if href.startswith("//"):
                href = "https:" + href
            elif not href.startswith("http"):
                continue
            snippet = snippets[i] if i < len(snippets) else ""
            results.append({"title": title, "url": href, "snippet": snippet})
        return results
    except Exception as e:
        return [{"title": "Search failed", "url": "", "snippet": f"Error: {str(e)}"}]
