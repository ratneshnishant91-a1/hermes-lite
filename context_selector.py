from __future__ import annotations


class ContextSelector:
    def __init__(self, memory, skills, max_memories: int = 5):
        self.memory = memory
        self.skills = skills
        self.max_memories = max_memories
        self.loaded_skills: set[str] = set()

    def select_context(self, user_message: str, current_messages: list[dict]) -> tuple[list[dict], dict]:
        stats = {"memories_found": 0, "skills_loaded": [], "context_added": False}
        memories = self.memory.search(user_message, limit=self.max_memories)
        if memories:
            stats["memories_found"] = len(memories)
            memory_context = {"role": "system", "content": "Relevant memories:\n" + "\n".join(f"- {m}" for m in memories)}
            current_messages.insert(1, memory_context)
            stats["context_added"] = True
        skill_keywords = {
            "coding": ["code", "program", "python", "file", "write", "debug"],
            "research": ["search", "find", "look up", "research", "web"],
            "git": ["git", "commit", "push", "repository", "branch"],
        }
        message_lower = user_message.lower()
        for skill_name, keywords in skill_keywords.items():
            if any(kw in message_lower for kw in keywords):
                if skill_name not in self.loaded_skills:
                    try:
                        self.skills.load(skill_name)
                        self.loaded_skills.add(skill_name)
                        stats["skills_loaded"].append(skill_name)
                    except FileNotFoundError:
                        pass
        return current_messages, stats
