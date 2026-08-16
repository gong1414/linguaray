//! Provider preset catalog IPC.

use serde::Serialize;

#[derive(Serialize, specta::Type)]
pub struct CatalogPresetDto {
    id: String,
    label: String,
    endpoint: String,
    default_model: String,
    needs_key: bool,
    auth: linguaray_contracts::AuthKind,
    requires_user_endpoint: bool,
    notes: Option<String>,
    console_url: Option<String>,
    support_tier: linguaray_contracts::SupportTier,
    icon: Option<String>,
}

/// Official catalog rows for the Provider Center preset grid.
/// No DB / keystore — default deny except the main window capability.
#[tauri::command]
#[specta::specta]
pub fn provider_list_presets() -> Result<Vec<CatalogPresetDto>, String> {
    let file = linguaray_catalog::load().map_err(|e| e.to_string())?;
    Ok(file
        .providers
        .into_iter()
        .map(|p| CatalogPresetDto {
            id: p.id,
            label: p.label,
            endpoint: p.endpoint,
            default_model: p.default_model,
            needs_key: p.needs_key,
            auth: p.auth,
            requires_user_endpoint: p.requires_user_endpoint,
            notes: p.notes,
            console_url: p.console_url,
            support_tier: p.support_tier,
            icon: p.icon,
        })
        .collect())
}
