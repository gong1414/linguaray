use crate::engine::ProviderType;

use super::presets::preset_by_id;

pub const CATALOG_SEED_REVISION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeedProvider {
    pub id: String,
    pub provider_type: ProviderType,
    pub preset_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeedService {
    pub id: String,
    pub provider_id: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogSeed {
    pub revision: u32,
    pub providers: Vec<SeedProvider>,
    pub services: Vec<SeedService>,
    pub default_translation_service: String,
    pub translation_service_order: Vec<String>,
}

/// Platform defaults applied only when settings have no providers and no
/// services. Bing Web and Tencent Transmart stay catalog-only.
pub fn default_seed(macos: bool) -> CatalogSeed {
    let mut providers = Vec::new();
    let mut services = Vec::new();
    let mut order = Vec::new();

    if macos {
        let system = preset_by_id("system").expect("system preset");
        providers.push(SeedProvider {
            id: "system".to_owned(),
            provider_type: system.engine_type,
            preset_id: system.id.to_owned(),
        });
        services.push(SeedService {
            id: "system+translation".to_owned(),
            provider_id: "system".to_owned(),
            enabled: false,
        });
    }

    let google_web = preset_by_id("google-web").expect("google-web preset");
    providers.push(SeedProvider {
        id: "google-web".to_owned(),
        provider_type: google_web.engine_type,
        preset_id: google_web.id.to_owned(),
    });
    services.push(SeedService {
        id: "google-web+translation".to_owned(),
        provider_id: "google-web".to_owned(),
        enabled: true,
    });
    order.push("google-web+translation".to_owned());
    if macos {
        order.push("system+translation".to_owned());
    }

    let default_translation_service = "google-web+translation".to_owned();

    CatalogSeed {
        revision: CATALOG_SEED_REVISION,
        providers,
        services,
        default_translation_service,
        translation_service_order: order,
    }
}

pub fn apply_full_seed(macos: bool) -> CatalogSeed {
    default_seed(macos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_seed_uses_google_web_and_keeps_system_available() {
        let seed = default_seed(true);
        assert_eq!(seed.revision, 1);
        assert_eq!(
            seed.providers
                .iter()
                .map(|provider| provider.id.as_str())
                .collect::<Vec<_>>(),
            vec!["system", "google-web"]
        );
        assert_eq!(seed.default_translation_service, "google-web+translation");
        let google = seed
            .services
            .iter()
            .find(|service| service.id == "google-web+translation")
            .unwrap();
        assert!(google.enabled);
        let system = seed
            .services
            .iter()
            .find(|service| service.id == "system+translation")
            .unwrap();
        assert!(!system.enabled);
        assert_eq!(
            seed.translation_service_order,
            vec![
                "google-web+translation".to_owned(),
                "system+translation".to_owned()
            ]
        );
        assert!(!seed
            .providers
            .iter()
            .any(|provider| provider.id == "bing-web"));
        assert!(!seed
            .providers
            .iter()
            .any(|provider| provider.id == "tencent-transmart-web"));
    }

    #[test]
    fn windows_seed_uses_google_web_and_skips_system() {
        let seed = default_seed(false);
        assert!(!seed
            .providers
            .iter()
            .any(|provider| provider.id == "system"));
        assert_eq!(seed.default_translation_service, "google-web+translation");
        let google = seed
            .services
            .iter()
            .find(|service| service.id == "google-web+translation")
            .unwrap();
        assert!(google.enabled);
        assert_eq!(
            seed.translation_service_order,
            vec!["google-web+translation".to_owned()]
        );
    }
}
