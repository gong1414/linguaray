use islandpot_lib::engines::google::Google;
use islandpot_lib::engines::TraditionalEngine;
use wiremock::{MockServer, Mock, ResponseTemplate};
use serde_json::json;

#[tokio::test]
async fn google_parses_nested_response() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            [ ["你好","hello",null,null,1], ["世界","world",null,null,1] ],
            null, "en", null, null, null, 1.0, []
        ])))
        .mount(&server).await;
    let eng = Google::with_base(server.uri());
    let client = reqwest::Client::new();
    let out = eng.translate(&client, "hello world", "auto", "zh").await.unwrap();
    assert_eq!(out, "你好世界");
}

#[tokio::test]
async fn google_500_is_fallback_eligible() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server).await;
    let eng = Google::with_base(server.uri());
    let client = reqwest::Client::new();
    let err = eng.translate(&client, "hi", "auto", "zh").await.unwrap_err();
    assert!(matches!(err, islandpot_lib::error::Error::FallbackEligible(_)));
}
