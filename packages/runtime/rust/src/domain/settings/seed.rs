use std::collections::HashMap;

use super::{ProviderConfigEntry, ServiceConfigEntry, ServiceType, Settings};

pub fn apply_catalog_seed(settings: &mut Settings) -> bool {
    apply_catalog_seed_for(settings, cfg!(target_os = "macos"))
}

pub fn apply_catalog_seed_for(settings: &mut Settings, macos: bool) -> bool {
    if settings.catalog_seed_revision >= linguaray_engine::catalog::CATALOG_SEED_REVISION {
        return false;
    }

    let legacy_system_only = !settings.providers.is_empty()
        && settings.providers.keys().all(|id| id == "system")
        && settings
            .services
            .values()
            .all(|service| service.provider_id == "system");
    if legacy_system_only && !macos {
        settings.providers.remove("system");
        settings
            .services
            .retain(|_, service| service.provider_id != "system");
    }
    if (settings.providers.is_empty() && settings.services.is_empty()) || legacy_system_only {
        let seed = linguaray_engine::catalog::default_seed(macos);
        for provider in seed.providers {
            settings
                .providers
                .entry(provider.id.clone())
                .or_insert_with(|| ProviderConfigEntry {
                    id: provider.id,
                    r#type: provider.provider_type,
                    fields: HashMap::new(),
                    created_at: None,
                    preset_id: Some(provider.preset_id),
                });
        }
        for service in seed.services {
            let enabled = if service.enabled { "true" } else { "false" };
            settings
                .services
                .entry(service.id.clone())
                .and_modify(|entry| {
                    entry
                        .fields
                        .insert("enabled".to_owned(), enabled.to_owned());
                })
                .or_insert_with(|| ServiceConfigEntry {
                    id: service.id.clone(),
                    provider_id: service.provider_id,
                    r#type: ServiceType::Translation,
                    name: service.id,
                    fields: HashMap::from([("enabled".to_owned(), enabled.to_owned())]),
                    created_at: None,
                });
        }
        settings.general.default_translation_service = seed.default_translation_service;
        settings.general.default_directory_service = seed.default_dictionary_service;
        settings.general.default_ocr_service = seed.default_ocr_service;
        settings.general.translation_service_order = seed.translation_service_order;
    }

    if settings.catalog_seed_revision < 2 {
        let preset =
            linguaray_engine::catalog::preset_by_id("ecdict").expect("ecdict catalog preset");
        settings
            .providers
            .entry("ecdict".to_owned())
            .or_insert_with(|| ProviderConfigEntry {
                id: "ecdict".to_owned(),
                r#type: preset.engine_type,
                fields: HashMap::new(),
                created_at: None,
                preset_id: Some(preset.id.to_owned()),
            });
        if settings.general.default_directory_service.is_empty() {
            settings.general.default_directory_service = "ecdict+dictionary".to_owned();
        }
    }

    if settings.catalog_seed_revision < 3 {
        if !macos {
            let preset =
                linguaray_engine::catalog::preset_by_id("system").expect("system catalog preset");
            settings
                .providers
                .entry("system".to_owned())
                .or_insert_with(|| ProviderConfigEntry {
                    id: "system".to_owned(),
                    r#type: preset.engine_type,
                    fields: HashMap::new(),
                    created_at: None,
                    preset_id: Some(preset.id.to_owned()),
                });
        }
        if settings.providers.contains_key("system")
            && settings.general.default_ocr_service.is_empty()
        {
            settings.general.default_ocr_service = "system+ocr".to_owned();
        }
    }

    settings.catalog_seed_revision = linguaray_engine::catalog::CATALOG_SEED_REVISION;
    true
}
