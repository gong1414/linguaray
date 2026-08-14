use linguaray_lib::engines::TraditionalEngine;
use linguaray_lib::plugins::drivers::traditional::{Deepl, Google};
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn google_parses_nested_response() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::query_param("client", "gtx"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            [["你好","hello"],["世界","world"]],
            null, "en"
        ])))
        .mount(&server)
        .await;
    let eng = Google::with_origin(server.uri());
    let client = reqwest::Client::new();
    let out = eng
        .translate(&client, "hello world", "auto", "zh", None)
        .await
        .unwrap();
    assert_eq!(out, "你好世界");
}

#[tokio::test]
async fn google_500_is_fallback_eligible() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let eng = Google::with_origin(server.uri());
    let client = reqwest::Client::new();
    let err = eng
        .translate(&client, "hi", "auto", "zh", None)
        .await
        .unwrap_err();
    assert!(matches!(err, linguaray_lib::error::Error::FallbackEligible(_)));
}

#[tokio::test]
async fn deepl_parses_official_json_and_sends_auth_header() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::header(
            "Authorization",
            "DeepL-Auth-Key test-key:fx",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "translations": [{"detected_source_language":"EN","text":"Hallo"}]
        })))
        .mount(&server)
        .await;
    let eng = Deepl::with_endpoint(format!("{}/v2/translate", server.uri()));
    let client = reqwest::Client::new();
    let out = eng
        .translate(&client, "Hello", "en", "de", Some("test-key:fx"))
        .await
        .unwrap();
    assert_eq!(out, "Hallo");
}
