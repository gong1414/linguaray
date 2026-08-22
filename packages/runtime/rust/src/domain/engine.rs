use std::collections::{BTreeMap, HashMap};

use linguaray_engine::{Engine, EngineConfig, ProviderConfig};

use crate::domain::settings::{provider_config_from_settings, Settings};

pub fn build_from_settings(settings: &Settings) -> Result<Engine, String> {
    build_from_settings_with_secrets(settings, &HashMap::new())
}

/// Builds the in-memory engine from persisted settings plus credentials that
/// were loaded from the platform secure store. The secret map is never copied
/// back into [`Settings`] and therefore can never reach `settings.json`.
pub fn build_from_settings_with_secrets(
    settings: &Settings,
    provider_secrets: &HashMap<String, HashMap<String, String>>,
) -> Result<Engine, String> {
    let mut providers = BTreeMap::new();
    for (provider_id, provider) in &settings.providers {
        let provider_id = provider_id.trim();
        if provider_id.is_empty() {
            return Err("provider id is required".to_owned());
        }

        let mut hydrated = provider.clone();
        if let Some(secrets) = provider_secrets.get(provider_id) {
            hydrated.fields.extend(secrets.clone());
        }
        providers.insert(
            provider_id.to_owned(),
            provider_config_from_settings(&hydrated)?,
        );
    }

    build_from_engine_config(EngineConfig { providers })
}

pub fn build_from_engine_config(config: EngineConfig) -> Result<Engine, String> {
    linguaray_engine::from_config(config).map_err(|error| error.to_string())
}

pub fn build_from_provider_config(
    provider_id: &str,
    provider_config_text: &str,
) -> Result<Engine, String> {
    let provider_id = provider_id.trim();
    if provider_id.is_empty() {
        return Err("provider_id is required".to_owned());
    }

    let provider_config = parse_provider_config(provider_config_text)?;
    let mut providers = BTreeMap::new();
    providers.insert(provider_id.to_owned(), provider_config);

    build_from_engine_config(EngineConfig { providers })
}

pub fn parse_provider_config(input: &str) -> Result<ProviderConfig, String> {
    serde_yaml::from_str::<ProviderConfig>(input)
        .map_err(|error| format!("invalid provider config: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_provider_config_requires_type_tag() {
        let error = parse_provider_config("api_key: test-key").unwrap_err();
        assert!(error.contains("type"));
    }

    #[test]
    fn build_from_provider_config_registers_provider() {
        let engine =
            build_from_provider_config("deepl-main", "type: deepl\napi_key: test-key").unwrap();
        assert_eq!(engine.names(), vec!["deepl-main"]);
    }

    #[test]
    fn direct_construction_matches_the_yaml_path() {
        let config_text = "deepl-main:\n  type: deepl\n  api_key: test-key";
        let parsed: EngineConfig = serde_yaml::from_str(config_text).unwrap();
        let direct = build_from_engine_config(parsed).unwrap();
        let via_yaml = linguaray_engine::from_yaml_str(config_text).unwrap();
        assert_eq!(direct.names(), via_yaml.names());
        let direct_error = direct.translation("deepl-main").err();
        let yaml_error = via_yaml.translation("deepl-main").err();
        // Both paths must agree, whether or not the service is available.
        assert_eq!(direct_error.is_some(), yaml_error.is_some());
        assert_eq!(
            direct_error.map(|e| e.to_string()),
            yaml_error.map(|e| e.to_string())
        );
    }
}
