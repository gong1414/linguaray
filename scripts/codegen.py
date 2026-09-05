#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]

PYTHON_STEPS = (
    "scripts/generate/runtime_bindings.py",
    "scripts/generate/languages.py",
)


def main() -> int:
    for relative_path in PYTHON_STEPS:
        result = run_script(REPO_ROOT / relative_path)
        if result:
            return result
    translations = subprocess.run(
        ["dart", "run", "slang"],
        cwd=REPO_ROOT / "apps/desktop/flutter",
        check=False,
    ).returncode
    if translations:
        return translations
    return run_script(REPO_ROOT / "scripts/format.py")


def run_script(script: Path) -> int:
    command = [sys.executable, str(script)]
    return subprocess.run(command, cwd=REPO_ROOT, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
