# IslandPot

A cross-platform, open-source translation / OCR / TTS desktop tool — an
actively-maintained successor to [pot-desktop](https://github.com/pot-app/pot-desktop)
(which stopped updating), and the open answer to its author's later closed-source
paid version ([manggo](https://manggo.pylogmon.cn/)).

**Status:** early scaffold (v1 in progress). Heads-down build before open-source.

## What makes it different

The headline feature is a **cc-switch-style AI provider catalog**: the user picks
a preset provider (OpenAI / Anthropic / Gemini / local Ollama / API 中转站 …),
fills in an API key, and it works — instead of the generic "OpenAI-compatible
form where you hand-edit base_url + model" that every other translation tool
uses. Adding a provider = adding one line of config, not reverse-engineering code.

Three legs, decided in the design grilling:

1. **AI-native** — the LLM is the default, translation-tuned engine (auto language
   detect, terminology consistency, long-text segmentation). Not a chat app —
   just translation, done well.
2. **Privacy / local-first** — local LLM (Ollama), local OCR (PaddleOCR), local
   TTS are first-class. No telemetry. Users supply their own keys.
3. **Continuously-maintained open source** — will never go closed/paid.

Traditional MT engines (DeepL / Google / 百度 / 有道 / …) are built-in Rust modules
that act as the **AI-failure fallback** and back the **system-dictionary** lookup
mode (word definitions, where LLMs are weak).

## Tech stack

- **Tauri 2** + **Rust** backend, **SolidJS** + TypeScript frontend
- **Platforms:** Windows + macOS (Linux out of scope for v1)
- v1 has **no plugin system** — engines are built-in. Plugin/WASM extensibility is
  deferred to post-v1.

## Develop

Requirements: Node 20+, pnpm, a working Rust toolchain (stable), Xcode CLT (macOS)
or the MSVC toolchain (Windows).

```bash
pnpm install
pnpm tauri dev      # launch the dev window
pnpm tauri build    # production bundle
```

## Project layout

```
src/                 # SolidJS frontend (the translation popup / settings UI)
src-tauri/src/
  lib.rs             # translate contract + Tauri commands (translate, list_engines)
  providers.rs       # AI preset catalog (the core differentiator) + unified caller
  engines/mod.rs     # built-in traditional MT engines (Phase 3: port from pot .potext)
```

## Roadmap (solo, ~1hr/day, must-ship)

- **Phase 0 — foundation** ✅ Tauri 2 + SolidJS scaffold, translate contract wired
- **Phase 1 — unified pipeline + AI preset catalog** (the headline feature)
- **Phase 2 — input/clipboard UI + popup polish**
- **Phase 3 — built-in traditional engines** (port from pot's `.potext` JS) + system dict
- **Phase 4 — PaddleOCR screenshot translate**
- **Phase 5 — TTS + external invocation**
- **Phase 6 — cross-platform parity + packaging, then open-source**

## License

MIT.
