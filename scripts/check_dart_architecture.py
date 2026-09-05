#!/usr/bin/env python3
"""Enforce Dart package purity and desktop presentation boundaries.

This is a source dependency check, not a Dart type checker. Conditional branches
and export/part chains are included; the analyzer checks unresolved symbols.
Composition-owned app shells may assemble features and native windows. Product
feature widgets and view models must consume controllers and ports.
"""
from dataclasses import dataclass
from pathlib import Path
import re
import sys

from check_dart_reachability import URI

ROOT = Path(__file__).resolve().parents[1]
DESKTOP = "apps/desktop/flutter/lib/"
SOURCE = DESKTOP + "src/"
APPLICATION = "packages/application/lib/"
UI = "packages/ui_flutter/lib/"
NATIVE_PACKAGES = {
    "nativeapi", "cnativeapi", "linguaray_runtime", "file_selector", "ffi",
    "flutter_secure_storage", "hotkey_manager", "screen_capturer",
    "path_provider", "package_info_plus", "url_launcher",
}


@dataclass(frozen=True, order=True)
class Violation:
    source: str
    target: str
    rule: str


def directives(source: str) -> list[tuple[str, str]]:
    # Preserve quoted strings while blanking comments, including nested blocks.
    # Dart import URIs themselves use ordinary, non-interpolated strings.
    pattern = re.compile(r'''r?(?:"""[\s\S]*?"""|\'\'\'[\s\S]*?\'\'\'|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')|//[^\n]*|/\*''')
    clean = []
    cursor = 0
    while match := pattern.search(source, cursor):
        clean.append(source[cursor:match.start()])
        token = match[0]
        end = match.end()
        if token == "/*":
            depth = 1
            while depth and end < len(source):
                marker = re.search(r"/\*|\*/", source[end:])
                if marker is None:
                    end = len(source)
                    break
                depth += 1 if marker[0] == "/*" else -1
                end += marker.end()
            token = source[match.start():end]
        clean.append(re.sub(r"[^\n]", " ", token) if token.startswith(("//", "/*")) else token)
        cursor = end
    clean.append(source[cursor:])
    text = "".join(clean)
    return [(match[1], uri) for match in re.finditer(r"^\s*(import|export|part)\s+(?!of\b)([^;]+);", text, re.MULTILINE) for uri in URI.findall(match[2])]


def inspect(root: Path) -> list[Violation]:
    root = root.resolve()
    packages = {}
    for manifest in [root / "apps/desktop/flutter/pubspec.yaml", *sorted((root / "packages").glob("*/pubspec.yaml"))]:
        if not manifest.exists():
            continue
        name = re.search(r"^name:\s*(\S+)", manifest.read_text(), re.MULTILINE)
        if name:
            packages[name[1]] = manifest.parent / "lib"
    sources = {p.relative_to(root).as_posix(): p.read_text() for lib in packages.values() for p in lib.rglob("*.dart")}
    graph = {}
    for path, text in sources.items():
        edges = []
        for kind, uri in directives(text):
            if uri.startswith("package:"):
                package, _, rest = uri[8:].partition("/")
                target = (packages[package] / rest).resolve().relative_to(root).as_posix() if package in packages else uri
            elif ":" in uri:
                target = uri
            else:
                resolved = ((root / path).parent / uri).resolve()
                target = resolved.relative_to(root).as_posix() if resolved.is_relative_to(root) else str(resolved)
            edges.append((kind, target))
        graph[path] = edges

    def surface(target: str, seen: set[str]):
        if target in seen:
            return
        seen.add(target)
        yield target
        for kind, child in graph.get(target, []):
            if kind in {"export", "part"}:
                yield from surface(child, seen)

    found = set()
    for path, source in sources.items():
        if path.startswith(SOURCE) and path[len(SOURCE):].split('/')[0] not in {"app", "features", "platform", "shared", "i18n", "catalog"}:
            found.add(Violation(path, path, "desktop-directory"))
        presentation = path.startswith((SOURCE + "features/", SOURCE + "shared/")) and (bool(re.search(r"\bextends\s+(?:ConsumerStatefulWidget|ConsumerWidget|StatefulWidget|StatelessWidget|HookWidget|State\s*<|ConsumerState\s*<)", source)) or "view_model" in path or path.endswith("/update_coordinator.dart"))
        if "/catalog/" in path:
            presentation = False
        for _, direct in graph[path]:
            for target in surface(direct, set()):
                package = target[8:].split('/')[0] if target.startswith("package:") else None
                native = package in NATIVE_PACKAGES or target.startswith("packages/runtime/lib/") or target in {"dart:io", "dart:ffi", "dart:ui"}
                if path.startswith(APPLICATION):
                    if not (target.startswith(APPLICATION) or target.startswith("dart:") and not native or package == "pub_semver"):
                        found.add(Violation(path, target, "application-purity"))
                if path.startswith(UI) and "/testing/" not in path and (target.startswith((APPLICATION, DESKTOP, "packages/runtime/lib/")) or native):
                    found.add(Violation(path, target, "design-system-purity"))
                implementation = target in {SOURCE + "app/runtime.dart", SOURCE + "app/settings/settings_store.dart"} or target.startswith(SOURCE + "app/windows/") or target.startswith(SOURCE + "features/") and "/data/" in target
                if presentation and (native or implementation):
                    found.add(Violation(path, target, "presentation-port"))
    return sorted(found)


def main() -> int:
    violations = inspect(ROOT)
    for item in violations:
        print(f"{item.rule}: {item.source} -> {item.target}", file=sys.stderr)
    if violations:
        return 1
    print("Dart architecture passed: package purity and feature presentation boundaries.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
