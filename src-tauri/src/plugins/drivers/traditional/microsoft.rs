//! Azure Translator v3 (official).
//! Docs: https://learn.microsoft.com/azure/ai-services/translator/reference/v3-0-translate
//!
//! POST `{origin}/translate?api-version=3.0&to=` with body `[{"Text":"..."}]`.
//! Header `Ocp-Apim-Subscription-Key`. Optional `key|region` adds
//! `Ocp-Apim-Subscription-Region`.

use async_trait::async_trait;

use super::{classify_http, lang, require_key};
use crate::engines::TraditionalEngine;
use crate::error::{Error, FallbackKind};

pub struct Microsoft {
    origin: String,
}

impl Default for Microsoft {
    fn default() -> Self {
        Self::new()
    }
}

impl Microsoft {
    pub fn new() -> Self {
        Self {
            origin: "https://api.cognitive.microsofttranslator.com".into(),
        }
    }

    pub fn with_origin(origin: impl Into<String>) -> Self {
        Self {
            origin: origin.into(),
        }
    }

    pub fn split_key(key: &str) -> (&str, Option<&str>) {
        match key.split_once('|') {
            Some((k, region)) if !region.is_empty() => (k, Some(region)),
            _ => (key, None),
        }
    }

    pub fn parse_body(body: &serde_json::Value) -> Result<String, Error> {
        let first = body
            .as_array()
            .and_then(|a| a.first())
            .ok_or_else(|| {
                Error::FallbackEligible(FallbackKind::Parse("missing translation array".into()))
            })?;
        first
            .get("translations")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("text"))
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                Error::FallbackEligible(FallbackKind::Parse("missing translations[0].text".into()))
            })
    }
}

#[async_trait]
impl TraditionalEngine for Microsoft {
    fn id(&self) -> &str {
        "microsoft"
    }
    fn label(&self) -> &str {
        "Microsoft Translator"
    }
    fn needs_key(&self) -> bool {
        true
    }

    async fn translate(
        &self,
        client: &reqwest::Client,
        text: &str,
        from: &str,
        to: &str,
        key: Option<&str>,
    ) -> Result<String, Error> {
        let raw = require_key(key, "microsoft")?;
        let (sub_key, region) = Self::split_key(raw);
        let target = lang::microsoft(to).ok_or_else(|| {
            crate::error::ConfigKind::InvalidRequest {
                provider: "microsoft".into(),
                status: 400,
            }
        })?;
        let url = format!("{}/translate", self.origin.trim_end_matches('/'));
        let mut req = client
            .post(&url)
            .query(&[("api-version", "3.0"), ("to", target.as_str())])
            .header("Ocp-Apim-Subscription-Key", sub_key)
            .header("Content-Type", "application/json; charset=UTF-8")
            .json(&serde_json::json!([{"Text": text}]));
        if let Some(src) = lang::microsoft(from) {
            req = req.query(&[("from", src)]);
        }
        if let Some(region) = region {
            req = req.header("Ocp-Apim-Subscription-Region", region);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| Error::FallbackEligible(FallbackKind::Network(e.to_string())))?;
        let status = resp.status().as_u16();
        classify_http("microsoft", status)?;
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::FallbackEligible(FallbackKind::Parse(e.to_string())))?;
        Self::parse_body(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_key_region() {
        assert_eq!(Microsoft::split_key("abc|eastus"), ("abc", Some("eastus")));
        assert_eq!(Microsoft::split_key("abc"), ("abc", None));
    }

    #[test]
    fn parse_official_shape() {
        let body = serde_json::json!([
            {"translations":[{"text":"你好","to":"zh-Hans"}]}
        ]);
        assert_eq!(Microsoft::parse_body(&body).unwrap(), "你好");
    }
}
