//! Opt-in authenticated smoke. Default `cargo test` does not hit the network.
//!
//!   LINGUARAY_SMOKE=1 LINGUARAY_SMOKE_KEY_OPENAI=sk-... \
//!     cargo test -p linguaray --test catalog_smoke -- --ignored --nocapture

use linguaray_catalog::load;
use linguaray_contracts::AuthKind;
use linguaray_lib::providers::ProviderPreset;
use linguaray_lib::wire::{build_prompt, call, AppOptions, WireParams};

#[tokio::test]
#[ignore]
async fn smoke_rows_with_env_keys() {
    if std::env::var("LINGUARAY_SMOKE").ok().as_deref() != Some("1") {
        return;
    }
    let file = load().expect("catalog");
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap();
    let (system, user) = build_prompt("ping", "auto", "zh", &AppOptions::default());
    for row in file.providers {
        if row.endpoint.is_empty() {
            continue;
        }
        let env_name = format!(
            "LINGUARAY_SMOKE_KEY_{}",
            row.id.to_uppercase().replace('-', "_")
        );
        let Ok(key) = std::env::var(&env_name) else {
            continue;
        };
        let preset = ProviderPreset {
            id: row.id.clone(),
            label: row.label.clone(),
            endpoint: row.endpoint.clone(),
            protocol: row.protocol,
            default_model: row.default_model.clone(),
            needs_key: row.needs_key,
            auth: row.auth,
        };
        let params = WireParams {
            model: row.default_model.clone(),
            temperature: None,
            max_tokens: Some(16),
            stream: false,
        };
        let key_ref = if row.auth == AuthKind::None { "" } else { &key };
        match call(&client, &preset, key_ref, &params, &system, &user).await {
            Ok(text) => eprintln!("{}: ok ({} chars)", row.id, text.len()),
            Err(e) => eprintln!("{}: {e}", row.id),
        }
    }
}
