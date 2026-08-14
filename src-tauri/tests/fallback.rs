//! §G classified fallback — `service::translate_with_fallback`.
//!
//! Three branches, one test each:
//!  1. `fallback_eligible_tries_fallback` — primary returns FallbackEligible (500);
//!     fallback (injected FakeFallback) runs and the WHOLE text reaches it.
//!  2. `config_error_does_not_fallback` — primary returns Config (401); fallback
//!     must NOT be called, Config propagates.
//!  3. `local_primary_no_remote_fallback` — primary is LOCAL (loopback); on
//!     FallbackEligible we must NOT degrade to a remote fallback (§G local-sacred);
//!     result is `LocalNoFallback` and the fallback is not called.
//!
//! Test harness notes
//! - wiremock binds 127.0.0.1, so the primary's loopback literal would itself be
//!   classified "local" by `is_local`. To exercise the REMOTE-primary branch
//!   faithfully we point the primary at `http://lvh.me:{port}` — a public wildcard
//!   DNS name that resolves to 127.0.0.1 and so reaches the mock, but is NOT a
//!   loopback literal, so `is_local` correctly returns false. The local-sacred
//!   test instead uses `http://localhost:{port}`, which `is_local` matches.
//! - The reqwest client is built with `.no_proxy()` so connection is direct and
//!   environment-independent (no reliance on a dev proxy to redirect lvh.me).
//! - The primary preset uses `needs_key: false` so `service::translate` skips the
//!   keystore and goes straight to the HTTP mock; keystore behavior is covered by
//!   tests/keystore.rs. This keeps these tests focused on the §G branches.

use async_trait::async_trait;
use linguaray_lib::engines::TraditionalEngine;
use linguaray_lib::error::Error;
use linguaray_lib::providers::ProviderPreset;
use linguaray_lib::service::{translate_with_fallback, TranslateInput};
use linguaray_lib::wire::ApiKind;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A fake fallback that records whether it was called, captures the text it was
/// given, and returns `[fb]<text>` (so tests assert BOTH that it ran AND that it
/// saw the full request text — i.e. no chunk mixing).
struct FakeFallback {
    called: Arc<AtomicBool>,
    seen_text: Arc<std::sync::Mutex<Option<String>>>,
}

#[async_trait]
impl TraditionalEngine for FakeFallback {
    fn id(&self) -> &str { "fake" }
    fn label(&self) -> &str { "Fake" }
    async fn translate(
        &self,
        _client: &reqwest::Client,
        text: &str,
        _from: &str,
        _to: &str,
    ) -> Result<String, Error> {
        self.called.store(true, Ordering::SeqCst);
        *self.seen_text.lock().unwrap() = Some(text.to_string());
        Ok(format!("[fb]{text}"))
    }
}

/// A counter so the "must NOT fall back" tests can assert the fallback path was
/// never even indirectly reached. wiremock's `received_requests()` is async.
async fn request_count(server: &MockServer) -> u32 {
    server
        .received_requests()
        .await
        .map(|v| v.len() as u32)
        .unwrap_or(0)
}

/// Build a primary preset. `needs_key: false` → `translate` skips the keystore and
/// calls the mock directly. The host controls local-vs-remote classification.
fn primary_preset(endpoint: &str) -> ProviderPreset {
    ProviderPreset {
        id: "test-primary".into(),
        label: "Test Primary".into(),
        endpoint: endpoint.into(),
        api_kind: ApiKind::OpenAIChat,
        default_model: "m".into(),
        needs_key: false,
        auth: linguaray_contracts::AuthKind::Bearer,
    }
}

/// Mount a single mock on the server responding with the given status / body.
/// For a 500 we return an empty body (status alone classifies as FallbackEligible).
/// For a 401 likewise (status alone → Config(AuthFailed)).
async fn mount_status(server: &MockServer, status: u16) {
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(status))
        .mount(server)
        .await;
}

/// Direct (no-proxy) client so the lvh.me connection does not depend on a dev proxy.
fn direct_client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

/// An empty keystore over a temp dir. `needs_key: false` presets never touch it,
/// but `translate_with_fallback`'s signature requires one.
fn empty_keystore() -> linguaray_lib::keystore::Keystore {
    let dir = tempfile::tempdir().unwrap().keep();
    linguaray_lib::keystore::Keystore::new(dir).unwrap()
}

#[tokio::test]
async fn fallback_eligible_tries_fallback() {
    // Primary returns 500 → FallbackEligible. Use lvh.me (resolves to 127.0.0.1,
    // is NOT a loopback literal) so is_local() is false and the fallback branch runs.
    let server = MockServer::start().await;
    let port: u16 = server.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_status(&server, 500).await;

    let preset = primary_preset(&format!("http://lvh.me:{port}/v1/chat/completions"));
    let client = direct_client();
    let keystore = empty_keystore();

    let called = Arc::new(AtomicBool::new(false));
    let seen_text = Arc::new(std::sync::Mutex::new(None));
    let fallback = Box::new(FakeFallback {
        called: called.clone(),
        seen_text: seen_text.clone(),
    });

    let input = TranslateInput {
        text: "hello world",
        from: "auto",
        to: "zh",
        options: Default::default(),
    };
    let out = translate_with_fallback(&client, &keystore, &preset, input, Some(fallback))
        .await
        .expect("fallback should succeed");

    // Fallback ran and returned its marker, tagged with the FALLBACK engine id.
    assert_eq!(out.text, "[fb]hello world");
    assert_eq!(out.engine, "fake", "result tagged with the fallback engine id, not primary");
    assert!(called.load(Ordering::SeqCst), "fallback engine must be called");
    // Whole-text fallback (no chunk mixing): it saw the entire input verbatim.
    assert_eq!(
        seen_text.lock().unwrap().as_deref(),
        Some("hello world"),
        "fallback must receive the WHOLE request text"
    );
}

#[tokio::test]
async fn config_error_does_not_fallback() {
    // Primary returns 401 → Config(AuthFailed). §G: Config propagates; fallback
    // must NOT be called.
    let server = MockServer::start().await;
    let port: u16 = server.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_status(&server, 401).await;

    let preset = primary_preset(&format!("http://lvh.me:{port}/v1/chat/completions"));
    let client = direct_client();
    let keystore = empty_keystore();

    let called = Arc::new(AtomicBool::new(false));
    let seen_text = Arc::new(std::sync::Mutex::new(None));
    let fallback = Box::new(FakeFallback {
        called: called.clone(),
        seen_text: seen_text.clone(),
    });

    let input = TranslateInput {
        text: "secret stuff",
        from: "en",
        to: "zh",
        options: Default::default(),
    };
    let err = translate_with_fallback(&client, &keystore, &preset, input, Some(fallback))
        .await
        .expect_err("Config error must propagate");

    assert!(
        matches!(err, Error::Config(_)),
        "expected Config error to propagate, got {err:?}"
    );
    assert!(
        !called.load(Ordering::SeqCst),
        "fallback engine must NOT be called on a Config error"
    );
    assert!(
        seen_text.lock().unwrap().is_none(),
        "fallback must not have seen any text"
    );
    // Sanity: exactly one primary request (no fallback-induced second call).
    assert_eq!(request_count(&server).await, 1);
}

#[tokio::test]
async fn local_primary_no_remote_fallback() {
    // Primary is LOCAL (localhost loopback). Even though it returns 500
    // (FallbackEligible) and a fallback IS configured, §G's local-sacred rule
    // forbids silently degrading a local AI engine to a remote fallback engine.
    let server = MockServer::start().await;
    let port: u16 = server.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_status(&server, 500).await;

    let preset = primary_preset(&format!("http://localhost:{port}/v1/chat/completions"));
    let client = direct_client();
    let keystore = empty_keystore();

    let called = Arc::new(AtomicBool::new(false));
    let seen_text = Arc::new(std::sync::Mutex::new(None));
    let fallback = Box::new(FakeFallback {
        called: called.clone(),
        seen_text: seen_text.clone(),
    });

    let input = TranslateInput {
        text: "local ai text",
        from: "auto",
        to: "zh",
        options: Default::default(),
    };
    let err = translate_with_fallback(&client, &keystore, &preset, input, Some(fallback))
        .await
        .expect_err("local primary must not fall back");

    assert!(
        matches!(err, Error::LocalNoFallback),
        "local-primary FallbackEligible must yield LocalNoFallback, got {err:?}"
    );
    assert!(
        !called.load(Ordering::SeqCst),
        "fallback engine must NOT be called for a local primary"
    );
    assert!(seen_text.lock().unwrap().is_none());
}

#[tokio::test]
async fn remote_primary_no_fallback_configured_yields_local_no_fallback() {
    // §G: opt-in default. Remote primary fails FallbackEligible, but the user has
    // NOT configured a fallback engine → LocalNoFallback (surfaced as "no fallback"
    // to the UI), not a silent network attempt.
    let server = MockServer::start().await;
    let port: u16 = server.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_status(&server, 500).await;

    let preset = primary_preset(&format!("http://lvh.me:{port}/v1/chat/completions"));
    let client = direct_client();
    let keystore = empty_keystore();

    let input = TranslateInput {
        text: "no fallback set",
        from: "auto",
        to: "zh",
        options: Default::default(),
    };
    let err = translate_with_fallback(&client, &keystore, &preset, input, None)
        .await
        .expect_err("no fallback configured → error");

    assert!(
        matches!(err, Error::LocalNoFallback),
        "remote primary with no fallback configured must yield LocalNoFallback, got {err:?}"
    );
}

#[tokio::test]
async fn primary_success_skips_fallback() {
    // Happy path: primary succeeds; fallback must not be touched.
    let server = MockServer::start().await;
    let port: u16 = server.uri().rsplit(':').next().unwrap().parse().unwrap();
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "你好"}}]
        })))
        .mount(&server)
        .await;

    let preset = primary_preset(&format!("http://lvh.me:{port}/v1/chat/completions"));
    let client = direct_client();
    let keystore = empty_keystore();

    let called = Arc::new(AtomicBool::new(false));
    let fallback = Box::new(FakeFallback {
        called: called.clone(),
        seen_text: Arc::new(std::sync::Mutex::new(None)),
    });

    let input = TranslateInput {
        text: "hi",
        from: "auto",
        to: "zh",
        options: Default::default(),
    };
    let out = translate_with_fallback(&client, &keystore, &preset, input, Some(fallback))
        .await
        .expect("primary success");
    assert_eq!(out.text, "你好");
    assert_eq!(out.engine, preset.id, "primary success tagged with primary id");
    assert!(
        !called.load(Ordering::SeqCst),
        "fallback must not be called when the primary succeeds"
    );
}
