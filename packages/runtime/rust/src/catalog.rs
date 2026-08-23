//! UniFFI view of the engine provider catalog.

use linguaray_engine::catalog::{
    models_for_preset, presets_for_platform, CatalogSnapshotModel, NetworkPolicy as EngineNetwork,
    ProviderCategory as EngineCategory, ProviderPreset, Stability as EngineStability,
};

#[derive(Clone, Debug, uniffi::Enum)]
pub enum CatalogCategory {
    BuiltIn,
    TraditionalApi,
    LlmOfficial,
    Aggregator,
    LocalOrSelfHosted,
}

#[derive(Clone, Debug, uniffi::Enum)]
pub enum CatalogNetworkPolicy {
    LocalOnly,
    OfficialApi,
    UnofficialWeb,
    SelfHosted,
}

#[derive(Clone, Debug, uniffi::Enum)]
pub enum CatalogStability {
    Stable,
    Experimental,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct CatalogFieldSpec {
    pub key: String,
    pub label_key: String,
    pub secret: bool,
    pub required: bool,
    pub placeholder: Option<String>,
    pub advanced: bool,
    pub default_value: Option<String>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct CatalogProviderPreset {
    pub id: String,
    pub engine_type: String,
    pub protocol: String,
    pub category: CatalogCategory,
    pub name: String,
    pub description_key: String,
    pub homepage_url: Option<String>,
    pub api_key_url: Option<String>,
    pub base_url: String,
    pub models_url: String,
    pub fields: Vec<CatalogFieldSpec>,
    pub translation: bool,
    pub dictionary: bool,
    pub ocr: bool,
    pub llm: bool,
    pub supported_macos: bool,
    pub supported_windows: bool,
    pub network_policy: CatalogNetworkPolicy,
    pub stability: CatalogStability,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct CatalogModelChoice {
    pub id: String,
    pub name: String,
}

fn map_preset(preset: &ProviderPreset, macos: bool) -> CatalogProviderPreset {
    let system_capability_available = preset.id != "system" || macos;
    CatalogProviderPreset {
        id: preset.id.to_owned(),
        engine_type: preset.engine_type.as_str().to_owned(),
        protocol: preset.protocol.to_owned(),
        category: match preset.category {
            EngineCategory::BuiltIn => CatalogCategory::BuiltIn,
            EngineCategory::TraditionalApi => CatalogCategory::TraditionalApi,
            EngineCategory::LlmOfficial => CatalogCategory::LlmOfficial,
            EngineCategory::Aggregator => CatalogCategory::Aggregator,
            EngineCategory::LocalOrSelfHosted => CatalogCategory::LocalOrSelfHosted,
        },
        name: preset.name.to_owned(),
        description_key: preset.description_key.to_owned(),
        homepage_url: preset.homepage_url.map(str::to_owned),
        api_key_url: preset.api_key_url.map(str::to_owned),
        base_url: preset.base_url.to_owned(),
        models_url: preset.effective_models_url(),
        fields: preset
            .fields
            .iter()
            .map(|field| CatalogFieldSpec {
                key: field.key.to_owned(),
                label_key: field.label_key.to_owned(),
                secret: field.secret,
                required: field.required,
                placeholder: field.placeholder.map(str::to_owned),
                advanced: field.advanced,
                default_value: field.default_value.map(str::to_owned),
            })
            .collect(),
        translation: preset.capabilities.translation && system_capability_available,
        dictionary: preset.capabilities.dictionary && system_capability_available,
        ocr: preset.capabilities.ocr,
        llm: preset.capabilities.llm,
        supported_macos: preset.available_on(true),
        supported_windows: preset.available_on(false),
        network_policy: match preset.network_policy {
            EngineNetwork::LocalOnly => CatalogNetworkPolicy::LocalOnly,
            EngineNetwork::OfficialApi => CatalogNetworkPolicy::OfficialApi,
            EngineNetwork::UnofficialWeb => CatalogNetworkPolicy::UnofficialWeb,
            EngineNetwork::SelfHosted => CatalogNetworkPolicy::SelfHosted,
        },
        stability: match preset.stability {
            EngineStability::Stable => CatalogStability::Stable,
            EngineStability::Experimental => CatalogStability::Experimental,
        },
    }
}

fn map_model(model: CatalogSnapshotModel) -> CatalogModelChoice {
    CatalogModelChoice {
        id: model.id,
        name: model.name,
    }
}

#[uniffi::export]
pub fn list_provider_catalog() -> Vec<CatalogProviderPreset> {
    let macos = cfg!(target_os = "macos");
    presets_for_platform(macos)
        .into_iter()
        .map(|preset| map_preset(preset, macos))
        .collect()
}

#[uniffi::export]
pub fn list_catalog_snapshot_models(preset_id: String) -> Vec<CatalogModelChoice> {
    models_for_preset(&preset_id)
        .into_iter()
        .map(map_model)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use linguaray_engine::catalog::preset_by_id;

    #[test]
    fn system_catalog_exposes_only_ocr_on_windows() {
        let system = preset_by_id("system").expect("system preset");
        let windows = map_preset(system, false);
        assert!(!windows.translation);
        assert!(!windows.dictionary);
        assert!(windows.ocr);
        assert!(windows.supported_windows);

        let macos = map_preset(system, true);
        assert!(macos.translation);
        assert!(macos.dictionary);
        assert!(macos.ocr);
        assert!(macos.supported_macos);
    }
}
