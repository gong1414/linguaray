pub mod catalog;
mod common;
mod engine;
mod languages;
mod provider;

pub use engine::{
    from_config, from_yaml_str, load_from_file, Engine, EngineConfig, EngineError, ProviderConfig,
    ProviderType,
};
pub use languages::{all_languages, app_languages};
pub use common::{
    configure_network_proxy, current_network_proxy, NetworkProxyConfig, NetworkProxyMode,
};
pub use linguaray_core::{
    DictionaryError, DictionaryService, OcrError, OcrService, Provider, TranslationError,
    TranslationService,
};

// LLM providers (primary).
pub use provider::{
    prompt, specs, OpenAiCompatibleProvider, OpenAiCompatibleProviderConfig, OpenAiCompatibleSpec,
    OpenAiProviderConfig, XAiProviderConfig,
};

// Traditional providers.
#[cfg(feature = "baidu")]
pub use provider::traditional::BaiduProvider;
#[cfg(feature = "caiyun")]
pub use provider::traditional::CaiyunProvider;
pub use provider::traditional::DeepLProvider;
#[cfg(feature = "google")]
pub use provider::traditional::GoogleProvider;
pub use provider::traditional::SystemProvider;
pub use provider::traditional::SystemTranslationService;
#[cfg(feature = "tencent")]
pub use provider::traditional::TencentProvider;
#[cfg(feature = "youdao")]
pub use provider::traditional::YoudaoProvider;
pub use provider::traditional::{
    BaiduProviderConfig, BingWebProviderConfig, CaiyunProviderConfig, DeepLProviderConfig,
    GoogleProviderConfig, GoogleWebProviderConfig, LibreTranslateProviderConfig,
    MTranServerProviderConfig, TencentProviderConfig, TransmartProviderConfig,
    YoudaoProviderConfig,
};

#[cfg(test)]
mod tests;
