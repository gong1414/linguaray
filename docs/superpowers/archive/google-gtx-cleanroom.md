# Google GTX clean-room (PR-6f)

- **Date:** 2026-08-14
- **Author:** LinguaRay plugin-core session (PR-6f)
- **New module:** `src-tauri/src/plugins/drivers/traditional/google.rs`
- **Supersedes:** `src-tauri/src/engines/google_legacy.rs` (removed from the crate; pot-desktop GPL-3.0 lineage)

## Contract used

Publicly observed unofficial GTX endpoint, documented in plugin-core spec §12.4 (not pot source):

- `GET {origin}/translate_a/single?client=gtx&sl=&tl=&dt=t&q=`
- Response: JSON array; `[0]` is a list of segments; each segment's `[0]` is the translated string. Concatenate those strings.

This is **not** Google Cloud Translation. It has no SLA and may stop working without notice. `engines.json` marks `unofficial_gtx: true`.

## What was not used

- pot-desktop source (`src/` plugins, `.potext` loaders)
- The isolated `google_legacy.rs` body as a copy source (the file existed only as a provenance fence)

## Fixtures

Independent constructed JSON (`[["你好","hello"],["世界","world"]]`), not copied from pot test fixtures.

## Declaration

This rewrite implements the public gtx observation recorded in §12.4. It does not include pot-desktop code.
