//! Traditional (non-LLM) translation providers.

pub mod baidu;
pub mod bing_web;
pub mod caiyun;
pub mod deepl;
pub mod google;
pub mod google_web;
pub mod libretranslate;
pub mod mtranserver;
pub mod system;
pub mod tencent;
pub mod transmart;
pub mod youdao;

#[cfg(feature = "baidu")]
pub use baidu::BaiduProvider;
pub use baidu::BaiduProviderConfig;
pub use bing_web::{BingWebProvider, BingWebProviderConfig};
#[cfg(feature = "caiyun")]
pub use caiyun::CaiyunProvider;
pub use caiyun::CaiyunProviderConfig;
pub use deepl::{DeepLProvider, DeepLProviderConfig};
#[cfg(feature = "google")]
pub use google::GoogleProvider;
pub use google::GoogleProviderConfig;
pub use google_web::{GoogleWebProvider, GoogleWebProviderConfig};
pub use libretranslate::{LibreTranslateProvider, LibreTranslateProviderConfig};
pub use mtranserver::{MTranServerProvider, MTranServerProviderConfig};
pub use system::SystemProvider;
pub use system::SystemTranslationService;
#[cfg(feature = "tencent")]
pub use tencent::TencentProvider;
pub use tencent::TencentProviderConfig;
pub use transmart::{TransmartProvider, TransmartProviderConfig};
#[cfg(feature = "youdao")]
pub use youdao::YoudaoProvider;
pub use youdao::YoudaoProviderConfig;
