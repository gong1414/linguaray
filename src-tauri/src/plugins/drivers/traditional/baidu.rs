//! Baidu Fanyi Open Platform — general text API.
//! Docs: https://api.fanyi.baidu.com/doc/21
//!
//! POST `https://fanyi-api.baidu.com/api/trans/vip/translate`
//! Fields: q, from, to, appid, salt, sign.
//! `sign = md5(appid + q + salt + secret)` (32-char lowercase hex).
//! Key format: `appid:secret`.

use async_trait::async_trait;
use md5::{Digest, Md5};

use super::{classify_http, lang, require_key, split_pair};
use crate::engines::TraditionalEngine;
use crate::error::{Error, FallbackKind};

pub struct Baidu {
    endpoint: String,
    salt: Option<String>,
}

impl Default for Baidu {
    fn default() -> Self {
        Self::new()
    }
}

impl Baidu {
    pub fn new() -> Self {
        Self {
            endpoint: "https://fanyi-api.baidu.com/api/trans/vip/translate".into(),
            salt: None,
        }
    }

    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            salt: None,
        }
    }

    pub fn with_salt(mut self, salt: impl Into<String>) -> Self {
        self.salt = Some(salt.into());
        self
    }

    pub fn sign(appid: &str, q: &str, salt: &str, secret: &str) -> String {
        let mut h = Md5::new();
        h.update(appid.as_bytes());
        h.update(q.as_bytes());
        h.update(salt.as_bytes());
        h.update(secret.as_bytes());
        format!("{:x}", h.finalize())
    }

    pub fn parse_body(body: &serde_json::Value) -> Result<String, Error> {
        if let Some(code) = body.get("error_code").and_then(|v| v.as_str()) {
            if code != "52000" {
                return Err(Error::FallbackEligible(FallbackKind::Parse(format!(
                    "baidu error_code {code}"
                ))));
            }
        }
        let rows = body
            .get("trans_result")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                Error::FallbackEligible(FallbackKind::Parse("missing trans_result".into()))
            })?;
        let mut out = String::new();
        for row in rows {
            if let Some(dst) = row.get("dst").and_then(|v| v.as_str()) {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(dst);
            }
        }
        if out.is_empty() {
            return Err(Error::FallbackEligible(FallbackKind::Parse(
                "empty trans_result".into(),
            )));
        }
        Ok(out)
    }
}

#[async_trait]
impl TraditionalEngine for Baidu {
    fn id(&self) -> &str {
        "baidu"
    }
    fn label(&self) -> &str {
        "百度翻译"
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
        let raw = require_key(key, "baidu")?;
        let (appid, secret) = split_pair(raw, "baidu")?;
        let salt = self
            .salt
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let sign = Self::sign(appid, text, &salt, secret);
        let resp = client
            .post(&self.endpoint)
            .form(&[
                ("q", text),
                ("from", &lang::baidu(from)),
                ("to", &lang::baidu(to)),
                ("appid", appid),
                ("salt", &salt),
                ("sign", &sign),
            ])
            .send()
            .await
            .map_err(|e| Error::FallbackEligible(FallbackKind::Network(e.to_string())))?;
        let status = resp.status().as_u16();
        classify_http("baidu", status)?;
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
    fn sign_matches_official_concatenation() {
        // Official: md5(appid + q + salt + secret), lowercase hex.
        let got = Baidu::sign("app", "hello", "salt", "sec");
        let expect = format!("{:x}", Md5::digest(b"apphellosaltsec"));
        assert_eq!(got, expect);
        assert_eq!(got.len(), 32);
    }

    #[test]
    fn parse_official_shape() {
        let body = serde_json::json!({
            "from": "en",
            "to": "zh",
            "trans_result": [{"src": "hi", "dst": "你好"}]
        });
        assert_eq!(Baidu::parse_body(&body).unwrap(), "你好");
    }
}
