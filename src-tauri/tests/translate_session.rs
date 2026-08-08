//! R2a Task 4: translate_session 核心（run_translate_session）集成测试。
//!
//! 直接测纯逻辑函数 `run_translate_session(db, client, keystore, app, ...)`，
//! 跳过 Tauri State/AppHandle 装配（那部分由类型签名保证）。
//! 复用 provider_crud 的 fresh_db 模式 + translate_parallel 的 wiremock 模式。

use linguaray_lib::db::providers as db_providers;
use linguaray_lib::db::schema;
use linguaray_lib::db::Database;
use serde_json::json;
use tempfile::tempdir;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fresh_db() -> (tempfile::TempDir, Database) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).unwrap();
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .unwrap();
    (dir, db)
}

fn direct_client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

fn empty_keystore() -> linguaray_lib::keystore::Keystore {
    let dir = tempfile::tempdir().unwrap().keep();
    linguaray_lib::keystore::Keystore::new(dir).unwrap()
}

async fn mount_ok(server: &MockServer, body: &str) {
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": body}}]
        })))
        .mount(server)
        .await;
}

/// 最小 AppHandle 替身：run_translate_session 只用它做 settings::load，
/// 而 settings 在测试环境下会回退到默认值（fallback_engine=None）。
/// 这里传一个未初始化 plugin-store 的 handle 也能让 settings::load 返回默认。
/// 若 settings::load 强依赖 plugin-store，则改用一个不读 settings 的入口
/// （见下方 run_translate_session_with_fallback 测试变体）。
#[allow(dead_code)]
fn test_app_handle() -> tauri::AppHandle {
    // tauri::test 不可用；run_translate_session 接收 &AppHandle 仅用于 settings::load。
    // 我们用一个 helper 入口避开 AppHandle（见 lib.rs 的 pub fn run_translate_session_no_settings）。
    panic!("use run_translate_session_no_settings in tests")
}

#[tokio::test]
async fn single_engine_path_primary_only() {
    // preferences 只设 primary（parallel 为空）→ 单引擎路径，actual_engine=Some。
    let (_dir, db) = fresh_db();
    let server = MockServer::start().await;
    let port: u16 = server.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_ok(&server, "单引擎结果").await;

    // 建一个 needs_key=false 的 openai profile 指向 mock，设为 primary。
    // endpoint 用 https 通过 validate_endpoint，再直接改库指向 mock 的 http 地址
    // （validate_endpoint 只在 create/update 跑，list/read 不复验——§Privacy 校验在写时）。
    let p = db
        .with_conn(|conn| {
            let p = db_providers::create(
                conn,
                "openai",
                "OpenAI",
                "https://api.openai.com/v1/chat/completions",
                None,
            )?;
            // 改 endpoint 指向 mock（lvh.me 非 loopback，避免 is_local 误判为 local-sacred）。
            conn.execute(
                "UPDATE providers SET endpoint=?1, needs_key=0 WHERE uuid=?2",
                rusqlite::params![format!("http://lvh.me:{port}/v1/chat/completions"), p.uuid],
            )?;
            // 设为 primary。
            conn.execute(
                "UPDATE preferences SET primary_uuid=?1 WHERE id=1",
                rusqlite::params![p.uuid],
            )?;
            Ok(p)
        })
        .unwrap();

    let client = direct_client();
    let keystore = empty_keystore();
    let db_arc = std::sync::Arc::new(db);

    let result = linguaray_lib::run_translate_session_no_settings(
        &db_arc, &client, &keystore, "hello", "auto", "zh",
    )
    .await
    .expect("single engine ok");

    assert_eq!(result.outcomes.len(), 1);
    let o = &result.outcomes[0];
    assert_eq!(o.uuid, p.uuid);
    let t = o.result.as_ref().expect("ok");
    assert_eq!(t.text, "单引擎结果");
    assert_eq!(result.actual_engine.as_deref(), Some(t.engine.as_str()));
}

#[tokio::test]
async fn parallel_path_two_engines() {
    // primary + 1 parallel → 并行路径，actual_engine=None。
    let (_dir, db) = fresh_db();
    let s1 = MockServer::start().await;
    let s2 = MockServer::start().await;
    let port1: u16 = s1.uri().rsplit(':').next().unwrap().parse().unwrap();
    let port2: u16 = s2.uri().rsplit(':').next().unwrap().parse().unwrap();
    mount_ok(&s1, "结果1").await;
    mount_ok(&s2, "结果2").await;

    let (uuid1, uuid2) = db
        .with_conn(|conn| {
            let p1 = db_providers::create(
                conn, "openai", "A",
                "https://api.openai.com/v1/chat/completions", None,
            )?;
            let p2 = db_providers::create(
                conn, "gemini", "B",
                "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions", None,
            )?;
            // 改 endpoint 指向各自 mock（lvh.me 非 loopback，避免 is_local 误判）+ 关 needs_key。
            conn.execute(
                "UPDATE providers SET endpoint=?1, needs_key=0 WHERE uuid=?2",
                rusqlite::params![format!("http://lvh.me:{port1}/v1/chat/completions"), p1.uuid],
            )?;
            conn.execute(
                "UPDATE providers SET endpoint=?1, needs_key=0 WHERE uuid=?2",
                rusqlite::params![format!("http://lvh.me:{port2}/v1/chat/completions"), p2.uuid],
            )?;
            // primary=p1, parallel=[p2]。
            conn.execute(
                "UPDATE preferences SET primary_uuid=?1, parallel_uuids=?2 WHERE id=1",
                rusqlite::params![p1.uuid, serde_json::to_string(&[&p2.uuid]).unwrap()],
            )?;
            Ok((p1.uuid, p2.uuid))
        })
        .unwrap();

    let client = direct_client();
    let keystore = empty_keystore();
    let db_arc = std::sync::Arc::new(db);

    let result = linguaray_lib::run_translate_session_no_settings(
        &db_arc, &client, &keystore, "hi", "auto", "zh",
    )
    .await
    .expect("parallel ok");

    assert_eq!(result.outcomes.len(), 2);
    assert!(result.actual_engine.is_none(), "parallel → actual_engine None");
    let mut by_uuid = std::collections::HashMap::new();
    for o in result.outcomes {
        by_uuid.insert(o.uuid, o.result.unwrap().text);
    }
    assert_eq!(by_uuid.get(&uuid1).map(String::as_str), Some("结果1"));
    assert_eq!(by_uuid.get(&uuid2).map(String::as_str), Some("结果2"));
}

#[tokio::test]
async fn no_active_selection_errors() {
    let (_dir, db) = fresh_db();
    let client = direct_client();
    let keystore = empty_keystore();
    let db_arc = std::sync::Arc::new(db);
    let err = linguaray_lib::run_translate_session_no_settings(
        &db_arc, &client, &keystore, "x", "auto", "zh",
    )
    .await
    .unwrap_err();
    assert!(err.contains("no active provider"), "got: {err}");
}

#[tokio::test]
async fn disabled_primary_is_skipped_even_if_selected() {
    // primary 被选中但 enabled=false：过滤后无可用 profile → 错误
    // （validate_active_selection 在 set_active 时拦截，但运行时也兜底）。
    // 本测试不触达网络（profile 在调用前被过滤），故不挂 mock。
    let (_dir, db) = fresh_db();

    let p = db
        .with_conn(|conn| {
            let p = db_providers::create(
                conn, "openai", "A",
                "https://api.openai.com/v1/chat/completions", None,
            )?;
            // enabled=0 → 过滤后无可用 profile。endpoint 不重要（不触达），保持 https 占位。
            conn.execute(
                "UPDATE providers SET needs_key=0, enabled=0 WHERE uuid=?1",
                rusqlite::params![p.uuid],
            )?;
            conn.execute("UPDATE preferences SET primary_uuid=?1 WHERE id=1", rusqlite::params![p.uuid])?;
            Ok(p.uuid)
        })
        .unwrap();
    let _ = p;

    let client = direct_client();
    let keystore = empty_keystore();
    let db_arc = std::sync::Arc::new(db);
    let err = linguaray_lib::run_translate_session_no_settings(
        &db_arc, &client, &keystore, "x", "auto", "zh",
    )
    .await
    .unwrap_err();
    assert!(err.contains("no active provider"), "disabled primary must be filtered out: {err}");
}
