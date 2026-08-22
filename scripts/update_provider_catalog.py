#!/usr/bin/env python3
"""Build the slim models.dev snapshot used by the LinguaRay runtime.

Pinned source commit: 08324a024a9de60e507e08779f6667fbf8a25001

The app never fetches models.dev at runtime. This script is a development-time
step. It downloads the pinned models.dev tree, keeps only the listed provider
IDs, and writes a reproducible JSON snapshot.
"""
from __future__ import annotations

import argparse
import io
import json
import re
import sys
import zipfile
from pathlib import Path
from urllib.request import urlopen

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUT = (
    REPO_ROOT / "crates" / "engine" / "src" / "catalog" / "models_dev_snapshot.json"
)
COMMIT = "08324a024a9de60e507e08779f6667fbf8a25001"
ARCHIVE_URL = f"https://codeload.github.com/anomalyco/models.dev/zip/{COMMIT}"
PROVIDER_IDS = [
    "openai",
    "anthropic",
    "google",
    "deepseek",
    "xai",
    "groq",
    "openrouter",
    "alibaba",
    "zhipuai",
    "moonshotai",
    "siliconflow",
    "modelscope",
    "lmstudio",
]


def parse_simple_toml(text: str) -> dict:
    data: dict = {}
    section = None
    for raw in text.splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1].strip()
            data.setdefault(section, {})
            continue
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        parsed = parse_value(value.strip())
        if section:
            if isinstance(data.get(section), dict):
                data[section][key] = parsed
        else:
            data[key] = parsed
    return data


def parse_value(value: str):
    if value.startswith("[") and value.endswith("]"):
        inner = value[1:-1].strip()
        if not inner:
            return []
        return [parse_value(part.strip()) for part in split_list(inner)]
    if (value.startswith('"') and value.endswith('"')) or (
        value.startswith("'") and value.endswith("'")
    ):
        return value[1:-1]
    if value in ("true", "false"):
        return value == "true"
    return value


def split_list(inner: str) -> list[str]:
    parts = []
    buf = []
    in_str = False
    quote = ""
    for char in inner:
        if in_str:
            buf.append(char)
            if char == quote:
                in_str = False
            continue
        if char in "\"'":
            in_str = True
            quote = char
            buf.append(char)
            continue
        if char == ",":
            parts.append("".join(buf))
            buf = []
            continue
        buf.append(char)
    if buf:
        parts.append("".join(buf))
    return parts


def read_modalities(parsed: dict) -> dict:
    modalities = parsed.get("modalities") or {}
    if not isinstance(modalities, dict):
        return {"input": [], "output": []}
    return {
        "input": list(modalities.get("input") or []),
        "output": list(modalities.get("output") or []),
    }


def load_lab_models(root: Path) -> dict[str, dict]:
    labs = {}
    models_dir = root / "models"
    if not models_dir.exists():
        return labs
    for path in models_dir.rglob("*.toml"):
        rel = path.relative_to(models_dir).with_suffix("")
        labs[str(rel).replace("\\", "/")] = parse_simple_toml(path.read_text(encoding="utf-8"))
    return labs


def collect_provider(root: Path, provider_id: str, labs: dict[str, dict]) -> list[dict]:
    models_dir = root / "providers" / provider_id / "models"
    if not models_dir.exists():
        return []
    collected = []
    for path in models_dir.rglob("*.toml"):
        parsed = parse_simple_toml(path.read_text(encoding="utf-8"))
        model_id = path.stem
        base = {}
        base_model = parsed.get("base_model")
        if isinstance(base_model, str) and base_model in labs:
            base = labs[base_model]
        merged = {**base, **{k: v for k, v in parsed.items() if k != "base_model"}}
        if "modalities" in base or "modalities" in parsed:
            modalities = read_modalities(parsed)
            if not modalities["input"] and not modalities["output"]:
                modalities = read_modalities(base)
        else:
            modalities = {"input": [], "output": []}
        status = merged.get("status")
        if status in ("alpha", "deprecated"):
            continue
        inputs = [str(item).lower() for item in modalities.get("input") or []]
        outputs = [str(item).lower() for item in modalities.get("output") or []]
        if inputs and "text" not in inputs:
            continue
        if outputs and "text" not in outputs:
            continue
        collected.append(
            {
                "id": model_id,
                "name": merged.get("name") or model_id,
                "release_date": merged.get("release_date"),
                "status": status,
                "modalities": modalities,
            }
        )
    collected.sort(key=lambda item: (-_date_key(item.get("release_date")), item["id"]))
    return collected


def _date_key(value) -> int:
    if not isinstance(value, str):
        return 0
    digits = re.sub(r"[^0-9]", "", value)
    return int(digits or "0")


def download_tree(dest: Path) -> Path:
    dest.mkdir(parents=True, exist_ok=True)
    with urlopen(ARCHIVE_URL) as response:
        payload = response.read()
    with zipfile.ZipFile(io.BytesIO(payload)) as archive:
        archive.extractall(dest)
    roots = [path for path in dest.iterdir() if path.is_dir()]
    if not roots:
        raise RuntimeError("models.dev archive had no directory")
    return roots[0]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    parser.add_argument("--source-dir", type=Path, default=None)
    args = parser.parse_args()

    if args.source_dir is not None:
        root = args.source_dir
    else:
        cache = REPO_ROOT / "target" / "models-dev" / COMMIT
        if not cache.exists():
            print(f"==> downloading models.dev@{COMMIT}", flush=True)
            root = download_tree(cache)
        else:
            roots = [path for path in cache.iterdir() if path.is_dir()]
            root = roots[0] if roots else cache

    labs = load_lab_models(root)
    providers = []
    for provider_id in PROVIDER_IDS:
        models = collect_provider(root, provider_id, labs)
        providers.append({"id": provider_id, "models": models})

    snapshot = {
        "source": "models.dev",
        "commit": COMMIT,
        "providers": providers,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    encoded = json.dumps(snapshot, indent=2, sort_keys=False, ensure_ascii=False) + "\n"
    args.out.write_text(encoded, encoding="utf-8")
    print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
