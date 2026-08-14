//! Built-in traditional MT engines (AI-failure fallback).
//! Google is isolated as `google_legacy` until the §12.4 clean-room rewrite.

pub mod google_legacy;

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
    vec![Box::new(google_legacy::Google::new())]
}

pub fn find(id: &str) -> Option<Box<dyn TraditionalEngine>> {
    registry().into_iter().find(|e| e.id() == id)
}
