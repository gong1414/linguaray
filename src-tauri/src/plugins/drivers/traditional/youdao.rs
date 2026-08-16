//! Youdao Zhiyun text translation API (signType=v3).
//! Docs: https://ai.youdao.com/DOCSIRMA/html/trans/api/wbfy/index.html
//!
//! POST `https://openapi.youdao.com/api` as form fields.
//! `sign = sha256(appKey + input + salt + curtime + appSecret)` hex lowercase.
//! `input` = q if len<=20 else q[0..10] + len + q[len-10..].
//! Key format: `appKey:appSecret`.

use async_trait::async_trait;
use sha2::{Digest, Sha256};

use super::{classify_http, lang, require_key, split_pair};
use crate::engines::TraditionalEngine;
use crate::error::{Error, FallbackKind};

pub struct Youdao {
    endpoint: String,
    salt: Option<String>,
    curtime: Option<String>,
}

impl Default for Youdao {
    fn default() -> Self {
        Self::new()
    }
}

impl Youdao {
    pub fn new() -> Self {
        Self {
            endpoint: "https://openapi.youdao.com/api".into(),
            salt: None,
            curtime: None,
        }
    }

    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            salt: None,
            curtime: None,
        }
    }

    pub fn with_nonce(mut self, salt: impl Into<String>, curtime: impl Into<String>) -> Self {
        self.salt = Some(salt.into());
        self.curtime = Some(curtime.into());
        self
    }

    pub fn input_digest(q: &str) -> String {
        let n = q.chars().count();
        if n <= 20 {
            q.to_string()
        } else {
            let head: String = q.chars().take(10).collect();
            let tail: String = q.chars().skip(n - 10).collect();
            format!("{head}{n}{tail}")
        }
    }

    pub fn sign(app_key: &str, q: &str, salt: &str, curtime: &str, secret: &str) -> String {
        let mut h = Sha256::new();
        h.update(app_key.as_bytes());
        h.update(Self::input_digest(q).as_bytes());
        h.update(salt.as_bytes());
        h.update(curtime.as_bytes());
        h.update(secret.as_bytes());
        format!("{:x}", h.finalize())
    }

    pub fn parse_body(body: &serde_json::Value) -> Result<String, Error> {
        if let Some(code) = body.get("errorCode").and_then(|v| v.as_str()) {
            if code != "0" {
                return Err(Error::FallbackEligible(FallbackKind::Parse(format!(
                    "youdao errorCode {code}"
                ))));
            }
        }
        let parts = body
            .get("translation")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                Error::FallbackEligible(FallbackKind::Parse("missing translation".into()))
            })?;
        let mut out = String::new();
        for p in parts {
            if let Some(s) = p.as_str() {
                out.push_str(s);
            }
        }
        if out.is_empty() {
            return Err(Error::FallbackEligible(FallbackKind::Parse(
                "empty translation".into(),
            )));
        }
        Ok(out)
    }
}

#[async_trait]
impl TraditionalEngine for Youdao {
    fn id(&self) -> &str {
        "youdao"
    }
    fn label(&self) -> &str {
        "有道"
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
        let raw = require_key(key, "youdao")?;
        let (app_key, secret) = split_pair(raw, "youdao")?;
        let salt = self
            .salt
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let curtime = self.curtime.clone().unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string()
        });
        let sign = Self::sign(app_key, text, &salt, &curtime, secret);
        let resp = client
            .post(&self.endpoint)
            .form(&[
                ("q", text),
                ("from", &lang::youdao(from)),
                ("to", &lang::youdao(to)),
                ("appKey", app_key),
                ("salt", &salt),
                ("curtime", &curtime),
                ("signType", "v3"),
                ("sign", &sign),
            ])
            .send()
            .await
            .map_err(|e| Error::FallbackEligible(FallbackKind::Network(e.to_string())))?;
        let status = resp.status().as_u16();
        classify_http("youdao", status)?;
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
    fn input_digest_short_is_identity() {
        assert_eq!(Youdao::input_digest("hello"), "hello");
    }

    #[test]
    fn input_digest_long_uses_head_len_tail() {
        let q = "abcdefghijklmnopqrstuvwxyz"; // 26
        assert_eq!(Youdao::input_digest(q), "abcdefghij26qrstuvwxyz");
    }

    #[test]
    fn parse_official_shape() {
        let body = serde_json::json!({"errorCode":"0","translation":["你好"]});
        assert_eq!(Youdao::parse_body(&body).unwrap(), "你好");
    }
}
