use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DictLookupResult {
    pub definition: String,
    pub source: String,
}

#[tauri::command]
pub fn dict_lookup(word: String) -> Option<DictLookupResult> {
    crate::dict::lookup(&word).map(|definition| DictLookupResult {
        definition,
        source: "system".into(),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct DictPackage {
    pub package_id: String,
    pub name: String,
    pub version: String,
}

#[tauri::command]
pub async fn dict_list_packages(
    app: tauri::AppHandle,
    state: tauri::State<'_, std::sync::Arc<crate::AppState>>,
) -> Result<Vec<DictPackage>, String> {
    let _ = app;
    let app_state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let gate = app_state.data_gate.read();
        let db = crate::require_database(&app_state, &gate)?;
        db.with_conn(|conn| {
            let mut stmt =
                conn.prepare("SELECT package_id, name, version FROM dict_packages ORDER BY name")?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(DictPackage {
                        package_id: row.get(0)?,
                        name: row.get(1)?,
                        version: row.get(2)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("dict list worker failed: {e}"))?
}
