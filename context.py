from compressor import ContextCompressor


class Context:
    def __init__(self, system_prompt: str, store, session_id: int, model_generate_fn):
        self.store = store
        self.session_id = session_id
        self.compressor = ContextCompressor(model_generate_fn)
        saved = store.load_messages(session_id)
        if saved:
            self.messages = saved
            if self.messages[0].get("role") != "system":
                self.messages.insert(0, {"role": "system", "content": system_prompt})
        else:
            self.messages = [{"role": "system", "content": system_prompt}]
            self._persist(self.messages[0])
        self._last_compression_stats = None

    def _append(self, message: dict) -> None:
        self.messages.append(message)
        self._persist(message)

    def _persist(self, message: dict) -> None:
        self.store.save_message(self.session_id, message)

    def add_user(self, text: str) -> None:
        self._append({"role": "user", "content": text})

    def add_assistant(self, text: str) -> None:
        self._append({"role": "assistant", "content": text})

    def add_tool_call(self, call: dict) -> None:
        self._append({"role": "assistant", "tool_calls": [call]})

    def add_tool_result(self, call_id, name, result) -> None:
        self._append({"role": "tool", "tool_call_id": call_id, "name": name, "content": str(result)})

    def prompt_messages(self, maximum: int = 30) -> list[dict]:
        messages = self.messages
        if self.compressor.should_compress(messages, threshold=maximum):
            messages, stats = self.compressor.compress(messages)
            self._last_compression_stats = stats
        if len(messages) > maximum + 5:
            system = messages[0]
            messages = [system, *messages[-(maximum + 5):]]
        return messages

    def get_compression_stats(self) -> dict | None:
        return self._last_compression_stats
