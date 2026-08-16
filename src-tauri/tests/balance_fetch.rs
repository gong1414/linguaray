use linguaray_lib::balance::{fetch_balance_url, parse_balance_json, should_fetch, BalanceResult};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn capability_gate() {
    assert!(!should_fetch(false));
    assert!(should_fetch(true));
}

#[tokio::test]
async fn fetch_only_hits_http_when_called() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/dashboard/billing/credit_grants"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total_available": 9.5,
            "total_used": 0.5
        })))
        .mount(&server)
        .await;
    let url = format!("{}/v1/dashboard/billing/credit_grants", server.uri());
    let got = fetch_balance_url(&url, "sk-test").await;
    assert_eq!(
        got,
        BalanceResult::Ok {
            balance: "9.5".into(),
            quota: Some("0.5".into()),
        }
    );
    assert!(matches!(
        parse_balance_json(r#"{"balance":"3"}"#),
        BalanceResult::Ok { .. }
    ));
}
