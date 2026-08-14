//! Traditional MT Drivers (one package, six request shapes).
//!
//! Runtime surface is still [`crate::engines::TraditionalEngine`] — chat-shaped
//! [`linguaray_contracts::EngineDriver`] does not fit GET/form/TC3 dialects.
//! `install` does not register into the ProtocolKind map.

mod baidu;
mod deepl;
mod google;
mod lang;
mod microsoft;
mod tencent;
mod youdao;

pub use baidu::Baidu;
pub use deepl::Deepl;
pub use google::Google;
pub use microsoft::Microsoft;
pub use tencent::Tencent;
pub use youdao::Youdao;

use super::DriverRegistry;
use crate::engines::TraditionalEngine;
use crate::error::{Error, FallbackKind};

pub fn install(_registry: &DriverRegistry) {}

pub fn all() -> Vec<Box<dyn TraditionalEngine>> {
    vec![
        Box::new(Google::new()),
        Box::new(Deepl::new()),
        Box::new(Microsoft::new()),
        Box::new(Baidu::new()),
        Box::new(Youdao::new()),
        Box::new(Tencent::new()),
    ]
}

pub fn find(id: &str) -> Option<Box<dyn TraditionalEngine>> {
    all().into_iter().find(|e| e.id() == id)
}

fn split_pair<'a>(key: &'a str, label: &str) -> Result<(&'a str, &'a str), Error> {
    let (a, b) = key.split_once(':').ok_or_else(|| {
        crate::error::ConfigKind::InvalidRequest {
            provider: label.to_string(),
            status: 400,
        }
    })?;
    if a.is_empty() || b.is_empty() {
        return Err(crate::error::ConfigKind::InvalidRequest {
            provider: label.to_string(),
            status: 400,
        }
        .into());
    }
    Ok((a, b))
}

fn require_key<'a>(key: Option<&'a str>, provider: &str) -> Result<&'a str, Error> {
    key.filter(|k| !k.is_empty()).ok_or_else(|| {
        crate::error::ConfigKind::MissingKey {
            provider: provider.to_string(),
        }
        .into()
    })
}

fn classify_http(provider: &str, status: u16) -> Result<(), Error> {
    if status == 401 || status == 403 {
        return Err(crate::error::ConfigKind::AuthFailed {
            provider: provider.to_string(),
            status,
        }
        .into());
    }
    if !(200..300).contains(&status) {
        return Err(Error::FallbackEligible(FallbackKind::ProviderStatus { status }));
    }
    Ok(())
}
