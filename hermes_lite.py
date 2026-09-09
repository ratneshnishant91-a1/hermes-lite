"""
Hermes-Lite Python Wrapper (Rust Core)

Calls the Rust binary via subprocess. Zero FFI overhead, stable ABI.
Requires `hermes-lite` binary in PATH or local `target/release/`.
"""

import json
import os
import subprocess
from pathlib import Path
from typing import Any, Dict, Optional


class Agent:
    """Python wrapper for the Rust Hermes-Lite agent."""

    def __init__(self, config_path: Optional[str] = None, api_key: Optional[str] = None):
        self.config_path = config_path
        self.api_key = api_key or os.getenv("OPENAI_API_KEY")
        if not self.api_key:
            raise RuntimeError("Set OPENAI_API_KEY or pass api_key=...")

        self._bin = self._find_binary()

    def _find_binary(self) -> Path:
        """Locate the hermes-lite binary."""
        # 1. Local build
        local = Path(__file__).parent / "target" / "release" / "hermes-lite"
        if local.exists():
            return local
        # 2. PATH
        import shutil
        bin_path = shutil.which("hermes-lite")
        if bin_path:
            return Path(bin_path)
        raise FileNotFoundError(
            "hermes-lite binary not found. Run `cargo build --release` or install via pip."
        )

    def _run_cmd(self, args: list[str]) -> str:
        """Execute Rust binary and capture stdout."""
        env = os.environ.copy()
        env["OPENAI_API_KEY"] = self.api_key
        if self.config_path:
            args = ["--config", self.config_path] + args

        try:
            result = subprocess.run(
                [str(self._bin)] + args,
                capture_output=True,
                text=True,
                check=True,
                env=env,
                timeout=60,
            )
            return result.stdout.strip()
        except subprocess.CalledProcessError as e:
            raise RuntimeError(f"Rust agent failed: {e.stderr.strip()}") from e
        except subprocess.TimeoutExpired:
            raise RuntimeError("Rust agent timed out")

    def run(self, prompt: str) -> str:
        """Run a single prompt through the Rust agent."""
        return self._run_cmd(["run", prompt])

    def stats(self) -> Dict[str, Any]:
        """Get learning stats as a dict."""
        output = self._run_cmd(["stats"])
        return json.loads(output)

    def chat(self):
        """Start an interactive REPL (blocks)."""
        env = os.environ.copy()
        env["OPENAI_API_KEY"] = self.api_key
        args = [str(self._bin), "chat"]
        if self.config_path:
            args = [str(self._bin), "--config", self.config_path, "chat"]
        subprocess.run(args, env=env)


if __name__ == "__main__":
    # Example usage
    agent = Agent()
    print("Rust core version:")
    print(agent.run("What is your core language?"))
    print("\nStats:")
    print(json.dumps(agent.stats(), indent=2))
