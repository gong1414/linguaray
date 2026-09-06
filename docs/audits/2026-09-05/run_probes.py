#!/usr/bin/env python3
"""Run the audit fixtures. A nonzero result is expected on fa398c1."""
from pathlib import Path
import importlib.util
import os
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def catalog_probe():
    spec = importlib.util.spec_from_file_location(
        "catalog_generator", ROOT / "scripts/update_provider_catalog.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    with tempfile.TemporaryDirectory(prefix="linguaray-catalog-audit-") as directory:
        root = Path(directory)
        fixture = root / "providers/openrouter/models/anthropic/claude-test.toml"
        fixture.parent.mkdir(parents=True)
        fixture.write_text(
            'name = "Claude Test"\n[modalities]\ninput = ["text"]\noutput = ["text"]\n'
        )
        actual = module.collect_provider(root, "openrouter", {})[0]["id"]
    expected = "anthropic/claude-test"
    print(f"Catalog model ID: expected={expected!r}; actual={actual!r}", flush=True)
    return actual == expected


def rust_probe():
    # Cargo integration tests can access the same dependencies as the runtime.
    # Refuse to overwrite an existing test and always remove our temporary file.
    destination = ROOT / "packages/runtime/rust/tests/linguaray_audit_protocol.rs"
    with destination.open("x") as output:
        output.write((HERE / "protocol_probe.rs").read_text())
    try:
        return subprocess.run(
            ["cargo", "test", "--locked", "-p", "linguaray_runtime", "--test",
             "linguaray_audit_protocol", "--", "--nocapture", "--test-threads=1"],
            cwd=ROOT,
        ).returncode == 0
    finally:
        destination.unlink()


def detection_probe():
    dart = os.environ.get("LINGUARAY_AUDIT_DART") or shutil.which("dart")
    if not dart:
        print("Dart probe skipped: put the project SDK on PATH.", flush=True)
        return False
    return subprocess.run(
        [dart, f"--packages={ROOT / '.dart_tool/package_config.json'}",
         str(HERE / "detection_probe.dart")], cwd=ROOT,
    ).returncode == 0


if __name__ == "__main__":
    results = [catalog_probe(), rust_probe(), detection_probe()]
    raise SystemExit(0 if all(results) else 1)
