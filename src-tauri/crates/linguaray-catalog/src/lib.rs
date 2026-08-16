mod validate;

pub use linguaray_contracts::{AuthKind, ProtocolKind, SupportTier};
pub use validate::{validate_catalog, validate_endpoint, CatalogError};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogFile {
    pub schema_version: u32,
    pub catalog_revision: u32,
    pub providers: Vec<CatalogProvider>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogProvider {
    pub id: String,
    pub label: String,
    pub protocol: ProtocolKind,
    pub auth: AuthKind,
    pub endpoint: String,
    pub default_model: String,
    pub needs_key: bool,
    pub support_tier: SupportTier,
    #[serde(default)]
    pub requires_user_endpoint: bool,
    #[serde(default)]
    pub models_url: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub console_url: Option<String>,
    #[serde(default)]
    pub docs: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

pub fn load() -> Result<CatalogFile, CatalogError> {
    let raw = include_str!("../providers.json");
    let file: CatalogFile = serde_json::from_str(raw)?;
    validate_catalog(&file)?;
    Ok(file)
}

pub fn get(id: &str) -> Option<CatalogProvider> {
    load().ok()?.providers.into_iter().find(|p| p.id == id)
}

/// Traditional MT metadata (`engines.json`). Not the 21 AI presets.
#[derive(Debug, Clone, Deserialize)]
pub struct EngineCatalogFile {
    pub schema_version: u32,
    pub engines: Vec<CatalogEngine>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogEngine {
    pub id: String,
    pub label: String,
    pub protocol: String,
    pub endpoint: String,
    pub needs_key: bool,
    #[serde(default)]
    pub unofficial_gtx: bool,
    #[serde(default)]
    pub docs: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

pub const REQUIRED_ENGINE_IDS: &[&str] =
    &["google", "deepl", "microsoft", "baidu", "youdao", "tencent"];

pub fn load_engines() -> Result<EngineCatalogFile, CatalogError> {
    let raw = include_str!("../engines.json");
    let file: EngineCatalogFile = serde_json::from_str(raw)?;
    validate::validate_engines(&file)?;
    Ok(file)
}

pub fn engine(id: &str) -> Option<CatalogEngine> {
    load_engines().ok()?.engines.into_iter().find(|e| e.id == id)
}

pub const REQUIRED_IDS: &[&str] = &[
    "openai",
    "anthropic",
    "gemini",
    "deepseek",
    "openrouter",
    "azure-openai",
    "ollama",
    "custom",
    "zhipu-glm",
    "kimi",
    "minimax",
    "bailian",
    "doubao",
    "siliconflow",
    "modelscope",
    "stepfun",
    "xiaomi-mimo",
    "nvidia-nim",
    "groq",
    "mistral",
    "together",
];

pub const RELAY_HOST_DENY: &[&str] = &[
    "packycode.com",
    "cubence.com",
    "aigocode.com",
    "right.codes",
    "aicodemirror.com",
];

#[cfg(test)]
mod tests {
    #[test]
    fn shipped_catalog_validates() {
        let c = crate::load().expect("catalog");
        assert_eq!(c.providers.len(), 21);
        assert_eq!(c.schema_version, 1);
    }

    #[test]
    fn shipped_engines_validate() {
        let e = crate::load_engines().expect("engines");
        assert_eq!(e.engines.len(), 6);
        assert_eq!(e.schema_version, 1);
        let google = e.engines.iter().find(|x| x.id == "google").unwrap();
        assert!(google.unofficial_gtx);
        assert!(!google.needs_key);
        assert!(e.engines.iter().filter(|x| x.id != "google").all(|x| x.needs_key));
    }
}
