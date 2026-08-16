use linguaray_contracts::{AuthKind, ProtocolKind};
use linguaray_lib::providers::ProviderPreset;
use linguaray_lib::wire::{build_prompt, call, WireParams};
use serde_json::json;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn preset(endpoint: &str, protocol: ProtocolKind, auth: AuthKind) -> ProviderPreset {
    ProviderPreset {
        id: "test".into(),
        label: "Test".into(),
        endpoint: endpoint.into(),
        protocol,
        default_model: "m".into(),
        needs_key: true,
        auth,
    }
}

fn params() -> WireParams {
    WireParams {
        model: "m".into(),
        temperature: None,
        max_tokens: None,
        stream: false,
    }
}

async fn openai_ok(server: &MockServer) {
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "你好"}}]
        })))
        .mount(server)
        .await;
}

#[tokio::test]
async fn openai_chat_success() {
    let server = MockServer::start().await;
    openai_ok(&server).await;
    let p = preset(&server.uri(), ProtocolKind::OpenaiChat, AuthKind::Bearer);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hello", "auto", "zh", &Default::default());
    let out = call(&client, &p, "sk-x", &params(), &sys, &usr)
        .await
        .unwrap();
    assert_eq!(out, "你好");
}

#[tokio::test]
async fn anthropic_success_sends_x_api_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": [{"text": "bonjour"}]
        })))
        .mount(&server)
        .await;
    let p = preset(&server.uri(), ProtocolKind::Anthropic, AuthKind::Bearer);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hi", "en", "fr", &Default::default());
    let out = call(&client, &p, "sk-ant", &params(), &sys, &usr)
        .await
        .unwrap();
    assert_eq!(out, "bonjour");
    let reqs = server.received_requests().await.unwrap();
    let headers = &reqs[0].headers;
    assert_eq!(
        headers.get("x-api-key").map(|v| v.to_str().unwrap()),
        Some("sk-ant")
    );
    assert_eq!(
        headers
            .get("anthropic-version")
            .map(|v| v.to_str().unwrap()),
        Some("2023-06-01")
    );
}

#[tokio::test]
async fn gemini_uses_openai_chat_driver() {
    let server = MockServer::start().await;
    openai_ok(&server).await;
    let p = preset(&server.uri(), ProtocolKind::OpenaiChat, AuthKind::Bearer);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hello", "auto", "zh", &Default::default());
    let out = call(&client, &p, "sk-g", &params(), &sys, &usr)
        .await
        .unwrap();
    assert_eq!(out, "你好");
    let body: serde_json::Value =
        serde_json::from_slice(&server.received_requests().await.unwrap()[0].body).unwrap();
    assert_eq!(body["messages"][0]["role"], "system");
}

#[tokio::test]
async fn ollama_sends_no_auth_header() {
    let server = MockServer::start().await;
    openai_ok(&server).await;
    let mut p = preset(&server.uri(), ProtocolKind::OpenaiChat, AuthKind::None);
    p.needs_key = false;
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hi", "en", "zh", &Default::default());
    call(&client, &p, "", &params(), &sys, &usr).await.unwrap();
    let headers = &server.received_requests().await.unwrap()[0].headers;
    assert!(headers.get("authorization").is_none());
    assert!(headers.get("api-key").is_none());
    assert!(headers.get("x-api-key").is_none());
}

#[tokio::test]
async fn deepseek_preset_uses_openai_chat() {
    let server = MockServer::start().await;
    openai_ok(&server).await;
    let row = linguaray_catalog::get("deepseek").expect("catalog row");
    let p = ProviderPreset {
        id: row.id,
        label: row.label,
        endpoint: server.uri(),
        protocol: row.protocol,
        default_model: row.default_model,
        needs_key: row.needs_key,
        auth: row.auth,
    };
    assert_eq!(p.protocol, ProtocolKind::OpenaiChat);
    assert_eq!(p.auth, AuthKind::Bearer);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hi", "en", "zh", &Default::default());
    let out = call(&client, &p, "sk-ds", &params(), &sys, &usr)
        .await
        .unwrap();
    assert_eq!(out, "你好");
}

#[tokio::test]
async fn xiaomi_and_azure_send_api_key_not_authorization() {
    for id in ["xiaomi-mimo", "azure-openai"] {
        let server = MockServer::start().await;
        openai_ok(&server).await;
        let row = linguaray_catalog::get(id).expect(id);
        assert_eq!(row.auth, AuthKind::AzureKey, "{id} catalog auth");
        assert_eq!(row.protocol, ProtocolKind::OpenaiChat, "{id} protocol");
        let p = ProviderPreset {
            id: row.id,
            label: row.label,
            endpoint: server.uri(),
            protocol: row.protocol,
            default_model: row.default_model,
            needs_key: true,
            auth: row.auth,
        };
        let client = reqwest::Client::new();
        let (sys, usr) = build_prompt("hi", "en", "zh", &Default::default());
        call(&client, &p, "sk-az", &params(), &sys, &usr)
            .await
            .unwrap();
        let headers = &server.received_requests().await.unwrap()[0].headers;
        assert_eq!(
            headers.get("api-key").map(|v| v.to_str().unwrap()),
            Some("sk-az"),
            "{id} must send api-key"
        );
        assert!(
            headers.get("authorization").is_none(),
            "{id} must not send Authorization"
        );
    }
}

#[tokio::test]
async fn http_401_is_config_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;
    let p = preset(&server.uri(), ProtocolKind::OpenaiChat, AuthKind::Bearer);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hi", "en", "zh", &Default::default());
    let err = call(&client, &p, "bad", &params(), &sys, &usr)
        .await
        .unwrap_err();
    assert!(matches!(err, linguaray_lib::error::Error::Config(_)));
}

#[tokio::test]
async fn http_429_is_fallback_eligible() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;
    let p = preset(&server.uri(), ProtocolKind::OpenaiChat, AuthKind::Bearer);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hi", "en", "zh", &Default::default());
    let err = call(&client, &p, "k", &params(), &sys, &usr)
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        linguaray_lib::error::Error::FallbackEligible(_)
    ));
}

#[tokio::test]
async fn http_404_is_config_not_fallback() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    let p = preset(&server.uri(), ProtocolKind::OpenaiChat, AuthKind::Bearer);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hi", "en", "zh", &Default::default());
    let err = call(&client, &p, "k", &params(), &sys, &usr)
        .await
        .unwrap_err();
    match err {
        linguaray_lib::error::Error::Config(_) => {}
        other => panic!("expected Config, got {:?}", other),
    }
}
