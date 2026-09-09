from __future__ import annotations

import json
import time
from typing import Generator
from dataclasses import dataclass


@dataclass
class StreamStats:
    total_tokens: int = 0
    time_to_first_token_ms: float = 0.0
    tokens_per_second: float = 0.0
    provider: str = ""
    model: str = ""


def stream_sse(response_generator: Generator[str, None, None], include_stats: bool = False, stats: StreamStats | None = None) -> Generator[str, None, None]:
    yield "data: [START]\n\n"
    start_time = time.perf_counter()
    first_token = True
    token_count = 0
    for chunk in response_generator:
        if first_token:
            if stats:
                stats.time_to_first_token_ms = (time.perf_counter() - start_time) * 1000
            first_token = False
        token_count += 1
        yield f"data: {chunk}\n\n"
    end_time = time.perf_counter()
    total_time = end_time - start_time
    if stats:
        stats.total_tokens = token_count
        if total_time > 0:
            stats.tokens_per_second = token_count / total_time
    if include_stats and stats:
        stats_data = {"total_tokens": stats.total_tokens, "ttft_ms": round(stats.time_to_first_token_ms, 2), "tokens_per_sec": round(stats.tokens_per_second, 2), "provider": stats.provider, "model": stats.model}
        yield f"data: [STATS] {json.dumps(stats_data)}\n\n"
    yield "data: [DONE]\n\n"
