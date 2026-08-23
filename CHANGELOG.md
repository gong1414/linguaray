# Changelog

All notable changes to LinguaRay will be documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and releases
use [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.6.0] - 2026-08-23

### Added

- Tray-first macOS and Windows experience with native menus, global shortcuts,
  quick translation, selection translation, input translation, and screenshot
  OCR.
- Offline ECDICT English-Chinese dictionary with 50,000 reproducibly selected
  entries and no API key requirement.
- Apple system translation on macOS 15 and later, including the native
  first-use language download flow.
- Built-in Google Web translation and a fixed provider catalog covering
  traditional APIs, OpenAI-compatible endpoints, and local servers.
- History, favourites, vocabulary, glossaries, speech, secure credential
  storage, local API integration, and update checks.
- Widgetbook catalog, golden tests, native integration smoke tests, and
  dual-platform release builds.

### Changed

- Rebuilt the application around Flutter, a Rust runtime, and UniFFI with the
  user interface and functional core kept behind typed controller boundaries.
- Reworked translation and dictionary services into separate configurable
  sections, with ECDICT as the default dictionary on both supported platforms.
