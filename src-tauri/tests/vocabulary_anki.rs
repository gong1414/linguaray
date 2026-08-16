use linguaray_lib::vocabulary::{self, VocabularyItem};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_item(word: &str, def: &str) -> VocabularyItem {
    VocabularyItem {
        item_uuid: "test".into(),
        timestamp: 1_700_000_100,
        source_language: "en".into(),
        target_language: "zh".into(),
        word: word.into(),
        definition: def.into(),
    }
}

#[tokio::test]
async fn anki_export_posts_correct_body_and_validates_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": 1,
            "error": null
        })))
        .expect(1)
        .mount(&server)
        .await;

    let items = vec![make_item("hello", "你好")];
    let result = vocabulary::export_anki_from_items_url(&items, "LinguaRay", &server.uri()).await;
    assert!(result.is_ok(), "{result:?}");
}

#[tokio::test]
async fn anki_export_returns_error_when_anki_returns_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": "deck not found"
        })))
        .mount(&server)
        .await;

    let items = vec![make_item("hello", "你好")];
    let result = vocabulary::export_anki_from_items_url(&items, "LinguaRay", &server.uri()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deck not found"));
}

#[tokio::test]
async fn anki_export_rejects_redirect() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(301).insert_header("Location", "http://evil.com"))
        .mount(&server)
        .await;

    let items = vec![make_item("hello", "你好")];
    let result = vocabulary::export_anki_from_items_url(&items, "LinguaRay", &server.uri()).await;
    assert!(result.is_err());
}
