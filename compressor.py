from __future__ import annotations

PRUNE_PROTECT_RECENT = 10
PRUNE_MINIMUM_RECLAIM = 5

COMPACTION_TEMPLATE = """Summarize this conversation section into structured context for task resumption.

Extract:
- Goal: What is the user trying to accomplish?
- Progress: What has been done so far?
- Decisions: Key choices made (tools used, files created, etc.)
- Files: Which files were read/written/modified?
- Errors: Any failures or blockers?
- Next Steps: What should happen next?

Format as concise bullet points. Do not narrate. Do not include greetings.

CONVERSATION TO SUMMARIZE:
{conversation}

STRUCTURED SUMMARY:
"""


def prune_tool_outputs(messages: list[dict]) -> tuple[list[dict], int]:
    if len(messages) <= 20:
        return messages, 0
    tool_results = []
    for i, msg in enumerate(messages):
        if msg.get("role") == "tool":
            tool_results.append(i)
    if len(tool_results) <= PRUNE_PROTECT_RECENT:
        return messages, 0
    protected = set(tool_results[-PRUNE_PROTECT_RECENT:])
    to_compress = tool_results[:-PRUNE_PROTECT_RECENT]
    if len(to_compress) < PRUNE_MINIMUM_RECLAIM:
        return messages, 0
    pruned = []
    compressed_count = 0
    for i, msg in enumerate(messages):
        if i in protected:
            pruned.append(msg)
        elif i in to_compress:
            name = msg.get("name", "tool")
            content = msg.get("content", "")
            char_count = len(str(content))
            if char_count > 200:
                compressed = {"role": "tool", "tool_call_id": msg.get("tool_call_id"), "name": name, "content": f"[{name}] output compressed ({char_count} chars)"}
            else:
                compressed = msg
            pruned.append(compressed)
            compressed_count += 1
        else:
            pruned.append(msg)
    return pruned, compressed_count


def compress_with_llm(messages: list[dict], model_generate_fn) -> list[dict]:
    if len(messages) <= 25:
        return messages
    head_size = 3
    tail_size = 8
    head = messages[:head_size]
    tail = messages[-tail_size:]
    middle = messages[head_size:-tail_size]
    if len(middle) < 5:
        return messages
    middle_text = []
    for msg in middle:
        role = msg.get("role", "unknown")
        content = msg.get("content") or msg.get("tool_calls") or "[tool result]"
        middle_text.append(f"{role}: {content}")
    conversation_str = "\n".join(middle_text[-50:])
    prompt = COMPACTION_TEMPLATE.format(conversation=conversation_str)
    summary_messages = [{"role": "system", "content": "You summarize conversations into structured task context."}, {"role": "user", "content": prompt}]
    try:
        response = model_generate_fn(summary_messages, [])
        summary_text = response.text or "No summary generated."
        summary_message = {"role": "system", "content": f"[COMPRESSED CONTEXT]\n{summary_text}"}
        return head + [summary_message] + tail
    except Exception as e:
        summary_message = {"role": "system", "content": f"[CONTEXT COMPRESSED] Middle section removed due to error: {e}"}
        return head + [summary_message] + tail


class ContextCompressor:
    def __init__(self, model_generate_fn):
        self.model_generate = model_generate_fn
        self.compression_count = 0

    def should_compress(self, messages: list[dict], threshold: int = 30) -> bool:
        return len(messages) >= threshold

    def compress(self, messages: list[dict]) -> tuple[list[dict], dict]:
        original_count = len(messages)
        pruned, pruned_count = prune_tool_outputs(messages)
        if self.should_compress(pruned):
            compressed = compress_with_llm(pruned, self.model_generate)
        else:
            compressed = pruned
        self.compression_count += 1
        stats = {"original_count": original_count, "compressed_count": len(compressed), "pruned_tool_outputs": pruned_count, "compression_round": self.compression_count}
        return compressed, stats
