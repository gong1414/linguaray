//! OpenAI-compatible URL joining and model-discovery candidate generation.

/// Join `{baseUrl}` (API root, including version when present) with a relative
/// path without producing `/v1/v1`.
pub fn join_openai_path(base_url: &str, path: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    let path = if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("/{path}")
    };
    if base.ends_with("/v1") && path.starts_with("/v1/") {
        format!("{}{}", base.trim_end_matches("/v1"), path)
    } else {
        format!("{base}{path}")
    }
}

/// Chat completions URL for an OpenAI-compatible root.
pub fn openai_chat_completions_url(base_url: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    if has_api_version_suffix(base) {
        format!("{base}/chat/completions")
    } else {
        format!("{base}/v1/chat/completions")
    }
}

/// Models listing URL for an OpenAI-compatible root (single best guess).
pub fn openai_models_url(base_url: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    if has_api_version_suffix(base) {
        format!("{base}/models")
    } else {
        format!("{base}/v1/models")
    }
}

fn has_api_version_suffix(base: &str) -> bool {
    let lower = base.to_ascii_lowercase();
    if lower.ends_with("/openai") {
        return true;
    }
    if let Some(index) = lower.rfind("/v") {
        let rest = &lower[index + 2..];
        if rest.is_empty() {
            return false;
        }
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return false;
        }
        rest[digits..].bytes().all(|b| b.is_ascii_lowercase())
    } else {
        false
    }
}

const COMPAT_SUFFIXES: &[&str] = &[
    "/api/claudecode",
    "/api/anthropic",
    "/apps/anthropic",
    "/api/coding",
    "/claudecode",
    "/anthropic",
    "/step_plan",
    "/coding",
    "/claude",
];

/// Candidate `/models` URLs following the fixed discovery rules.
pub fn model_discovery_candidates(base_url: &str, models_url: Option<&str>) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(explicit) = models_url.map(str::trim).filter(|s| !s.is_empty()) {
        push_unique(&mut out, Some(explicit));
        return out;
    }

    let base = base_url.trim();
    if base.is_empty() {
        return out;
    }

    let trimmed = base.trim_end_matches('/');
    if has_api_version_suffix(trimmed) {
        push_unique(&mut out, Some(&format!("{trimmed}/models")));
    } else {
        push_unique(&mut out, Some(&format!("{trimmed}/v1/models")));
    }

    if let Some(v1) = truncate_at_v1(trimmed) {
        push_unique(&mut out, Some(&format!("{v1}/models")));
    }

    for suffix in COMPAT_SUFFIXES {
        if let Some(stripped) = strip_suffix_ci(trimmed, suffix) {
            let stripped = stripped.trim_end_matches('/');
            if stripped.is_empty() {
                continue;
            }
            if has_api_version_suffix(stripped) {
                push_unique(&mut out, Some(&format!("{stripped}/models")));
            } else {
                push_unique(&mut out, Some(&format!("{stripped}/v1/models")));
            }
            if let Some(v1) = truncate_at_v1(stripped) {
                push_unique(&mut out, Some(&format!("{v1}/models")));
            }
        }
    }

    out
}

fn truncate_at_v1(url: &str) -> Option<String> {
    let lower = url.to_ascii_lowercase();
    let index = lower.find("/v1/")?;
    Some(url[..index + 3].to_owned())
}

fn strip_suffix_ci<'a>(value: &'a str, suffix: &str) -> Option<&'a str> {
    let lower = value.to_ascii_lowercase();
    let suffix_lower = suffix.to_ascii_lowercase();
    if lower.ends_with(&suffix_lower) {
        Some(&value[..value.len() - suffix.len()])
    } else {
        None
    }
}

fn push_unique(out: &mut Vec<String>, candidate: Option<&str>) {
    let Some(candidate) = candidate else {
        return;
    };
    if candidate.is_empty() {
        return;
    }
    if !out.iter().any(|existing| existing == candidate) {
        out.push(candidate.to_owned());
    }
}

/// Redact API keys and bearer tokens from URLs, headers, and error bodies.
pub fn redact_secrets(text: &str, secrets: &[&str]) -> String {
    let mut out = text.to_owned();
    for secret in secrets {
        let secret = secret.trim();
        if secret.len() < 4 {
            continue;
        }
        out = out.replace(secret, "[redacted]");
    }
    out = redact_bearer(&out);
    out = redact_query_key(&out, "key");
    out = redact_query_key(&out, "api_key");
    out = redact_query_key(&out, "apiKey");
    out
}

fn redact_bearer(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let lower = text.to_ascii_lowercase();
    let mut i = 0;
    while i < text.len() {
        if let Some(rest) = lower[i..].strip_prefix("bearer ") {
            out.push_str("Bearer [redacted]");
            let skipped = rest
                .bytes()
                .take_while(|b| !b.is_ascii_whitespace() && *b != b'"' && *b != b'\'')
                .count();
            i += "bearer ".len() + skipped;
            continue;
        }
        out.push(text[i..].chars().next().unwrap());
        i += text[i..].chars().next().unwrap().len_utf8();
    }
    out
}

fn redact_query_key(text: &str, key: &str) -> String {
    let needle = format!("{key}=");
    let mut out = String::new();
    let mut rest = text;
    while let Some(index) = rest.find(&needle) {
        out.push_str(&rest[..index + needle.len()]);
        let after = &rest[index + needle.len()..];
        out.push_str("[redacted]");
        let skip = after
            .bytes()
            .take_while(|b| *b != b'&' && *b != b' ' && *b != b'"' && *b != b'\'')
            .count();
        rest = &after[skip..];
    }
    out.push_str(rest);
    out
}

/// Truncate an error body to 512 characters.
pub fn truncate_error_body(body: &str) -> String {
    const LIMIT: usize = 512;
    if body.chars().count() <= LIMIT {
        body.to_owned()
    } else {
        body.chars().take(LIMIT).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_does_not_double_v1() {
        assert_eq!(
            join_openai_path("https://api.openai.com/v1", "/v1/chat/completions"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            join_openai_path("https://api.openai.com/v1/", "/chat/completions"),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn chat_url_uses_versioned_root() {
        assert_eq!(
            openai_chat_completions_url("https://api.openai.com/v1"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            openai_chat_completions_url("https://api.openai.com"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            openai_chat_completions_url("https://generativelanguage.googleapis.com/v1beta/openai"),
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
        );
    }

    #[test]
    fn models_url_from_versioned_root() {
        assert_eq!(
            openai_models_url("https://openrouter.ai/api/v1"),
            "https://openrouter.ai/api/v1/models"
        );
        assert_eq!(
            openai_models_url("http://127.0.0.1:1234/v1"),
            "http://127.0.0.1:1234/v1/models"
        );
    }

    #[test]
    fn discovery_prefers_explicit_models_url() {
        let urls = model_discovery_candidates(
            "https://api.openai.com/v1",
            Some("https://example.test/custom/models"),
        );
        assert_eq!(urls, vec!["https://example.test/custom/models"]);
    }

    #[test]
    fn discovery_appends_models_when_base_is_versioned() {
        let urls = model_discovery_candidates("https://api.openai.com/v1", None);
        assert_eq!(urls[0], "https://api.openai.com/v1/models");
    }

    #[test]
    fn discovery_appends_v1_models_when_unversioned() {
        let urls = model_discovery_candidates("https://api.example.com", None);
        assert_eq!(urls[0], "https://api.example.com/v1/models");
    }

    #[test]
    fn discovery_truncates_full_chat_url() {
        let urls = model_discovery_candidates("https://api.openai.com/v1/chat/completions", None);
        assert!(urls.contains(&"https://api.openai.com/v1/models".to_owned()));
    }

    #[test]
    fn discovery_strips_compat_suffixes() {
        let urls = model_discovery_candidates("https://gateway.example/v1/anthropic", None);
        assert!(urls.contains(&"https://gateway.example/v1/models".to_owned()));
    }

    #[test]
    fn discovery_deduplicates_and_keeps_order() {
        let urls = model_discovery_candidates(
            "https://api.openai.com/v1",
            Some("https://api.openai.com/v1/models"),
        );
        assert_eq!(
            urls.iter()
                .filter(|u| *u == "https://api.openai.com/v1/models")
                .count(),
            1
        );
    }

    #[test]
    fn redacts_bearer_and_query_secrets() {
        let text =
            "Authorization: Bearer sk-secret-value url=https://x.test?api_key=sk-secret-value";
        let redacted = redact_secrets(text, &["sk-secret-value"]);
        assert!(!redacted.contains("sk-secret-value"));
        assert!(redacted.contains("[redacted]"));
    }

    #[test]
    fn truncates_error_bodies_at_512() {
        let body = "a".repeat(600);
        let truncated = truncate_error_body(&body);
        assert_eq!(truncated.chars().count(), 512);
    }
}
