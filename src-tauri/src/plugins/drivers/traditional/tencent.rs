//! Tencent Cloud TMT TextTranslate (2018-03-21) via API 3.0 TC3-HMAC-SHA256.
//! Docs: https://cloud.tencent.com/document/api/551/15619
//!
//! POST `https://tmt.tencentcloudapi.com` JSON
//! `{SourceText, Source, Target, ProjectId:0}`.
//! Key format: `secretId:secretKey`.

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use super::{classify_http, lang, require_key, split_pair};
use crate::engines::TraditionalEngine;
use crate::error::{Error, FallbackKind};

type HmacSha256 = Hmac<Sha256>;

pub struct Tencent {
    host: String,
    timestamp: Option<u64>,
}

impl Default for Tencent {
    fn default() -> Self {
        Self::new()
    }
}

impl Tencent {
    pub fn new() -> Self {
        Self {
            host: "tmt.tencentcloudapi.com".into(),
            timestamp: None,
        }
    }

    pub fn with_host(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            timestamp: None,
        }
    }

    pub fn with_timestamp(mut self, ts: u64) -> Self {
        self.timestamp = Some(ts);
        self
    }

    pub fn hashed_payload(body: &str) -> String {
        format!("{:x}", Sha256::digest(body.as_bytes()))
    }

    pub fn canonical_request(host: &str, payload_hash: &str) -> String {
        format!(
            "POST\n/\n\ncontent-type:application/json; charset=utf-8\nhost:{host}\n\ncontent-type;host\n{payload_hash}"
        )
    }

    fn hmac(key: &[u8], msg: &str) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key).expect("hmac key");
        mac.update(msg.as_bytes());
        mac.finalize().into_bytes().to_vec()
    }

    pub fn tc3_authorization(
        secret_id: &str,
        secret_key: &str,
        host: &str,
        timestamp: u64,
        payload: &str,
    ) -> String {
        let date = unix_utc_date(timestamp);
        let service = "tmt";
        let credential_scope = format!("{date}/{service}/tc3_request");
        let payload_hash = Self::hashed_payload(payload);
        let canonical = Self::canonical_request(host, &payload_hash);
        let hashed_canonical = format!("{:x}", Sha256::digest(canonical.as_bytes()));
        let string_to_sign = format!(
            "TC3-HMAC-SHA256\n{timestamp}\n{credential_scope}\n{hashed_canonical}"
        );
        let mut secret = Self::hmac(format!("TC3{secret_key}").as_bytes(), &date);
        secret = Self::hmac(&secret, service);
        secret = Self::hmac(&secret, "tc3_request");
        let signature = {
            let mut mac = HmacSha256::new_from_slice(&secret).expect("hmac key");
            mac.update(string_to_sign.as_bytes());
            format!("{:x}", mac.finalize().into_bytes())
        };
        format!(
            "TC3-HMAC-SHA256 Credential={secret_id}/{credential_scope}, SignedHeaders=content-type;host, Signature={signature}"
        )
    }

    pub fn parse_body(body: &serde_json::Value) -> Result<String, Error> {
        if let Some(err) = body.pointer("/Response/Error/Message").and_then(|v| v.as_str())
        {
            return Err(Error::FallbackEligible(FallbackKind::Parse(err.into())));
        }
        body.pointer("/Response/TargetText")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                Error::FallbackEligible(FallbackKind::Parse("missing Response.TargetText".into()))
            })
    }
}

fn unix_utc_date(ts: u64) -> String {
    let z = ts as i64 / 86400 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[async_trait]
impl TraditionalEngine for Tencent {
    fn id(&self) -> &str {
        "tencent"
    }
    fn label(&self) -> &str {
        "腾讯"
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
        let raw = require_key(key, "tencent")?;
        let (secret_id, secret_key) = split_pair(raw, "tencent")?;
        let timestamp = self.timestamp.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });
        let payload = serde_json::json!({
            "SourceText": text,
            "Source": lang::tencent(from),
            "Target": lang::tencent(to),
            "ProjectId": 0,
        })
        .to_string();
        let auth = Self::tc3_authorization(secret_id, secret_key, &self.host, timestamp, &payload);
        let url = if self.host.starts_with("http") {
            self.host.clone()
        } else {
            format!("https://{}", self.host)
        };
        let host_header = self
            .host
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or(&self.host);
        let resp = client
            .post(&url)
            .header("Authorization", auth)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Host", host_header)
            .header("X-TC-Action", "TextTranslate")
            .header("X-TC-Version", "2018-03-21")
            .header("X-TC-Timestamp", timestamp.to_string())
            .header("X-TC-Region", "ap-guangzhou")
            .body(payload)
            .send()
            .await
            .map_err(|e| Error::FallbackEligible(FallbackKind::Network(e.to_string())))?;
        let status = resp.status().as_u16();
        classify_http("tencent", status)?;
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
    fn utc_date_known_instant() {
        // 2020-01-01 00:00:00 UTC
        assert_eq!(unix_utc_date(1577836800), "2020-01-01");
    }

    #[test]
    fn parse_official_shape() {
        let body = serde_json::json!({
            "Response": {"TargetText": "你好", "RequestId": "x"}
        });
        assert_eq!(Tencent::parse_body(&body).unwrap(), "你好");
    }

    #[test]
    fn authorization_is_deterministic() {
        let a = Tencent::tc3_authorization(
            "AKIDtest",
            "secret",
            "tmt.tencentcloudapi.com",
            1577836800,
            "{\"SourceText\":\"hi\"}",
        );
        let b = Tencent::tc3_authorization(
            "AKIDtest",
            "secret",
            "tmt.tencentcloudapi.com",
            1577836800,
            "{\"SourceText\":\"hi\"}",
        );
        assert_eq!(a, b);
        assert!(a.starts_with("TC3-HMAC-SHA256 Credential=AKIDtest/2020-01-01/tmt/tc3_request"));
    }
}
