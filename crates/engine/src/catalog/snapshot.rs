use serde::Deserialize;
use std::sync::OnceLock;

const SNAPSHOT_JSON: &str = include_str!("models_dev_snapshot.json");

#[derive(Clone, Debug, Deserialize)]
struct SnapshotFile {
    #[allow(dead_code)]
    commit: String,
    providers: Vec<SnapshotProvider>,
}

#[derive(Clone, Debug, Deserialize)]
struct SnapshotProvider {
    id: String,
    models: Vec<CatalogSnapshotModel>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CatalogSnapshotModel {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub modalities: SnapshotModalities,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct SnapshotModalities {
    #[serde(default)]
    pub input: Vec<String>,
    #[serde(default)]
    pub output: Vec<String>,
}

fn snapshot() -> &'static SnapshotFile {
    static SNAPSHOT: OnceLock<SnapshotFile> = OnceLock::new();
    SNAPSHOT.get_or_init(|| {
        serde_json::from_str(SNAPSHOT_JSON).expect("models.dev snapshot must parse")
    })
}

/// models.dev provider id used for a LinguaRay preset.
pub fn models_dev_id_for_preset(preset_id: &str) -> Option<&'static str> {
    match preset_id {
        "openai" => Some("openai"),
        "anthropic" => Some("anthropic"),
        "gemini" => Some("google"),
        "deepseek" => Some("deepseek"),
        "xai" => Some("xai"),
        "groq" => Some("groq"),
        "openrouter" => Some("openrouter"),
        "bailian-qwen" => Some("alibaba"),
        "zhipu-bigmodel" => Some("zhipuai"),
        "moonshot-kimi" => Some("moonshotai"),
        "siliconflow-cn" | "siliconflow-global" => Some("siliconflow"),
        "modelscope" => Some("modelscope"),
        "lm-studio" => Some("lmstudio"),
        "minimax" => Some("minimax"),
        "stepfun" => Some("stepfun-ai"),
        "mistral" => Some("mistral"),
        "together" => Some("togetherai"),
        "fireworks" => Some("fireworks-ai"),
        _ => None,
    }
}

pub fn models_for_preset(preset_id: &str) -> Vec<CatalogSnapshotModel> {
    let Some(dev_id) = models_dev_id_for_preset(preset_id) else {
        return Vec::new();
    };
    snapshot()
        .providers
        .iter()
        .find(|provider| provider.id == dev_id)
        .map(|provider| provider.models.clone())
        .unwrap_or_default()
}

pub fn snapshot_models() -> Vec<(String, Vec<CatalogSnapshotModel>)> {
    snapshot()
        .providers
        .iter()
        .map(|provider| (provider.id.clone(), provider.models.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_json_parses() {
        let file = snapshot();
        assert_eq!(file.commit, "08324a024a9de60e507e08779f6667fbf8a25001");
        let ids: Vec<_> = file
            .providers
            .iter()
            .map(|provider| provider.id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec![
                "openai",
                "anthropic",
                "google",
                "deepseek",
                "xai",
                "groq",
                "openrouter",
                "alibaba",
                "zhipuai",
                "moonshotai",
                "siliconflow",
                "modelscope",
                "lmstudio",
                "minimax",
                "stepfun-ai",
                "mistral",
                "togetherai",
                "fireworks-ai",
            ]
        );
        assert!(file
            .providers
            .iter()
            .any(|provider| !provider.models.is_empty()));
    }
}
