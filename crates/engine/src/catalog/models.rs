use crate::engine::ProviderType;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderCategory {
    BuiltIn,
    TraditionalApi,
    LlmOfficial,
    Aggregator,
    LocalOrSelfHosted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkPolicy {
    LocalOnly,
    OfficialApi,
    UnofficialWeb,
    SelfHosted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stability {
    Stable,
    Experimental,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthScheme {
    None,
    Bearer,
    Anthropic,
    DeepLAuthKey,
    QueryKey,
    HeaderToken,
    Custom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogPlatform {
    Macos,
    Windows,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderFieldSpec {
    pub key: &'static str,
    pub label_key: &'static str,
    pub secret: bool,
    pub required: bool,
    pub placeholder: Option<&'static str>,
    pub advanced: bool,
    pub default_value: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderCapabilities {
    pub translation: bool,
    pub dictionary: bool,
    pub ocr: bool,
    pub llm: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderPreset {
    pub id: &'static str,
    pub engine_type: ProviderType,
    pub protocol: &'static str,
    pub category: ProviderCategory,
    pub name: &'static str,
    pub description_key: &'static str,
    pub homepage_url: Option<&'static str>,
    pub api_key_url: Option<&'static str>,
    pub base_url: &'static str,
    pub models_url: &'static str,
    pub fields: &'static [ProviderFieldSpec],
    pub capabilities: ProviderCapabilities,
    pub auth_scheme: AuthScheme,
    pub supported_platforms: &'static [CatalogPlatform],
    pub network_policy: NetworkPolicy,
    pub stability: Stability,
}

impl ProviderPreset {
    pub fn available_on(&self, macos: bool) -> bool {
        if self.supported_platforms.is_empty() {
            return true;
        }
        self.supported_platforms
            .iter()
            .any(|platform| match platform {
                CatalogPlatform::Macos => macos,
                CatalogPlatform::Windows => !macos,
            })
    }

    pub fn has_user_fields(&self) -> bool {
        !self.fields.is_empty()
    }
}

pub const TRANSLATION_ONLY: ProviderCapabilities = ProviderCapabilities {
    translation: true,
    dictionary: false,
    ocr: false,
    llm: false,
};

pub const LLM_TRANSLATION: ProviderCapabilities = ProviderCapabilities {
    translation: true,
    dictionary: false,
    ocr: false,
    llm: true,
};

pub const SYSTEM_CAPABILITIES: ProviderCapabilities = ProviderCapabilities {
    translation: true,
    dictionary: true,
    ocr: true,
    llm: false,
};

pub const BOTH_DESKTOPS: &[CatalogPlatform] = &[CatalogPlatform::Macos, CatalogPlatform::Windows];
pub const MACOS_ONLY: &[CatalogPlatform] = &[CatalogPlatform::Macos];

pub const fn field(
    key: &'static str,
    label_key: &'static str,
    secret: bool,
    required: bool,
    advanced: bool,
    default_value: Option<&'static str>,
) -> ProviderFieldSpec {
    ProviderFieldSpec {
        key,
        label_key,
        secret,
        required,
        placeholder: default_value,
        advanced,
        default_value,
    }
}
