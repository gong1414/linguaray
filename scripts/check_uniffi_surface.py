#!/usr/bin/env python3
"""Check the committed UniFFI callable surface against generated Dart bindings."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BINDINGS = ROOT / "packages/runtime/lib/src/generated/linguaray_runtime.dart"
BASELINE = ROOT / "docs/refactor/UNIFFI_SURFACE.txt"


def current_surface() -> str:
    source = BINDINGS.read_text()
    version = re.search(r"final bindingsVersion = (\d+);", source)
    if version is None:
        raise SystemExit("Unable to find the UniFFI contract version")
    checksums = sorted(
        re.findall(
            r"if \((uniffi_linguaray_runtime_checksum_[a-z0-9_]+)\(\) !=\s*(\d+)\)",
            source,
        )
    )
    if not checksums:
        raise SystemExit("Unable to find UniFFI API checksums")
    lines = [
        "# UniFFI callable surface. Update only in a focused API migration.",
        f"contract_version={version.group(1)}",
        *(f"{name}={value}" for name, value in checksums),
    ]
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--update",
        action="store_true",
        help="Accept the generated surface during an explicit API migration.",
    )
    args = parser.parse_args()
    current = current_surface()
    if args.update:
        BASELINE.write_text(current)
        return 0
    expected = BASELINE.read_text()
    if current != expected:
        print(
            "UniFFI public surface changed. Regenerate bindings and review this "
            "as a focused API migration; use --update only after approval."
        )
        return 1
    print("UniFFI public surface matches the committed baseline.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
