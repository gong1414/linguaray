//! Keyless Google GTX client, written against the publicly observed contract:
//! `GET {origin}/translate_a/single?client=gtx&sl=&tl=&dt=t&q=`
//! Response JSON: a nested array whose `[0]` is segments; each segment's `[0]`
//! is the translated string. Concatenate those strings.
//!
//! This is not an official Google API and has no SLA. See engines.json notes
//! and `docs/superpowers/archive/google-gtx-cleanroom.md`.

use async_trait::async_trait;

use super::classify_http;
use crate::engines::TraditionalEngine;
use crate::error::{Error, FallbackKind};

pub struct Google {
    origin: String,
}

impl Default for Google {
    fn default() -> Self {
        Self::new()
    }
}

impl Google {
    pub fn new() -> Self {
        Self {
            origin: "https://translate.google.com".into(),
        }
    }

    pub fn with_origin(origin: impl Into<String>) -> Self {
        Self {
            origin: origin.into(),
        }
    }

    pub fn parse_segments(body: &serde_json::Value) -> Result<String, Error> {
        let segments = body
            .get(0)
            .and_then(|v| v.as_array())
            .ok_or_else(|| Error::FallbackEligible(FallbackKind::Parse("missing segment list".into())))?;
        let mut out = String::new();
        for segment in segments {
            if let Some(piece) = segment.get(0).and_then(|v| v.as_str()) {
                out.push_str(piece);
            }
        }
        if out.is_empty() {
            return Err(Error::FallbackEligible(FallbackKind::Parse(
                "no translated segments".into(),
            )));
        }
        Ok(out)
    }
}

#[async_trait]
impl TraditionalEngine for Google {
    fn id(&self) -> &str {
        "google"
    }
    fn label(&self) -> &str {
        "Google Translate"
    }
    fn needs_key(&self) -> bool {
        false
    }

    async fn translate(
        &self,
        client: &reqwest::Client,
        text: &str,
        from: &str,
        to: &str,
        key: Option<&str>,
    ) -> Result<String, Error> {
        let _ = key;
        let sl = if from.is_empty() { "auto" } else { from };
        let url = format!("{}/translate_a/single", self.origin.trim_end_matches('/'));
        let resp = client
            .get(&url)
            .query(&[
                ("client", "gtx"),
                ("sl", sl),
                ("tl", to),
                ("dt", "t"),
                ("q", text),
            ])
            .send()
            .await
            .map_err(|e| Error::FallbackEligible(FallbackKind::Network(e.to_string())))?;
        let status = resp.status().as_u16();
        classify_http("google", status)?;
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::FallbackEligible(FallbackKind::Parse(e.to_string())))?;
        Self::parse_segments(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concatenates_first_cell_of_each_segment() {
        let body = serde_json::json!([
            [["你好", "hello"], ["世界", "world"]],
            null,
            "en"
        ]);
        assert_eq!(Google::parse_segments(&body).unwrap(), "你好世界");
    }

    #[test]
    fn rejects_object_payload() {
        let err = Google::parse_segments(&serde_json::json!({"error": 1})).unwrap_err();
        assert!(matches!(err, Error::FallbackEligible(FallbackKind::Parse(_))));
    }
}
