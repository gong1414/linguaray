//! DB-backed provider → wire-layer preset 适配器（R2a）。
//!
//! DB 的 `ProviderProfile`（运行时用户配置）与 wire 层的 `ProviderPreset`
//! （HTTP 调用模板）是两个独立类型：前者带 uuid/protocol/capabilities，
//! 后者是 Driver 调用的输入。本模块做无 IO 的纯转换，让并行翻译编排器
//! 可以复用既有的 `service::translate_with_fallback` + `wire::call`。
//!
//! 关键不变量：`preset.id = profile.secret_ref`（不是 uuid），这样
//! `service::translate` 内部的 `keystore.get_key(&preset.id)` 自动命中
//! profile 的 secret_ref，无需修改 translate 的鉴权逻辑。

use crate::db::providers::{Protocol, ProviderProfile};
use crate::providers::ProviderPreset;
use linguaray_contracts::{AuthKind, ProtocolKind};

/// 把 DB wire 协议族映射到可调用的 Driver 协议。
/// - OpenAIChat / Gemini → OpenaiChat（spec：Gemini 走 openai-chat Driver）
/// - Anthropic           → Anthropic
/// - GoogleTranslate / CustomHttp / 传统 MT → None
///   （传统协议直到 PR-6f Drivers 才可调用；本 PR 只开 CHECK）
pub fn protocol_to_kind(protocol: &Protocol) -> Option<ProtocolKind> {
    match protocol {
        Protocol::OpenaiChat | Protocol::Gemini => Some(ProtocolKind::OpenaiChat),
        Protocol::Anthropic => Some(ProtocolKind::Anthropic),
        Protocol::GoogleTranslate
        | Protocol::CustomHttp
        | Protocol::Deepl
        | Protocol::Microsoft
        | Protocol::Baidu
        | Protocol::Youdao
        | Protocol::Tencent => None,
    }
}

/// 把 DB-backed profile 转成 wire-layer preset。
///
/// 失败条件：`protocol_to_kind` 返回 `None`（google_translate/custom_http/传统 MT）。
/// 此时该 profile 不是可调用的 AI 引擎，调用方应跳过它或把它标为失败结果。
///
/// 字段映射（load-bearing）：
/// - `preset.id = profile.secret_ref` — keystore key 名；`service::translate` 用
///   `keystore.get_key(&preset.id)`，所以 id 必须是 secret_ref 才能命中 DB-backed key。
/// - `preset.default_model = profile.model.unwrap_or_default()` — 空字符串由 wire 层
///   404 分类（Config::InvalidRequest）兜底，适配器不做 model 必填校验。
/// - `preset.auth` 只读 **行上** `capabilities.auth`，缺省 Bearer。禁止回查 catalog。
pub fn profile_to_preset(profile: &ProviderProfile) -> Result<ProviderPreset, String> {
    let protocol = protocol_to_kind(&profile.protocol).ok_or_else(|| {
        format!(
            "unsupported protocol for provider {}: {:?}",
            profile.uuid, profile.protocol
        )
    })?;
    let auth = profile.capabilities.auth.unwrap_or(AuthKind::Bearer);
    Ok(ProviderPreset {
        id: profile.secret_ref.clone(),
        label: profile.name.clone(),
        endpoint: profile.endpoint.clone(),
        protocol,
        default_model: profile.model.clone().unwrap_or_default(),
        needs_key: profile.needs_key,
        auth,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smoke_adapter_compiles_and_maps() {
        assert_eq!(
            protocol_to_kind(&Protocol::Anthropic),
            Some(ProtocolKind::Anthropic)
        );
    }
}
