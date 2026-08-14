use linguaray_kernel::ServiceKey;
use serde::{Deserialize, Serialize};

/// Domain traits live here so the kernel stays generic. Production hookup is
/// PR-4+; K0 Go is required before any Fiber is activated from `lib.rs`.
pub trait TranslationService: Send + Sync {}
pub trait SecretsService: Send + Sync {}
pub trait DatabaseService: Send + Sync {}
pub trait HttpTransport: Send + Sync {}
pub trait EngineDriver: Send + Sync {}
pub trait EngineDriverRegistry: Send + Sync {}

pub static TRANSLATION: ServiceKey<dyn TranslationService> =
    ServiceKey::new("linguaray.translation");
pub static SECRETS: ServiceKey<dyn SecretsService> = ServiceKey::new("linguaray.secrets");
pub static DATABASE: ServiceKey<dyn DatabaseService> = ServiceKey::new("linguaray.database");
pub static HTTP: ServiceKey<dyn HttpTransport> = ServiceKey::new("linguaray.http");
pub static DRIVERS: ServiceKey<dyn EngineDriverRegistry> = ServiceKey::new("linguaray.drivers");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProtocolKind {
    OpenaiChat,
    Anthropic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthKind {
    #[default]
    Bearer,
    XApiKey,
    AzureKey,
    Query,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportTier {
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "setup_required")]
    SetupRequired,
    #[serde(rename = "unverified")]
    Unverified,
}

impl AuthKind {
    pub fn header_name(self) -> Option<&'static str> {
        match self {
            AuthKind::Bearer => Some("Authorization"),
            AuthKind::XApiKey => Some("x-api-key"),
            AuthKind::AzureKey => Some("api-key"),
            AuthKind::Query | AuthKind::None => None,
        }
    }
}

impl ProtocolKind {
    pub fn to_db(self) -> &'static str {
        match self {
            ProtocolKind::OpenaiChat => "openai_chat",
            ProtocolKind::Anthropic => "anthropic",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_kebab() {
        assert_eq!(
            serde_json::to_string(&AuthKind::AzureKey).unwrap(),
            "\"azure-key\""
        );
        assert_eq!(
            serde_json::to_string(&SupportTier::SetupRequired).unwrap(),
            "\"setup_required\""
        );
        assert_eq!(
            serde_json::from_str::<ProtocolKind>("\"openai-chat\"").unwrap(),
            ProtocolKind::OpenaiChat
        );
    }

    #[test]
    fn unknown_auth_rejected() {
        assert!(serde_json::from_str::<AuthKind>("\"magic\"").is_err());
    }

    #[test]
    fn service_keys_are_stable() {
        assert_eq!(TRANSLATION.id.0, "linguaray.translation");
        assert_eq!(SECRETS.id.0, "linguaray.secrets");
        assert_eq!(DATABASE.id.0, "linguaray.database");
        assert_eq!(HTTP.id.0, "linguaray.http");
        assert_eq!(DRIVERS.id.0, "linguaray.drivers");
    }
}
