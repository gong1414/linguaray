use linguaray_lib::wire::{call, ApiKind, WireParams, build_prompt};
use linguaray_lib::providers::ProviderPreset;
use wiremock::{MockServer, Mock, ResponseTemplate};
use serde_json::json;

fn preset(endpoint: &str, kind: ApiKind) -> ProviderPreset {
    ProviderPreset { id: "test".into(), label: "Test".into(), endpoint: endpoint.into(),
        api_kind: kind, default_model: "m".into(), needs_key: true }
}

#[tokio::test]
async fn openai_chat_success() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "你好"}}]
        })))
        .mount(&server).await;
    let p = preset(&server.uri(), ApiKind::OpenAIChat);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hello", "auto", "zh", &Default::default());
    let params = WireParams { model: "gpt-4o-mini".into(), temperature: None, max_tokens: None, stream: false };
    let out = call(&client, &p, "sk-x", &params, &sys, &usr).await.unwrap();
    assert_eq!(out, "你好");
}

#[tokio::test]
async fn http_401_is_config_error() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server).await;
    let p = preset(&server.uri(), ApiKind::OpenAIChat);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hi", "en", "zh", &Default::default());
    let params = WireParams { model: "m".into(), temperature: None, max_tokens: None, stream: false };
    let err = call(&client, &p, "bad", &params, &sys, &usr).await.unwrap_err();
    assert!(matches!(err, linguaray_lib::error::Error::Config(_)));
}

#[tokio::test]
async fn http_429_is_fallback_eligible() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server).await;
    let p = preset(&server.uri(), ApiKind::OpenAIChat);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hi", "en", "zh", &Default::default());
    let params = WireParams { model: "m".into(), temperature: None, max_tokens: None, stream: false };
    let err = call(&client, &p, "k", &params, &sys, &usr).await.unwrap_err();
    assert!(matches!(err, linguaray_lib::error::Error::FallbackEligible(_)));
}

#[tokio::test]
async fn http_404_is_config_not_fallback() {
    // §G: a 4xx other than 401/403 must be Config(InvalidRequest), NOT
    // FallbackEligible — otherwise an invalid model/endpoint needlessly sends
    // the text to a 2nd provider.
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server).await;
    let p = preset(&server.uri(), ApiKind::OpenAIChat);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hi", "en", "zh", &Default::default());
    let params = WireParams { model: "m".into(), temperature: None, max_tokens: None, stream: false };
    let err = call(&client, &p, "k", &params, &sys, &usr).await.unwrap_err();
    match err {
        linguaray_lib::error::Error::Config(_) => { /* correct */ }
        other => panic!("expected Config, got {:?}", other),
    }
}
