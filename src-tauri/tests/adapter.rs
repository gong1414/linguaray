//! adapter.rs — Protocol→ApiKind + ProviderProfile→ProviderPreset 适配器测试。
//! 纯函数转换，无 IO、无网络。

use linguaray_lib::adapter::{profile_to_preset, protocol_to_api_kind};
use linguaray_lib::db::providers::{Protocol, ProviderProfile, ProviderCapabilities};
use linguaray_lib::wire::ApiKind;

fn profile(uuid: &str, protocol: Protocol, endpoint: &str, secret_ref: &str) -> ProviderProfile {
    ProviderProfile {
        uuid: uuid.into(),
        template_id: "openai".into(),
        name: format!("Name-{uuid}"),
        protocol,
        endpoint: endpoint.into(),
        model: Some("gpt-4o-mini".into()),
        enabled: true,
        sort_order: 0,
        is_local: false,
        needs_key: true,
        secret_ref: secret_ref.into(),
        capabilities: ProviderCapabilities::default(),
        status: "active".into(),
        version: 1,
    }
}

#[test]
fn protocol_to_api_kind_maps_ai_protocols() {
    assert_eq!(protocol_to_api_kind(&Protocol::OpenaiChat), Some(ApiKind::OpenAIChat));
    assert_eq!(protocol_to_api_kind(&Protocol::Gemini), Some(ApiKind::OpenAIChat));
    assert_eq!(protocol_to_api_kind(&Protocol::Anthropic), Some(ApiKind::Anthropic));
}

#[test]
fn protocol_to_api_kind_returns_none_for_non_ai() {
    assert_eq!(protocol_to_api_kind(&Protocol::GoogleTranslate), None);
    assert_eq!(protocol_to_api_kind(&Protocol::CustomHttp), None);
}

#[test]
fn profile_to_preset_openai_chat() {
    let p = profile("u1", Protocol::OpenaiChat, "https://api.openai.com/v1/chat/completions", "provider/u1");
    let preset = profile_to_preset(&p).expect("openai_chat → preset");
    assert_eq!(preset.id, "provider/u1", "preset.id MUST be secret_ref so keystore.get_key hits the right key");
    assert_eq!(preset.label, "Name-u1");
    assert_eq!(preset.endpoint, "https://api.openai.com/v1/chat/completions");
    assert_eq!(preset.api_kind, ApiKind::OpenAIChat);
    assert_eq!(preset.default_model, "gpt-4o-mini");
    assert!(preset.needs_key);
}

#[test]
fn profile_to_preset_anthropic() {
    let p = profile("u2", Protocol::Anthropic, "https://api.anthropic.com/v1/messages", "provider/u2");
    let preset = profile_to_preset(&p).expect("anthropic → preset");
    assert_eq!(preset.api_kind, ApiKind::Anthropic);
    assert_eq!(preset.id, "provider/u2");
}

#[test]
fn profile_to_preset_gemini_maps_to_openai_chat() {
    // Gemini 走 OpenAI-compatible 路径（spec §Wire），与 preset catalog 一致。
    let p = profile("u3", Protocol::Gemini, "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions", "provider/u3");
    let preset = profile_to_preset(&p).expect("gemini → preset");
    assert_eq!(preset.api_kind, ApiKind::OpenAIChat);
}

#[test]
fn profile_to_preset_uses_empty_string_when_model_none() {
    let mut p = profile("u4", Protocol::OpenaiChat, "https://api.openai.com/v1/chat/completions", "provider/u4");
    p.model = None;
    let preset = profile_to_preset(&p).expect("None model → empty string default");
    assert_eq!(preset.default_model, "");
}

#[test]
fn profile_to_preset_rejects_google_translate() {
    let p = profile("u5", Protocol::GoogleTranslate, "https://translate.google.com", "provider/u5");
    let err = profile_to_preset(&p).expect_err("google_translate is not an AI protocol");
    assert!(err.contains("unsupported protocol"), "got: {err}");
}

#[test]
fn profile_to_preset_rejects_custom_http() {
    let p = profile("u6", Protocol::CustomHttp, "https://example.com", "provider/u6");
    assert!(profile_to_preset(&p).is_err());
}

#[test]
fn profile_to_preset_needs_key_false_propagates() {
    // Ollama（keyless）：needs_key=false 必须透传，否则 translate 会去 keystore 找不存在的 key。
    let mut p = profile("u7", Protocol::OpenaiChat, "http://localhost:11434/v1/chat/completions", "provider/u7");
    p.needs_key = false;
    let preset = profile_to_preset(&p).expect("ok");
    assert!(!preset.needs_key);
}
