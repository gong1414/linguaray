# LinguaRay brand assets

This directory contains the production brand system for **LinguaRay**. The
canonical spelling is always `LinguaRay`: capital `L`, capital `R`, no space.

## Concept

The symbol is an `LR` monogram separated by one diagonal negative-space ray.
The left form represents the source side and the right form represents the
translated result. The master geometry is intentionally flat and survives as a
single-color 18 px tray glyph.

## Colors

| Role | Value |
|---|---|
| Lingua blue | `#2859D9` |
| Ray teal | `#18A6A6` |
| Ray teal on dark | `#34C0BE` |
| Navy app background | `#13233F` |
| Graphite wordmark | `#172033` |
| Paper | `#F7F9FC` |

Color never carries the structure by itself. Use the monochrome SVGs wherever
the operating system controls icon tinting.

## Deliverables

- `dist/svg/linguaray-logo-primary.svg` — primary horizontal lockup
- `dist/svg/linguaray-wordmark.svg` — outlined wordmark
- `dist/svg/linguaray-symbol.svg` — pure symbol
- `dist/svg/linguaray-symbol-mono-{black,white}.svg` — one-color masters
- `dist/app-icon/linguaray-app-icon-1024.png` — 1024 px square app icon
- `dist/macos/LinguaRay.iconset/` and `dist/macos/LinguaRay.icns`
- `dist/windows/LinguaRay.ico` — 16, 20, 24, 32, 40, 48, 64, 128, 256 px
- `dist/tray/` — 18 px and 36 px monochrome tray assets, plus a 32 px macOS template
- `dist/readme/` — light and dark README lockups in SVG and PNG
- `dist/preview/linguaray-brand-board.png` — visual QA board

The generated application assets are also installed into the existing Flutter,
macOS, and Windows resource locations.

## Rules

- Do not add glow, blur, gradients, glass, shadows, stars, language cards, flags,
  globes, chat bubbles, or extra rays.
- Do not recolor only the word `Ray`; the complete wordmark uses one color.
- Do not redraw the ray or change its angle independently between sizes.
- Do not use the full-color symbol in a system template or tray slot.
- Preserve at least one-quarter of the symbol width as clear space around a
  standalone mark.

## Regeneration

The generator uses the repository's MiSans Semibold font only to produce the
outlined wordmark; the distributed SVG does not require the font at runtime.
From this directory, run:

```bash
npm install
python3 -m pip install -r requirements.txt
swift tools/outline_wordmark.swift \
  ../../../apps/desktop/flutter/resources/fonts/MiSans-Semibold.ttf \
  build/wordmark-outline.json
npm run generate
python3 tools/make_ico.py
iconutil -c icns dist/macos/LinguaRay.iconset \
  -o dist/macos/LinguaRay.icns
```

`iconutil` and the Swift outline step require macOS. The Node generator also
installs the generated Flutter and macOS resources into their application
locations; the ICO builder installs the Windows icon.
