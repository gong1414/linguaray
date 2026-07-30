//! Built-in traditional MT engines (spec: AI-failure fallback + system-dict).
//! Each engine is a Rust module ported from the corresponding pot-app plugin's
//! JS (leverage ①: turn reversing into JS→Rust porting). v1 ships Google first;
//! others (DeepL/百度/有道/…) follow the same pattern.

pub mod google;

/// A built-in traditional MT engine. Unlike providers (config-driven), these are
/// compiled-in Rust modules implementing their own request construction.
#[async_trait::async_trait]
pub trait TraditionalEngine: Send + Sync {
    fn id(&self) -> &str;
    fn label(&self) -> &str;
    /// Whether the user must supply credentials (Google free: false).
    fn needs_key(&self) -> bool { false }
    /// Translate. `client` is the shared reqwest client (redirect policy none).
    async fn translate(
        &self,
        client: &reqwest::Client,
        text: &str,
        from: &str,
        to: &str,
    ) -> Result<String, crate::error::Error>;
}

/// The static registry of built-in traditional engines.
pub fn registry() -> Vec<Box<dyn TraditionalEngine>> {
    vec![Box::new(google::Google::new())]
}

pub fn find(id: &str) -> Option<Box<dyn TraditionalEngine>> {
    registry().into_iter().find(|e| e.id() == id)
}
