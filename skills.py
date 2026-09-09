from __future__ import annotations

from pathlib import Path


class Skills:
    def __init__(self, root: str = "skills"):
        self.root = Path(root)
        self.loaded: dict[str, str] = {}

    def list(self) -> list[dict[str, str]]:
        items = []
        for skill_md in sorted(self.root.glob("*/SKILL.md")):
            name = skill_md.parent.name
            description = self._description(skill_md)
            items.append({"name": name, "description": description})
        return items

    def catalog_text(self) -> str:
        lines = ["Available skills:"]
        for item in self.list():
            lines.append(f"- {item['name']}: {item['description']}")
        lines.append("Load a skill with skill_load before following it.")
        return "\n".join(lines)

    def load(self, name: str) -> str:
        path = self.root / name / "SKILL.md"
        if not path.exists():
            raise FileNotFoundError(f"Unknown skill: {name}")
        body = path.read_text(encoding="utf-8")
        self.loaded[name] = body
        return body

    def _description(self, path: Path) -> str:
        for line in path.read_text(encoding="utf-8").splitlines():
            if line.lower().startswith("description:"):
                return line.split(":", 1)[1].strip().strip('"')
        return "No description."
