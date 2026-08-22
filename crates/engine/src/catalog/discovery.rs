//! Model-discovery HTTP policy shared by OpenAI-compatible adapters.

use super::urls::{redact_secrets, truncate_error_body};
use serde_json::Value;
use std::collections::BTreeSet;
use std::time::Duration;

pub const MODELS_FETCH_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CandidateOutcome {
    Success(Vec<String>),
    TryNext,
    Fail(String),
}

pub fn parse_openai_models_body(body: &str) -> Result<Vec<String>, String> {
    let json = serde_json::from_str::<Value>(body)
        .map_err(|error| format!("invalid models response: {error}"))?;
    let items = json
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| "invalid models response: missing data array".to_owned())?;
    let mut models: Vec<String> = items
        .iter()
        .filter_map(|item| item.get("id").and_then(Value::as_str).map(str::to_owned))
        .collect();
    models.sort();
    let unique: BTreeSet<String> = models.into_iter().collect();
    Ok(unique.into_iter().collect())
}

pub fn interpret_models_response(status: u16, body: &str, secrets: &[&str]) -> CandidateOutcome {
    let redacted = truncate_error_body(&redact_secrets(body, secrets));
    if (200..300).contains(&status) {
        return match parse_openai_models_body(body) {
            Ok(models) => CandidateOutcome::Success(models),
            Err(error) => CandidateOutcome::Fail(error),
        };
    }
    if status == 404 || status == 405 {
        return CandidateOutcome::TryNext;
    }
    CandidateOutcome::Fail(format!("HTTP {status}: {redacted}"))
}

/// Walk candidate URLs. Only 404/405 continue; any other HTTP error stops.
pub async fn fetch_models_with_candidates<F, Fut>(
    candidates: &[String],
    secrets: &[&str],
    mut fetch: F,
) -> Result<Vec<String>, String>
where
    F: FnMut(String) -> Fut,
    Fut: std::future::Future<Output = Result<(u16, String), String>>,
{
    let mut last_error = "no models endpoint responded".to_owned();
    for url in candidates {
        let redacted_url = redact_secrets(url, secrets);
        match fetch(url.clone()).await {
            Ok((status, body)) => match interpret_models_response(status, &body, secrets) {
                CandidateOutcome::Success(models) => return Ok(models),
                CandidateOutcome::TryNext => {
                    last_error = format!("HTTP {status} at {redacted_url}");
                }
                CandidateOutcome::Fail(error) => return Err(error),
            },
            Err(error) => {
                return Err(redact_secrets(&error, secrets));
            }
        }
    }
    Err(last_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::urls::model_discovery_candidates;

    #[test]
    fn parses_and_sorts_model_ids() {
        let body = r#"{"data":[{"id":"gpt-4o"},{"id":"gpt-4o-mini"},{"id":"gpt-4o"}]}"#;
        assert_eq!(
            parse_openai_models_body(body).unwrap(),
            vec!["gpt-4o".to_owned(), "gpt-4o-mini".to_owned()]
        );
    }

    #[test]
    fn successful_but_malformed_payload_is_an_error() {
        assert!(matches!(
            interpret_models_response(200, "<html>not json</html>", &[]),
            CandidateOutcome::Fail(message) if message.contains("invalid models response")
        ));
    }

    #[test]
    fn status_404_and_405_try_next() {
        assert_eq!(
            interpret_models_response(404, "missing", &[]),
            CandidateOutcome::TryNext
        );
        assert_eq!(
            interpret_models_response(405, "nope", &[]),
            CandidateOutcome::TryNext
        );
    }

    #[test]
    fn status_401_stops_and_redacts() {
        let outcome = interpret_models_response(
            401,
            "Bearer sk-secret-value is invalid",
            &["sk-secret-value"],
        );
        match outcome {
            CandidateOutcome::Fail(message) => {
                assert!(!message.contains("sk-secret-value"));
                assert!(message.contains("[redacted]"));
            }
            other => panic!("expected fail, got {other:?}"),
        }
    }

    #[test]
    fn truncates_error_body_to_512() {
        let body = "e".repeat(800);
        let outcome = interpret_models_response(500, &body, &[]);
        match outcome {
            CandidateOutcome::Fail(message) => {
                let body_part = message.split(": ").nth(1).unwrap_or(&message);
                assert_eq!(body_part.chars().count(), 512);
            }
            other => panic!("expected fail, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn walks_404_then_succeeds() {
        let candidates = vec![
            "http://127.0.0.1/missing/models".to_owned(),
            "http://127.0.0.1/v1/models".to_owned(),
        ];
        let result = fetch_models_with_candidates(&candidates, &[], |url| async move {
            if url.ends_with("/missing/models") {
                Ok((404, "no".to_owned()))
            } else {
                Ok((200, r#"{"data":[{"id":"alpha"}]}"#.to_owned()))
            }
        })
        .await
        .expect("models");
        assert_eq!(result, vec!["alpha".to_owned()]);
    }

    #[tokio::test]
    async fn stops_on_401_without_trying_next() {
        let candidates = vec![
            "http://127.0.0.1/v1/models".to_owned(),
            "http://127.0.0.1/v1/models-alt".to_owned(),
        ];
        let error = fetch_models_with_candidates(&candidates, &["sk-live"], |url| async move {
            assert!(
                !url.contains("models-alt"),
                "must not fall through after 401"
            );
            Ok((401, "sk-live denied".to_owned()))
        })
        .await
        .expect_err("401");
        assert!(!error.contains("sk-live"));
    }

    #[tokio::test]
    async fn network_error_stops_and_redacts() {
        let candidates = vec!["http://127.0.0.1/v1/models?api_key=sk-live".to_owned()];
        let error = fetch_models_with_candidates(&candidates, &["sk-live"], |_url| async move {
            Err("timeout contacting http://127.0.0.1/v1/models?api_key=sk-live".to_owned())
        })
        .await
        .expect_err("timeout");
        assert!(!error.contains("sk-live"));
    }

    #[test]
    fn candidate_rules_cover_cc_switch_order() {
        let urls = model_discovery_candidates(
            "https://gateway.example/v1/chat/completions",
            Some("https://custom.example/models"),
        );
        assert_eq!(urls, vec!["https://custom.example/models"]);
    }
}
