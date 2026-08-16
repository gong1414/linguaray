//! Built-in traditional MT engines (AI-failure fallback).
//! Implementations live in `plugins/drivers/traditional`.

/// A built-in traditional MT engine. Unlike providers (config-driven), these are
/// compiled-in Rust modules implementing their own request construction.
#[async_trait::async_trait]
pub trait TraditionalEngine: Send + Sync {
    fn id(&self) -> &str;
    fn label(&self) -> &str;
    /// Whether the user must supply credentials (Google GTX: false).
    fn needs_key(&self) -> bool {
        false
    }
    /// Translate. `key` is the keystore secret when [`needs_key`] is true.
    async fn translate(
        &self,
        client: &reqwest::Client,
        text: &str,
        from: &str,
        to: &str,
        key: Option<&str>,
    ) -> Result<String, crate::error::Error>;
}

/// The static registry of built-in traditional engines.
pub fn registry() -> Vec<Box<dyn TraditionalEngine>> {
    crate::plugins::drivers::traditional::all()
}

pub fn find(id: &str) -> Option<Box<dyn TraditionalEngine>> {
    crate::plugins::drivers::traditional::find(id)
}
