//! Built-in traditional MT engines.
//!
//! These are the "AI-failure fallback + dictionary" layer (decided in grilling).
//! They are NOT AI providers and NOT plugins — they're compiled-in Rust modules.
//!
//! Implementation strategy (leverage ① from grilling): each engine's request
//! construction (signing, params, parsing) is PORTED from pot's `.potext` JS
//! source (https://github.com/pot-app/pot-desktop) rather than reverse-engineered
//! from scratch. That turns "days of reversing" into "hours of JS→Rust porting".
//!
//! For now this is a skeleton registry with no concrete engines — they're
//! Phase 3 of the roadmap. The registry returns empty so the app compiles and
//! the picker shows only AI providers until traditional engines land.

use crate::Engine;

/// The static registry of built-in traditional engines.
///
/// Populated as engines are ported. The translate() command looks engines up by
/// `id()` here (after checking AI providers).
pub fn registry() -> Vec<Box<dyn Engine>> {
    // Phase 3 will append: Google, DeepL, 百度, 有道, ...
    vec![]
}
