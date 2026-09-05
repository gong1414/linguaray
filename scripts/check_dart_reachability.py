#!/usr/bin/env python3
"""Reject orphan Dart libraries and catalog imports in the desktop product.

This checks library reachability, not member-level liveness. Generated public
FFI exports remain part of their package contract. The analyzer complements
this check for unused imports, private declarations, and unreachable statements.
"""

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
DIRECTIVE = re.compile(r"^\s*(?:import|export|part)\s+(?!of\b)([^;]+);", re.MULTILINE)
URI = re.compile(r"['\"]([^'\"]+)['\"]")


def inspect(root: Path) -> tuple[list[Path], list[Path]]:
    root = root.resolve()
    manifests = [root / "apps/desktop/flutter/pubspec.yaml", *sorted((root / "packages").glob("*/pubspec.yaml"))]
    packages = {}
    for manifest in manifests:
        name = re.search(r"^name:\s*(\S+)", manifest.read_text(), re.MULTILINE)
        if name:
            packages[name[1]] = manifest.parent / "lib"
    files = {file for library in packages.values() for file in library.rglob("*.dart")}
    graph = {}
    for file in files:
        dependencies = set()
        for directive in DIRECTIVE.findall(file.read_text()):
            for uri in URI.findall(directive):
                if uri.startswith("package:"):
                    package, _, relative = uri[8:].partition("/")
                    if package not in packages:
                        continue
                    target = packages[package] / relative
                elif ":" in uri:
                    continue
                else:
                    target = (file.parent / uri).resolve()
                if target in files:
                    dependencies.add(target)
        graph[file] = dependencies

    def reachable(entries: list[Path]) -> set[Path]:
        pending, seen = entries[:], set()
        while pending:
            file = pending.pop()
            if file in seen:
                continue
            seen.add(file)
            pending.extend(graph.get(file, ()))
        return seen

    product = reachable([root / "apps/desktop/flutter/lib/main.dart"])
    # Explicit development entry points; ordinary tests cannot keep dead
    # product libraries alive just by importing them.
    development = reachable([
        root / "apps/desktop/flutter/lib/widgetbook.dart",
        root / "packages/ui_flutter/lib/testing.dart",
    ])
    leaks = [file for file in product if "/catalog/" in file.as_posix() or file.name in {"widgetbook.dart", "golden_fonts.dart"}]
    return sorted(files - product - development), sorted(leaks)


def main() -> int:
    orphans, leaks = inspect(ROOT)
    for label, files in [("Unreachable library", orphans), ("Development code imported by product", leaks)]:
        for file in files:
            print(f"{label}: {file.relative_to(ROOT)}", file=sys.stderr)
    if orphans or leaks:
        return 1
    print("Dart library reachability passed; development fixtures are isolated.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
