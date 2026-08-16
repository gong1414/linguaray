use linguaray_kernel::ServiceKey;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Domain traits live here so the kernel stays generic. Production hookup is
/// PR-4+; K0 Go is required before any Fiber is activated from `lib.rs`.
pub trait TranslationService: Send + Sync {}
pub trait SecretsService: Send + Sync {}
pub trait DatabaseService: Send + Sync {}
pub trait HttpTransport: Send + Sync {}
pub trait ProviderService: Send + Sync {}
pub trait SelectionService: Send + Sync {}
pub trait ClipboardService: Send + Sync {}
pub trait PopupService: Send + Sync {}
pub trait TrayService: Send + Sync {}
pub trait HistoryService: Send + Sync {}

/// One HTTP dialect. Implementations live in the host; this crate stays
/// reqwest-free (`HttpRequestPlan` is headers + JSON).
pub trait EngineDriver: Send + Sync {
    fn id(&self) -> &'static str;
    fn protocol(&self) -> ProtocolKind;
    fn build_request(&self, input: &DriverInput<'_>) -> Result<HttpRequestPlan, DriverError>;
    fn parse_response(&self, body: &serde_json::Value) -> Result<String, DriverError>;
}

pub trait EngineDriverRegistry: Send + Sync {
    fn get(&self, protocol: ProtocolKind) -> Option<Arc<dyn EngineDriver>>;
}

pub static TRANSLATION: ServiceKey<dyn TranslationService> =
    ServiceKey::new("linguaray.translation");
pub static SECRETS: ServiceKey<dyn SecretsService> = ServiceKey::new("linguaray.secrets");
pub static DATABASE: ServiceKey<dyn DatabaseService> = ServiceKey::new("linguaray.database");
pub static HTTP: ServiceKey<dyn HttpTransport> = ServiceKey::new("linguaray.http");
pub static DRIVERS: ServiceKey<dyn EngineDriverRegistry> = ServiceKey::new("linguaray.drivers");
pub static PROVIDERS: ServiceKey<dyn ProviderService> = ServiceKey::new("linguaray.providers");
pub static SELECTION: ServiceKey<dyn SelectionService> = ServiceKey::new("linguaray.selection");
pub static CLIPBOARD: ServiceKey<dyn ClipboardService> = ServiceKey::new("linguaray.clipboard");
pub static POPUP: ServiceKey<dyn PopupService> = ServiceKey::new("linguaray.popup");
pub static TRAY: ServiceKey<dyn TrayService> = ServiceKey::new("linguaray.tray");
pub static HISTORY: ServiceKey<dyn HistoryService> = ServiceKey::new("linguaray.history");

/// Snapshot the openai-chat / anthropic Driver reads. `key` is borrowed for
/// the shortest window; the Driver must not store it.
#[derive(Debug, Clone)]
pub struct DriverInput<'a> {
    pub endpoint: &'a str,
    pub model: &'a str,
    pub auth: AuthKind,
    pub key: &'a str,
    pub system: &'a str,
    pub user: &'a str,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

#[derive(Debug, Clone)]
pub struct HttpRequestPlan {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
    pub body: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverError(pub String);

impl std::fmt::Display for DriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DriverError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "kebab-case")]
pub enum ProtocolKind {
    OpenaiChat,
    Anthropic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "kebab-case")]
pub enum AuthKind {
    #[default]
    Bearer,
    XApiKey,
    AzureKey,
    Query,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
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

    /// Header pairs for a chat/models request. `Query` / `None` yield none;
    /// callers that need `?key=` use [`query_pairs`].
    pub fn http_headers(self, key: &str) -> Vec<(String, String)> {
        match self {
            AuthKind::Bearer => vec![("Authorization".into(), format!("Bearer {key}"))],
            AuthKind::XApiKey => vec![("x-api-key".into(), key.into())],
            AuthKind::AzureKey => vec![("api-key".into(), key.into())],
            AuthKind::Query | AuthKind::None => vec![],
        }
    }

    pub fn query_pairs(self, key: &str) -> Vec<(String, String)> {
        match self {
            AuthKind::Query => vec![("key".into(), key.into())],
            _ => vec![],
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
        assert_eq!(PROVIDERS.id.0, "linguaray.providers");
        assert_eq!(SELECTION.id.0, "linguaray.selection");
        assert_eq!(CLIPBOARD.id.0, "linguaray.clipboard");
        assert_eq!(POPUP.id.0, "linguaray.popup");
        assert_eq!(TRAY.id.0, "linguaray.tray");
        assert_eq!(HISTORY.id.0, "linguaray.history");
    }

    #[test]
    fn auth_headers_do_not_invent_authorization_for_azure_key() {
        let headers = AuthKind::AzureKey.http_headers("sk-az");
        assert_eq!(headers, vec![("api-key".into(), "sk-az".into())]);
        assert!(AuthKind::None.http_headers("x").is_empty());
        assert_eq!(
            AuthKind::Query.query_pairs("q"),
            vec![("key".into(), "q".into())]
        );
    }
}
