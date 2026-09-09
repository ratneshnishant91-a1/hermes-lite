from __future__ import annotations

from pathlib import Path


class Workspace:
    def __init__(self, root: str = "workspace"):
        self.root = Path(root).resolve()
        self.files = self.root / "files"
        self.files.mkdir(parents=True, exist_ok=True)

    def resolve(self, path: str) -> Path:
        target = (self.files / path).resolve()
        if not str(target).startswith(str(self.files)):
            raise PermissionError(f"Path escapes workspace: {path}")
        return target
