//! Built-in provider catalog. Rust is the only runtime source of provider
//! presets, field specs, and the slim models.dev snapshot.

mod discovery;
mod models;
mod presets;
mod seed;
mod snapshot;
pub mod urls;

pub use discovery::{
    fetch_models_with_candidates, interpret_models_response, parse_openai_models_body,
    CandidateOutcome, MODELS_FETCH_TIMEOUT,
};
pub use models::{
    field, AuthScheme, CatalogPlatform, NetworkPolicy, ProviderCapabilities, ProviderCategory,
    ProviderFieldSpec, ProviderPreset, Stability, BOTH_DESKTOPS, LLM_TRANSLATION, MACOS_ONLY,
    SYSTEM_CAPABILITIES, TRANSLATION_ONLY,
};
pub use presets::{preset_by_id, presets_for_platform, PRESETS};
pub use seed::{
    apply_full_seed, default_seed, CatalogSeed, SeedProvider, SeedService, CATALOG_SEED_REVISION,
};
pub use snapshot::{models_for_preset, snapshot_models, CatalogSnapshotModel};

use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize)]
pub struct CatalogJson {
    pub presets: Vec<CatalogJsonPreset>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CatalogJsonPreset {
    pub id: String,
    pub engine_type: String,
    pub protocol: String,
    pub category: String,
    pub name: String,
    pub description_key: String,
    pub homepage_url: Option<String>,
    pub api_key_url: Option<String>,
    pub base_url: String,
    pub models_url: String,
    pub fields: Vec<CatalogJsonField>,
    pub capabilities: CatalogJsonCapabilities,
    pub auth_scheme: String,
    pub supported_platforms: Vec<String>,
    pub network_policy: String,
    pub stability: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CatalogJsonField {
    pub key: String,
    pub label_key: String,
    pub secret: bool,
    pub required: bool,
    pub placeholder: Option<String>,
    pub advanced: bool,
    pub default_value: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CatalogJsonCapabilities {
    pub translation: bool,
    pub dictionary: bool,
    pub ocr: bool,
    pub llm: bool,
}

impl ProviderPreset {
    pub fn effective_models_url(&self) -> String {
        if !self.models_url.is_empty() {
            return self.models_url.to_owned();
        }
        if self.capabilities.llm && !self.base_url.is_empty() {
            return urls::openai_models_url(self.base_url);
        }
        String::new()
    }

    pub fn to_json(&self) -> CatalogJsonPreset {
        CatalogJsonPreset {
            id: self.id.to_owned(),
            engine_type: self.engine_type.as_str().to_owned(),
            protocol: self.protocol.to_owned(),
            category: match self.category {
                ProviderCategory::BuiltIn => "builtIn".to_owned(),
                ProviderCategory::TraditionalApi => "traditionalApi".to_owned(),
                ProviderCategory::LlmOfficial => "llmOfficial".to_owned(),
                ProviderCategory::Aggregator => "aggregator".to_owned(),
                ProviderCategory::LocalOrSelfHosted => "localOrSelfHosted".to_owned(),
            },
            name: self.name.to_owned(),
            description_key: self.description_key.to_owned(),
            homepage_url: self.homepage_url.map(str::to_owned),
            api_key_url: self.api_key_url.map(str::to_owned),
            base_url: self.base_url.to_owned(),
            models_url: self.effective_models_url(),
            fields: self
                .fields
                .iter()
                .map(|field| CatalogJsonField {
                    key: field.key.to_owned(),
                    label_key: field.label_key.to_owned(),
                    secret: field.secret,
                    required: field.required,
                    placeholder: field.placeholder.map(str::to_owned),
                    advanced: field.advanced,
                    default_value: field.default_value.map(str::to_owned),
                })
                .collect(),
            capabilities: CatalogJsonCapabilities {
                translation: self.capabilities.translation,
                dictionary: self.capabilities.dictionary,
                ocr: self.capabilities.ocr,
                llm: self.capabilities.llm,
            },
            auth_scheme: format!("{:?}", self.auth_scheme),
            supported_platforms: self
                .supported_platforms
                .iter()
                .map(|platform| match platform {
                    CatalogPlatform::Macos => "macos".to_owned(),
                    CatalogPlatform::Windows => "windows".to_owned(),
                })
                .collect(),
            network_policy: match self.network_policy {
                NetworkPolicy::LocalOnly => "localOnly".to_owned(),
                NetworkPolicy::OfficialApi => "officialApi".to_owned(),
                NetworkPolicy::UnofficialWeb => "unofficialWeb".to_owned(),
                NetworkPolicy::SelfHosted => "selfHosted".to_owned(),
            },
            stability: match self.stability {
                Stability::Stable => "stable".to_owned(),
                Stability::Experimental => "experimental".to_owned(),
            },
        }
    }
}

pub fn catalog_json() -> CatalogJson {
    CatalogJson {
        presets: PRESETS.iter().map(ProviderPreset::to_json).collect(),
    }
}

pub fn is_allowed_catalog_url(url: &str) -> bool {
    let url = url.trim();
    if url.is_empty() {
        return true;
    }
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return false;
    };
    match parsed.scheme() {
        "https" => true,
        "http" => {
            matches!(parsed.host_str(), Some("127.0.0.1" | "localhost" | "::1"))
        }
        _ => false,
    }
}

pub fn secret_field_keys() -> BTreeSet<&'static str> {
    PRESETS
        .iter()
        .flat_map(|preset| preset.fields.iter())
        .filter(|field| field.secret)
        .map(|field| field.key)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn preset_ids_are_unique_and_ordered() {
        let ids: Vec<_> = PRESETS.iter().map(|preset| preset.id).collect();
        let unique: HashSet<_> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len());
        assert_eq!(
            ids,
            vec![
                "system",
                "ecdict",
                "google-web",
                "bing-web",
                "tencent-transmart-web",
                "deepl-free",
                "deepl-pro",
                "google-cloud-translation",
                "baidu-translate",
                "tencent-cloud-tmt",
                "youdao-zhiyun",
                "caiyun",
                "openai",
                "anthropic",
                "gemini",
                "deepseek",
                "bailian-qwen",
                "zhipu-bigmodel",
                "moonshot-kimi",
                "doubao-ark",
                "xai",
                "groq",
                "openrouter",
                "siliconflow-cn",
                "siliconflow-global",
                "modelscope",
                "ollama",
                "lm-studio",
                "localai",
                "vllm",
                "llama-cpp",
                "litellm",
                "libretranslate",
                "mtranserver",
            ]
        );
    }

    #[test]
    fn catalog_json_round_trips_through_serde() {
        let json = serde_json::to_value(catalog_json()).expect("serialize catalog");
        let presets = json
            .get("presets")
            .and_then(|value| value.as_array())
            .expect("presets array");
        assert_eq!(presets.len(), PRESETS.len());
        assert_eq!(presets[0]["id"], "system");
        assert_eq!(presets[1]["network_policy"], "localOnly");
        assert_eq!(presets[1]["stability"], "stable");
        assert_eq!(presets[2]["network_policy"], "unofficialWeb");
        assert_eq!(presets[2]["stability"], "experimental");
        let parsed: serde_json::Value = serde_json::from_str(&json.to_string()).expect("parse");
        assert_eq!(parsed["presets"].as_array().unwrap().len(), PRESETS.len());
    }

    #[test]
    fn catalog_urls_are_http_or_https() {
        for preset in PRESETS {
            for url in [
                preset.base_url,
                preset.models_url,
                preset.homepage_url.unwrap_or(""),
                preset.api_key_url.unwrap_or(""),
            ] {
                assert!(
                    is_allowed_catalog_url(url),
                    "invalid catalog url {} on {}",
                    url,
                    preset.id
                );
            }
        }
    }

    #[test]
    fn secret_fields_are_marked_and_known() {
        let keys = secret_field_keys();
        for expected in [
            "apiKey",
            "authKey",
            "appKey",
            "appSecret",
            "secretId",
            "secretKey",
            "token",
        ] {
            assert!(keys.contains(expected), "missing secret key {expected}");
        }
        for preset in PRESETS {
            for field in preset.fields {
                if matches!(
                    field.key,
                    "apiKey" | "authKey" | "appSecret" | "secretKey" | "secretId"
                ) {
                    assert!(
                        field.secret,
                        "{} must be secret on {}",
                        field.key, preset.id
                    );
                }
            }
        }
    }

    #[test]
    fn caiyun_includes_request_id() {
        let caiyun = preset_by_id("caiyun").expect("caiyun");
        assert!(caiyun.fields.iter().any(|field| field.key == "requestId"));
        assert!(caiyun
            .fields
            .iter()
            .any(|field| field.key == "token" && field.secret));
    }

    #[test]
    fn unofficial_web_presets_are_experimental() {
        for id in ["google-web", "bing-web", "tencent-transmart-web"] {
            let preset = preset_by_id(id).expect(id);
            assert_eq!(preset.network_policy, NetworkPolicy::UnofficialWeb);
            assert_eq!(preset.stability, Stability::Experimental);
        }
    }

    #[test]
    fn system_is_available_on_both_supported_desktops() {
        let system = preset_by_id("system").unwrap();
        assert!(system.available_on(true));
        assert!(system.available_on(false));
    }
}
