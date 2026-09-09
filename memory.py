class Memory:
    def __init__(self, store):
        self.store = store

    def remember(self, content: str, source: str = "agent") -> str:
        return self.store.remember(content, source)

    def search(self, query: str, limit: int = 5) -> list[str]:
        return self.store.search_memories(query, limit)
