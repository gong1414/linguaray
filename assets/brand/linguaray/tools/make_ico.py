from pathlib import Path
import shutil
import sys

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / "dist"
SIZES = (16, 20, 24, 32, 40, 48, 64, 128, 256)


def main() -> None:
    frames = [
        Image.open(DIST / "windows" / "png" / f"linguaray-{size}.png").convert("RGBA")
        for size in SIZES
    ]
    destination = DIST / "windows" / "LinguaRay.ico"
    frames[-1].save(
        destination,
        format="ICO",
        append_images=frames[:-1],
        sizes=[(size, size) for size in SIZES],
    )
    app_destination = (
        ROOT.parents[2]
        / "apps"
        / "desktop"
        / "flutter"
        / "windows"
        / "runner"
        / "resources"
        / "app_icon.ico"
    )
    shutil.copyfile(destination, app_destination)
    print(destination)


if __name__ == "__main__":
    sys.exit(main())
