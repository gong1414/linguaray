//! Google Translate (free, keyless) — ported from pot's google plugin logic.
//! Endpoint: translate.google.com/translate_a/single (client=gtx, dt=t).
//! Response: nested JSON array; translated segments are at [0][*][0].
use async_trait::async_trait;
use crate::error::{Error, FallbackKind};

pub struct Google { base_url: String }

impl Google {
    pub fn new() -> Self { Self { base_url: "https://translate.google.com".into() } }
    pub fn with_base(base_url: impl Into<String>) -> Self { Self { base_url: base_url.into() } }
}

#[async_trait]
impl super::TraditionalEngine for Google {
    fn id(&self) -> &str { "google" }
    fn label(&self) -> &str { "Google Translate" }

    async fn translate(
        &self,
        client: &reqwest::Client,
        text: &str,
        from: &str,
        to: &str,
    ) -> Result<String, Error> {
        let sl = if from == "auto" { "auto".to_string() } else { from.to_string() };
        let url = format!("{}/translate_a/single", self.base_url);
        let resp = client
            .get(&url)
            .query(&[
                ("client", "gtx"), ("sl", &sl), ("tl", to),
                ("dt", "t"), ("q", text),
            ])
            .send().await
            .map_err(|e| Error::FallbackEligible(FallbackKind::Network(e.to_string())))?;
        let status = resp.status().as_u16();
        if status == 429 || (500..600).contains(&status) || !resp.status().is_success() {
            return Err(Error::FallbackEligible(FallbackKind::ProviderStatus { status }));
        }
        let json: serde_json::Value = resp.json().await
            .map_err(|e| Error::FallbackEligible(FallbackKind::Parse(e.to_string())))?;
        // Response shape: [ [ ["translated","orig",...], ... ], ... ].
        let segments = json.get(0).and_then(|a| a.as_array()).ok_or_else(|| {
            Error::FallbackEligible(FallbackKind::Parse("no segment array".into()))
        })?;
        let mut out = String::new();
        for seg in segments {
            if let Some(t) = seg.get(0).and_then(|v| v.as_str()) {
                out.push_str(t);
            }
        }
        if out.is_empty() {
            return Err(Error::FallbackEligible(FallbackKind::Parse("empty translation".into())));
        }
        Ok(out)
    }
}
