use std::collections::{BTreeMap, HashMap};

use linguaray_engine::{ProviderConfig, ProviderType};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serializer};
use serde_json::Value;

use super::{ProviderConfigEntry, ServiceConfigEntry};

pub fn provider_config_from_settings(
    provider: &ProviderConfigEntry,
) -> Result<ProviderConfig, String> {
    let mut options = BTreeMap::new();
    for (key, value) in &provider.fields {
        if key != "presetId" {
            options.insert(key.clone(), serde_yaml::Value::String(value.clone()));
        }
    }
    Ok(ProviderConfig {
        provider_type: provider.r#type,
        options,
    })
}

pub(super) fn serialize_providers<S>(
    providers: &HashMap<String, ProviderConfigEntry>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(providers.len()))?;
    for (provider_id, provider) in providers {
        let config = provider_config_from_settings(provider).map_err(serde::ser::Error::custom)?;
        let mut value = provider_config_json_value(&config).map_err(serde::ser::Error::custom)?;
        if let Some(preset_id) = &provider.preset_id {
            if let Value::Object(object) = &mut value {
                object.insert("presetId".to_owned(), Value::String(preset_id.clone()));
            }
        }
        map.serialize_entry(provider_id, &value)?;
    }
    map.end()
}

pub(super) fn deserialize_providers<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, ProviderConfigEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    let providers = HashMap::<String, Value>::deserialize(deserializer)?;
    Ok(providers
        .into_iter()
        .filter_map(|(provider_id, value)| {
            provider_entry_from_value(&provider_id, value)
                .ok()
                .map(|entry| (provider_id, entry))
        })
        .collect())
}

fn provider_entry_from_value(
    provider_id: &str,
    mut value: Value,
) -> Result<ProviderConfigEntry, String> {
    let preset_id = value.as_object_mut().and_then(|object| {
        object
            .remove("presetId")
            .and_then(|value| value.as_str().map(str::to_owned))
    });
    let config = serde_json::from_value::<ProviderConfig>(value)
        .map_err(|error| format!("invalid provider config `{provider_id}`: {error}"))?;
    let mut entry = provider_entry_from_config(provider_id, &config)?;
    entry.preset_id = preset_id;
    Ok(entry)
}

pub(super) fn deserialize_services<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, ServiceConfigEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    let services = HashMap::<String, Value>::deserialize(deserializer)?;
    Ok(services
        .into_iter()
        .filter_map(|(service_id, value)| {
            serde_json::from_value::<ServiceConfigEntry>(value)
                .ok()
                .map(|entry| (service_id, entry))
        })
        .collect())
}

pub fn provider_entry_from_config(
    provider_id: &str,
    config: &ProviderConfig,
) -> Result<ProviderConfigEntry, String> {
    let value = provider_config_json_value(config)?;
    let Value::Object(mut object) = value else {
        return Err("provider config must encode to an object".to_owned());
    };
    object.remove("type");
    let fields = object
        .into_iter()
        .filter_map(|(key, value)| provider_config_field_value(value).map(|value| (key, value)))
        .collect();
    Ok(ProviderConfigEntry {
        id: provider_id.to_owned(),
        r#type: config.provider_type,
        fields,
        created_at: None,
        preset_id: None,
    })
}

pub(crate) fn parse_provider_type(value: &str) -> Result<ProviderType, String> {
    serde_yaml::from_value::<ProviderType>(serde_yaml::Value::String(value.to_owned()))
        .map_err(|error| format!("invalid provider type `{value}`: {error}"))
}

fn provider_config_field_value(value: Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Null => None,
        Value::Array(_) | Value::Object(_) => Some(value.to_string()),
    }
}

fn provider_config_json_value(config: &ProviderConfig) -> Result<Value, String> {
    let mut value = serde_json::to_value(config)
        .map_err(|error| format!("failed to encode provider: {error}"))?;
    normalize_provider_config_keys(&mut value);
    Ok(value)
}

fn normalize_provider_config_keys(value: &mut Value) {
    let Value::Object(object) = value else {
        return;
    };
    for (from, to) in [
        ("api_key", "apiKey"),
        ("app_key", "appKey"),
        ("app_id", "appId"),
        ("base_url", "baseUrl"),
        ("request_id", "requestId"),
        ("secret_id", "secretId"),
        ("secret_key", "secretKey"),
        ("app_secret", "appSecret"),
        ("picture_base_url", "pictureBaseUrl"),
    ] {
        if let Some(value) = object.remove(from) {
            object.insert(to.to_owned(), value);
        }
    }
}
