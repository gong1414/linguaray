//! Provider balance/quota fetch. HTTP only when the profile opted in.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, specta::Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BalanceResult {
    Unsupported,
    Ok { balance: String, quota: Option<String> },
    Error { message: String },
}

#[derive(Debug, Deserialize)]
struct LooseBalance {
    #[serde(default)]
    total_available: Option<serde_json::Value>,
    #[serde(default)]
    balance: Option<serde_json::Value>,
    #[serde(default)]
    quota: Option<serde_json::Value>,
    #[serde(default)]
    total_used: Option<serde_json::Value>,
}

/// Decide whether to fetch. Capability false never hits the network.
pub fn should_fetch(capability_balance: bool) -> bool {
    capability_balance
}

pub fn parse_balance_json(body: &str) -> BalanceResult {
    let parsed: LooseBalance = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => {
            return BalanceResult::Error {
                message: e.to_string(),
            }
        }
    };
    let balance = stringify_num(parsed.total_available)
        .or_else(|| stringify_num(parsed.balance));
    match balance {
        Some(balance) => BalanceResult::Ok {
            quota: stringify_num(parsed.quota).or_else(|| stringify_num(parsed.total_used)),
            balance,
        },
        None => BalanceResult::Error {
            message: "no balance field".into(),
        },
    }
}

fn stringify_num(v: Option<serde_json::Value>) -> Option<String> {
    match v {
        Some(serde_json::Value::Number(n)) => Some(n.to_string()),
        Some(serde_json::Value::String(s)) if !s.is_empty() => Some(s),
        _ => None,
    }
}

/// GET `url` with bearer key. Used by the IPC command and tests (wiremock).
pub async fn fetch_balance_url(url: &str, key: &str) -> BalanceResult {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return BalanceResult::Error {
                message: e.to_string(),
            }
        }
    };
    let resp = match client
        .get(url)
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return BalanceResult::Error {
                message: e.to_string(),
            }
        }
    };
    if !resp.status().is_success() {
        return BalanceResult::Error {
            message: format!("http {}", resp.status()),
        };
    }
    match resp.text().await {
        Ok(t) => parse_balance_json(&t),
        Err(e) => BalanceResult::Error {
            message: e.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_fetch_when_capability_false() {
        assert!(!should_fetch(false));
        assert!(should_fetch(true));
    }

    #[test]
    fn parses_openai_style_grant() {
        let got = parse_balance_json(r#"{"total_available":12.5,"total_used":1}"#);
        assert_eq!(
            got,
            BalanceResult::Ok {
                balance: "12.5".into(),
                quota: Some("1".into()),
            }
        );
    }
}
