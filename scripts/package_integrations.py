#!/usr/bin/env python3
"""Build distributable LinguaRay integration archives from canonical assets."""

from __future__ import annotations

import argparse
import shutil
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
INTEGRATIONS = ROOT / "integrations"
BRAND = ROOT / "assets" / "brand" / "linguaray" / "dist" / "app-icon"


def copy_tree(source: Path, destination: Path) -> None:
    if not source.is_dir():
        raise FileNotFoundError(source)
    shutil.copytree(source, destination)


def archive(directory: Path, output: Path) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    base = output.with_suffix("")
    created = Path(shutil.make_archive(str(base), "zip", directory.parent, directory.name))
    if created != output:
        created.replace(output)


def package(output: Path) -> None:
    icon_256 = BRAND / "linguaray-app-icon-256.png"
    icon_512 = BRAND / "linguaray-app-icon-512.png"
    for icon in (icon_256, icon_512):
        if not icon.is_file():
            raise FileNotFoundError(icon)

    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="linguaray-integrations-") as temporary:
        staging = Path(temporary)

        popclip = staging / "LinguaRay.popclipext"
        copy_tree(INTEGRATIONS / "popclip", popclip)
        shutil.copy2(icon_256, popclip / "LinguaRay.png")
        archive(popclip, output / "LinguaRay-PopClip.popclipext.zip")

        snipdo = staging / "LinguaRay-SnipDo"
        copy_tree(INTEGRATIONS / "snipdo", snipdo)
        shutil.copy2(icon_256, snipdo / "LinguaRay.png")
        archive(snipdo, output / "LinguaRay-SnipDo.zip")

        raycast = staging / "LinguaRay-Raycast"
        copy_tree(INTEGRATIONS / "raycast", raycast)
        shutil.copy2(icon_512, raycast / "icon.png")
        archive(raycast, output / "LinguaRay-Raycast-source.zip")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="Directory that receives the three integration archives.",
    )
    arguments = parser.parse_args()
    package(arguments.output.resolve())


if __name__ == "__main__":
    main()
