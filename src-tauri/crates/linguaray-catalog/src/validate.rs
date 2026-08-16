use crate::{
    CatalogFile, CatalogProvider, EngineCatalogFile, RELAY_HOST_DENY, REQUIRED_ENGINE_IDS,
    REQUIRED_IDS,
};
use linguaray_contracts::{AuthKind, SupportTier};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("catalog json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(String),
}

pub fn validate_catalog(file: &CatalogFile) -> Result<(), CatalogError> {
    if file.schema_version != 1 {
        return Err(CatalogError::Invalid(format!(
            "schema_version must be 1, got {}",
            file.schema_version
        )));
    }
    if file.providers.len() != REQUIRED_IDS.len() {
        return Err(CatalogError::Invalid(format!(
            "expected {} providers, got {}",
            REQUIRED_IDS.len(),
            file.providers.len()
        )));
    }

    let mut seen = std::collections::BTreeSet::new();
    for p in &file.providers {
        validate_one(p)?;
        if !seen.insert(p.id.as_str()) {
            return Err(CatalogError::Invalid(format!("duplicate id {}", p.id)));
        }
    }
    for id in REQUIRED_IDS {
        if !seen.contains(id) {
            return Err(CatalogError::Invalid(format!("missing required id {id}")));
        }
    }
    for p in &file.providers {
        if p.support_tier == SupportTier::Ready
            && !matches!(p.id.as_str(), "openai" | "anthropic" | "gemini" | "ollama")
        {
            return Err(CatalogError::Invalid(format!(
                "{} cannot be ready without authenticated smoke promotion",
                p.id
            )));
        }
    }
    Ok(())
}

fn validate_one(p: &CatalogProvider) -> Result<(), CatalogError> {
    if p.id.is_empty() || p.id.contains('_') || p.id.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(CatalogError::Invalid(format!(
            "id {} must be kebab-case",
            p.id
        )));
    }
    if p.support_tier == SupportTier::Ready {
        if p.requires_user_endpoint {
            return Err(CatalogError::Invalid(format!(
                "ready row {} cannot require user endpoint",
                p.id
            )));
        }
        if p.endpoint.is_empty() || p.default_model.is_empty() {
            return Err(CatalogError::Invalid(format!(
                "ready row {} needs endpoint and default_model",
                p.id
            )));
        }
    }
    if !p.endpoint.is_empty() {
        validate_endpoint(&p.endpoint)?;
        if let Ok(u) = url::Url::parse(&p.endpoint) {
            let host = u.host_str().unwrap_or("");
            if RELAY_HOST_DENY.iter().any(|h| host == *h || host.ends_with(&format!(".{h}"))) {
                return Err(CatalogError::Invalid(format!(
                    "relay host {host} is not allowed in official catalog"
                )));
            }
        }
    } else if !p.requires_user_endpoint {
        return Err(CatalogError::Invalid(format!(
            "{} has empty endpoint but requires_user_endpoint is false",
            p.id
        )));
    }
    if !p.needs_key && p.auth != AuthKind::None && !endpoint_is_loopback(&p.endpoint) {
        return Err(CatalogError::Invalid(format!(
            "{} is keyless but is not loopback / auth=none",
            p.id
        )));
    }
    Ok(())
}

pub fn validate_engines(file: &EngineCatalogFile) -> Result<(), CatalogError> {
    if file.schema_version != 1 {
        return Err(CatalogError::Invalid(format!(
            "engines schema_version must be 1, got {}",
            file.schema_version
        )));
    }
    if file.engines.len() != REQUIRED_ENGINE_IDS.len() {
        return Err(CatalogError::Invalid(format!(
            "expected {} engines, got {}",
            REQUIRED_ENGINE_IDS.len(),
            file.engines.len()
        )));
    }
    let mut seen = std::collections::BTreeSet::new();
    for e in &file.engines {
        if e.id.is_empty() || e.id.contains('_') || e.id.chars().any(|c| c.is_ascii_uppercase()) {
            return Err(CatalogError::Invalid(format!(
                "engine id {} must be kebab-case",
                e.id
            )));
        }
        if e.endpoint.is_empty() {
            return Err(CatalogError::Invalid(format!(
                "engine {} needs an endpoint",
                e.id
            )));
        }
        validate_endpoint(&e.endpoint)?;
        if !seen.insert(e.id.as_str()) {
            return Err(CatalogError::Invalid(format!("duplicate engine id {}", e.id)));
        }
    }
    for id in REQUIRED_ENGINE_IDS {
        if !seen.contains(id) {
            return Err(CatalogError::Invalid(format!("missing required engine {id}")));
        }
    }
    Ok(())
}

pub fn validate_endpoint(endpoint: &str) -> Result<(), CatalogError> {
    let parsed = url::Url::parse(endpoint)
        .map_err(|e| CatalogError::Invalid(format!("bad url: {e}")))?;
    match parsed.scheme() {
        "https" => Ok(()),
        "http" => {
            let h = parsed.host_str().unwrap_or("");
            if h == "localhost" || h == "127.0.0.1" || h == "::1" {
                Ok(())
            } else {
                Err(CatalogError::Invalid(format!(
                    "http only allowed for loopback, got {h}"
                )))
            }
        }
        s => Err(CatalogError::Invalid(format!("scheme {s} not allowed"))),
    }
}

fn endpoint_is_loopback(endpoint: &str) -> bool {
    url::Url::parse(endpoint)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .is_some_and(|h| h == "localhost" || h == "127.0.0.1" || h == "::1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CatalogFile, CatalogProvider};
    use linguaray_contracts::{AuthKind, ProtocolKind, SupportTier};

    fn row(id: &str, tier: SupportTier, endpoint: &str, model: &str) -> CatalogProvider {
        CatalogProvider {
            id: id.into(),
            label: id.into(),
            protocol: ProtocolKind::OpenaiChat,
            auth: AuthKind::Bearer,
            endpoint: endpoint.into(),
            default_model: model.into(),
            needs_key: true,
            support_tier: tier,
            requires_user_endpoint: endpoint.is_empty(),
            models_url: None,
            website: None,
            console_url: None,
            docs: None,
            notes: None,
            tags: vec![],
            icon: None,
        }
    }

    #[test]
    fn ready_cannot_be_empty_endpoint() {
        let file = CatalogFile {
            schema_version: 1,
            catalog_revision: 1,
            providers: vec![row("openai", SupportTier::Ready, "", "m")],
        };
        assert!(validate_catalog(&file).is_err());
    }

    #[test]
    fn unverified_cannot_be_silently_ready() {
        let file = CatalogFile {
            schema_version: 1,
            catalog_revision: 1,
            providers: REQUIRED_IDS
                .iter()
                .map(|id| {
                    let mut r = row(
                        id,
                        if matches!(
                            *id,
                            "deepseek" | "openai" | "anthropic" | "gemini" | "ollama"
                        ) {
                            SupportTier::Ready
                        } else if matches!(*id, "azure-openai" | "custom" | "doubao") {
                            SupportTier::SetupRequired
                        } else {
                            SupportTier::Unverified
                        },
                        if matches!(*id, "azure-openai" | "custom") {
                            ""
                        } else {
                            "https://api.example.com/v1/chat/completions"
                        },
                        "m",
                    );
                    if *id == "ollama" {
                        r.endpoint = "http://localhost:11434/v1/chat/completions".into();
                        r.needs_key = false;
                        r.auth = AuthKind::None;
                    }
                    r
                })
                .collect(),
        };
        let err = validate_catalog(&file).unwrap_err().to_string();
        assert!(err.contains("deepseek"), "{err}");
    }

    #[test]
    fn relay_host_rejected() {
        let err = validate_endpoint("https://packycode.com/v1/chat/completions");
        // endpoint scheme is fine; catalog-level deny is in validate_one
        assert!(err.is_ok());
        let p = row(
            "evil",
            SupportTier::Unverified,
            "https://api.packycode.com/v1/chat/completions",
            "m",
        );
        let err = validate_one(&p).unwrap_err().to_string();
        assert!(err.contains("relay"), "{err}");
    }

    #[test]
    fn http_zero_zero_not_loopback() {
        assert!(validate_endpoint("http://0.0.0.0:11434/v1").is_err());
    }
}
