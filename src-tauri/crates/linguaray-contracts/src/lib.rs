use serde::{Deserialize, Serialize};

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
}
