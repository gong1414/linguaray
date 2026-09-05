use std::future::Future;

use linguaray_core::{DictionaryError, OcrError, TranslationError};
use reqwest::{Response, StatusCode};

async fn failed_response(response: Response, secrets: &[&str]) -> (StatusCode, String) {
    let status = response.status();
    let mut body = response.text().await.unwrap_or_default();
    for secret in secrets.iter().copied().filter(|value| !value.is_empty()) {
        body = body.replace(secret, "[redacted]");
    }
    let message = if body.is_empty() {
        status.to_string()
    } else {
        body.chars().take(512).collect()
    };
    (status, message)
}

pub trait ClassifyHttpResponse: Sized {
    fn classify_status(provider: &'static str, status: u16, message: String) -> Self;

    fn from_response(
        provider: &'static str,
        response: Response,
    ) -> impl Future<Output = Result<Response, Self>> {
        Self::from_response_redacting(provider, response, &[])
    }

    fn from_response_redacting(
        provider: &'static str,
        response: Response,
        secrets: &[&str],
    ) -> impl Future<Output = Result<Response, Self>> {
        let secrets: Vec<String> = secrets.iter().map(|secret| (*secret).to_string()).collect();
        async move {
            if response.status().is_success() {
                return Ok(response);
            }
            let secret_refs: Vec<&str> = secrets.iter().map(String::as_str).collect();
            let (status, message) = failed_response(response, &secret_refs).await;
            Err(Self::classify_status(provider, status.as_u16(), message))
        }
    }
}

macro_rules! impl_classify_http_response {
    ($ty:ty) => {
        impl ClassifyHttpResponse for $ty {
            fn classify_status(provider: &'static str, status: u16, message: String) -> Self {
                <$ty>::from_http_status(provider, status, message)
            }
        }
    };
}

impl_classify_http_response!(TranslationError);
impl_classify_http_response!(DictionaryError);
impl_classify_http_response!(OcrError);

#[cfg(test)]
mod tests {
    #[test]
    fn redaction_is_bounded_and_ignores_empty_values() {
        let mut message = "Bearer secret".to_owned();
        for _ in 0..600 {
            message.push('x');
        }
        let sanitized = ["secret", ""].into_iter().fold(message, |value, secret| {
            if secret.is_empty() {
                value
            } else {
                value.replace(secret, "[redacted]")
            }
        });
        assert!(sanitized.contains("[redacted]"));
        assert!(!sanitized.contains("Bearer secret"));
        assert!(sanitized.len() > 512);
    }
}
