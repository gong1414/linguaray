//! DeepL official Translate API (v2).
//! Docs: https://developers.deepl.com/docs/api-reference/translate
//!
//! POST JSON `{text:[...], target_lang, source_lang?}`.
//! Header `Authorization: DeepL-Auth-Key <key>`.
//! Free keys end with `:fx` and use `api-free.deepl.com`.

use async_trait::async_trait;

use super::{classify_http, lang, require_key};
use crate::engines::TraditionalEngine;
use crate::error::{Error, FallbackKind};

pub struct Deepl {
    endpoint: Option<String>,
}

impl Default for Deepl {
    fn default() -> Self {
        Self::new()
    }
}

impl Deepl {
    pub fn new() -> Self {
        Self { endpoint: None }
    }

    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: Some(endpoint.into()),
        }
    }

    pub fn resolve_url(key: &str, override_endpoint: Option<&str>) -> String {
        if let Some(ep) = override_endpoint {
            return ep.to_string();
        }
        if key.trim_end().ends_with(":fx") {
            "https://api-free.deepl.com/v2/translate".into()
        } else {
            "https://api.deepl.com/v2/translate".into()
        }
    }

    pub fn parse_body(body: &serde_json::Value) -> Result<String, Error> {
        let texts = body
            .get("translations")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                Error::FallbackEligible(FallbackKind::Parse("missing translations".into()))
            })?;
        let mut out = String::new();
        for item in texts {
            if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                out.push_str(t);
            }
        }
        if out.is_empty() {
            return Err(Error::FallbackEligible(FallbackKind::Parse(
                "empty DeepL translations".into(),
            )));
        }
        Ok(out)
    }
}

#[async_trait]
impl TraditionalEngine for Deepl {
    fn id(&self) -> &str {
        "deepl"
    }
    fn label(&self) -> &str {
        "DeepL"
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
        let key = require_key(key, "deepl")?;
        let url = Self::resolve_url(key, self.endpoint.as_deref());
        let target = lang::deepl(to).ok_or_else(|| {
            crate::error::ConfigKind::InvalidRequest {
                provider: "deepl".into(),
                status: 400,
            }
        })?;
        let mut payload = serde_json::json!({
            "text": [text],
            "target_lang": target,
        });
        if let Some(src) = lang::deepl(from) {
            payload["source_lang"] = serde_json::Value::String(src);
        }
        let resp = client
            .post(&url)
            .header("Authorization", format!("DeepL-Auth-Key {key}"))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::FallbackEligible(FallbackKind::Network(e.to_string())))?;
        let status = resp.status().as_u16();
        classify_http("deepl", status)?;
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
    fn free_key_selects_api_free_host() {
        assert!(Deepl::resolve_url("abc:fx", None).contains("api-free.deepl.com"));
        assert!(Deepl::resolve_url("pro-key", None).contains("api.deepl.com"));
        assert_eq!(
            Deepl::resolve_url("abc:fx", Some("https://example.test/v2/translate")),
            "https://example.test/v2/translate"
        );
    }

    #[test]
    fn parse_official_shape() {
        let body = serde_json::json!({
            "translations": [
                {"detected_source_language": "EN", "text": "Hallo"}
            ]
        });
        assert_eq!(Deepl::parse_body(&body).unwrap(), "Hallo");
    }
}
