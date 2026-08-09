//! R2a Task 3: translate_parallel 并行编排测试。
//!
//! 三类场景：
//!  1. 2 引擎都成功 → 2 个 Ok outcome，engine 字段是各自 preset.id（=secret_ref）
//!  2. 1 成功 1 失败（500）→ 1 Ok + 1 Err(FallbackEligible)；无 fallback 配置时
//!     失败的那个走 §G：remote primary + no fallback → LocalNoFallback
//!  3. 全部失败（两个 500，无 fallback）→ 2 个 Err(LocalNoFallback)
//!
//! 复用 fallback.rs 的测试 harness 风格：wiremock + lvh.me（非 loopback literal，
//! 避免 is_local 误判）+ no_proxy client + needs_key:false preset（跳过 keystore）。

use linguaray_lib::adapter::profile_to_preset;
use linguaray_lib::db::providers::{Protocol, ProviderProfile, ProviderCapabilities};
use linguaray_lib::error::Error;
use linguaray_lib::service::{translate_parallel, TranslationOutcome};
use linguaray_lib::wire::AppOptions;
use serde_json::json;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn direct_client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

fn empty_keystore() -> linguaray_lib::keystore::Keystore {
    let dir = tempfile::tempdir().unwrap().keep();
    linguaray_lib::keystore::Keystore::new(dir).unwrap()
}

/// 构造一个 needs_key=false 的 profile（translate 跳过 keystore，直接打 mock）。
fn profile(uuid: &str, endpoint: &str) -> ProviderProfile {
    ProviderProfile {
        uuid: uuid.into(),
        template_id: "openai".into(),
        name: format!("P-{uuid}"),
        protocol: Protocol::OpenaiChat,
        endpoint: endpoint.into(),
        model: Some("m".into()),
        enabled: true,
        sort_order: 0,
        is_local: false,
        needs_key: false,
        secret_ref: format!("provider/{uuid}"),
        capabilities: ProviderCapabilities::default(),
        status: "active".into(),
    }
}

/// 构造一个 `profile_to_preset` 会拒绝的 profile（google_translate 协议→None）。
/// 用于 B5 顺序测试：pre-failed entry 必须留在原输入位置，不能浮到 ready 之前。
fn profile_unsupported(uuid: &str) -> ProviderProfile {
    let mut p = profile(uuid, "https://translate.google.com");
    p.protocol = Protocol::GoogleTranslate;
    // 自洽性：确认 adapter 确实拒绝它。
    assert!(profile_to_preset(&p).is_err());
    p
}

async fn mount_ok(server: &MockServer, body: &str) {
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": body}}]
        })))
        .mount(server)
        .await;
}

async fn mount_500(server: &MockServer) {
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .mount(server)
        .await;
}

/// 把 Vec<TranslationOutcome> 按 uuid 排序后返回，断言时与输入顺序解耦（并发完成顺序不定）。
fn sorted_by_uuid(mut v: Vec<TranslationOutcome>) -> Vec<TranslationOutcome> {
    v.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    v
}

#[tokio::test]
async fn two_engines_both_success() {
    let s1 = MockServer::start().await;
    let s2 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_ok(&s1, "你好").await;
    mount_ok(&s2, "您好").await;

    let profiles = vec![
        profile("u1", &format!("http://lvh.me:{port1}/v1/chat/completions")),
        profile("u2", &format!("http://lvh.me:{port2}/v1/chat/completions")),
    ];
    let client = direct_client();
    let keystore = empty_keystore();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "hello", "auto", "zh",
        AppOptions::default(), None,
    )
    .await;
    assert_eq!(outcomes.len(), 2, "exactly one outcome per profile");

    let mut by_uuid = std::collections::HashMap::new();
    for o in outcomes {
        by_uuid.insert(o.uuid, o.result);
    }
    let r1 = by_uuid.remove("u1").unwrap().expect("u1 ok");
    assert_eq!(r1.text, "你好");
    assert_eq!(r1.engine, "provider/u1", "engine tagged with preset.id (=secret_ref)");
    let r2 = by_uuid.remove("u2").unwrap().expect("u2 ok");
    assert_eq!(r2.text, "您好");
    assert_eq!(r2.engine, "provider/u2");
}

#[tokio::test]
async fn one_success_one_failure_no_fallback() {
    let s1 = MockServer::start().await;
    let s2 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_ok(&s1, "ok-text").await;
    mount_500(&s2).await;

    let profiles = vec![
        profile("u1", &format!("http://lvh.me:{port1}/v1/chat/completions")),
        profile("u2", &format!("http://lvh.me:{port2}/v1/chat/completions")),
    ];
    let client = direct_client();
    let keystore = empty_keystore();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), None,
    )
    .await;

    let mut by_uuid = std::collections::HashMap::new();
    for o in outcomes {
        by_uuid.insert(o.uuid, o.result);
    }
    let r1 = by_uuid.remove("u1").unwrap().expect("u1 ok");
    assert_eq!(r1.text, "ok-text");
    // §G：remote primary + 500 + no fallback → LocalNoFallback（不是裸 FallbackEligible）
    let err2 = by_uuid.remove("u2").unwrap().expect_err("u2 failed");
    assert!(
        matches!(err2, Error::LocalNoFallback),
        "expected LocalNoFallback (remote primary, no fallback configured), got {err2:?}"
    );
}

#[tokio::test]
async fn all_fail_no_fallback_yields_all_local_no_fallback() {
    let s1 = MockServer::start().await;
    let s2 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_500(&s1).await;
    mount_500(&s2).await;

    let profiles = vec![
        profile("u1", &format!("http://lvh.me:{port1}/v1/chat/completions")),
        profile("u2", &format!("http://lvh.me:{port2}/v1/chat/completions")),
    ];
    let client = direct_client();
    let keystore = empty_keystore();

    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), None,
    )
    .await;
    assert_eq!(outcomes.len(), 2);
    for o in sorted_by_uuid(outcomes) {
        assert!(
            matches!(o.result, Err(Error::LocalNoFallback)),
            "all engines failed with no fallback → LocalNoFallback each, got {:?} for {}",
            o.result, o.uuid
        );
    }
}

#[tokio::test]
async fn unsupported_protocol_profile_yields_config_error_outcome() {
    // google_translate 协议无法走 wire::call：profile_to_preset 失败 →
    // translate_parallel 必须把它标成 Err(Config::Unsupported)，而不是 panic 或丢弃。
    let mut p = profile("u-bad", "https://translate.google.com");
    p.protocol = Protocol::GoogleTranslate;
    // 先确认 adapter 确实拒绝它（测试自洽性）。
    assert!(profile_to_preset(&p).is_err());

    let profiles = vec![p];
    let client = direct_client();
    let keystore = empty_keystore();
    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), None,
    )
    .await;
    assert_eq!(outcomes.len(), 1);
    match &outcomes[0].result {
        Err(Error::Config(c)) => {
            // ConfigKind::Unsupported；Display 含 "unsupported protocol"。
            let s = format!("{c}");
            assert!(s.contains("unsupported"), "got: {s}");
        }
        other => panic!("expected Config(Unsupported), got {other:?}"),
    }
    assert_eq!(outcomes[0].uuid, "u-bad");
}

#[tokio::test]
async fn empty_profiles_yields_empty_outcomes() {
    let client = direct_client();
    let keystore = empty_keystore();
    let outcomes = translate_parallel(
        &client, &keystore, vec![], "x", "auto", "zh",
        AppOptions::default(), None,
    )
    .await;
    assert!(outcomes.is_empty());
}

#[tokio::test]
async fn outcomes_preserve_input_order_with_pre_failed_middle() {
    let s1 = MockServer::start().await;
    let s3 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port3: u16 = s3.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_ok(&s1, "first").await;
    mount_ok(&s3, "third").await;
    let profiles = vec![
        profile("u1", &format!("http://lvh.me:{port1}/v1/chat/completions")),
        profile_unsupported("u2"),
        profile("u3", &format!("http://lvh.me:{port3}/v1/chat/completions")),
    ];
    let client = direct_client();
    let keystore = empty_keystore();
    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), None,
    ).await;
    let got: Vec<&str> = outcomes.iter().map(|o| o.uuid.as_str()).collect();
    assert_eq!(got, vec!["u1", "u2", "u3"],
        "outcomes must preserve STRICT input order including the pre-failed middle entry");
    assert!(outcomes[1].result.is_err(), "u2 must be the pre-failed entry");
}

#[tokio::test]
async fn ready_outcomes_preserve_input_order_under_completion_jitter() {
    use std::time::Duration;
    let s1 = MockServer::start().await;
    let s2 = MockServer::start().await;
    let s3 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port3: u16 = s3.uri().rsplit(':').next().unwrap().parse().unwrap();
    Mock::given(any()).respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "choices": [{"message": {"content": "slow"}}]
    })).set_delay(Duration::from_millis(150))).mount(&s1).await;
    mount_ok(&s2, "fast").await;
    Mock::given(any()).respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "choices": [{"message": {"content": "medium"}}]
    })).set_delay(Duration::from_millis(50))).mount(&s3).await;
    let profiles = vec![
        profile("u1", &format!("http://lvh.me:{port1}/v1/chat/completions")),
        profile("u2", &format!("http://lvh.me:{port2}/v1/chat/completions")),
        profile("u3", &format!("http://lvh.me:{port3}/v1/chat/completions")),
    ];
    let client = direct_client();
    let keystore = empty_keystore();
    let outcomes = translate_parallel(
        &client, &keystore, profiles, "x", "auto", "zh",
        AppOptions::default(), None,
    ).await;
    let got: Vec<&str> = outcomes.iter().map(|o| o.uuid.as_str()).collect();
    assert_eq!(got, vec!["u1", "u2", "u3"]);
}
