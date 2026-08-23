#!/usr/bin/env python3
"""Build LinguaRay's compact offline ECDICT asset from a pinned source."""

from __future__ import annotations

import csv
import gzip
import hashlib
import json
import sys
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/engine/assets/ecdict-compact.json.gz"
SOURCE_COMMIT = "bc015ed2e24a7abef49fc6dbbb7fe32c1dadaf8b"
SOURCE_URL = (
    "https://raw.githubusercontent.com/skywind3000/ECDICT/"
    f"{SOURCE_COMMIT}/ecdict.csv"
)
SOURCE_SHA256 = "1a6947e04785db63613a92e14903cdae7954f7e84860b10e68e5c7cbb3f9c3cf"
ENTRY_LIMIT = 50_000


def numeric(value: str) -> int:
    try:
        result = int(value)
    except (TypeError, ValueError):
        return 0
    return result if result > 0 else 0


def rank(row: dict[str, str]) -> tuple[int, int, int, int, str]:
    collins = numeric(row.get("collins", ""))
    oxford = numeric(row.get("oxford", ""))
    frequencies = [
        value
        for value in (numeric(row.get("bnc", "")), numeric(row.get("frq", "")))
        if value
    ]
    frequency = min(frequencies, default=10_000_000)
    tagged = bool(row.get("tag", "").strip())
    tier = 0 if collins or oxford else 1 if frequencies else 2 if tagged else 3
    return (
        tier,
        frequency,
        -max(collins, oxford),
        len(row.get("word", "")),
        row.get("word", "").casefold(),
    )


def main() -> int:
    source = urllib.request.urlopen(SOURCE_URL, timeout=60).read()
    digest = hashlib.sha256(source).hexdigest()
    if digest != SOURCE_SHA256:
        print(f"unexpected ECDICT sha256: {digest}", file=sys.stderr)
        return 1

    decoded = source.decode("utf-8-sig").splitlines(keepends=True)
    rows = []
    for row in csv.DictReader(decoded):
        word = row.get("word", "").strip()
        translation = row.get("translation", "").strip()
        definition = row.get("definition", "").strip()
        if not word or not (translation or definition):
            continue
        rows.append(row)

    selected = sorted(rows, key=rank)[:ENTRY_LIMIT]
    entries = [
        {
            "w": row["word"].strip(),
            "p": row.get("phonetic", "").strip(),
            "t": row.get("translation", "").strip(),
            "d": row.get("definition", "").strip(),
            "x": row.get("exchange", "").strip(),
        }
        for row in sorted(selected, key=lambda item: item["word"].casefold())
    ]

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(entries, ensure_ascii=False, separators=(",", ":"))
    with OUTPUT.open("wb") as raw:
        with gzip.GzipFile(filename="", fileobj=raw, mode="wb", mtime=0) as archive:
            archive.write(payload.encode("utf-8"))

    print(f"wrote {len(entries)} entries to {OUTPUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
