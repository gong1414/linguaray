use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectLanguageRequest {
    pub texts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextDetection {
    pub detected_language: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectLanguageResponse {
    pub detections: Option<Vec<TextDetection>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub code: String,
    pub local_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguagePair {
    pub source_language: Option<String>,
    pub source_language_id: Option<String>,
    pub target_language: Option<String>,
    pub target_language_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TranslationTarget {
    pub source: String,
    pub target: String,
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
}

const fn enabled_by_default() -> bool {
    true
}

impl TranslationTarget {
    pub const AUTO_SOURCE: &'static str = "auto";

    /// Chooses enabled routes for a detected source. Detection is advisory: if
    /// no route matches, enabled user routes remain usable.
    pub fn filter_active(targets: &[Self], detected: Option<&str>) -> Vec<Self> {
        let enabled = targets
            .iter()
            .filter(|target| target.enabled)
            .cloned()
            .collect::<Vec<_>>();
        let selected = enabled
            .iter()
            .filter(|target| {
                target.source == Self::AUTO_SOURCE
                    || detected.is_none_or(|language| target.source == language)
            })
            .cloned()
            .collect::<Vec<_>>();

        if selected.is_empty() { enabled } else { selected }
    }
}

#[cfg(test)]
mod tests {
    use super::TranslationTarget;

    fn target(source: &str, enabled: bool) -> TranslationTarget {
        TranslationTarget {
            source: source.to_owned(),
            target: "zh-Hans".to_owned(),
            enabled,
        }
    }

    #[test]
    fn detection_never_disables_every_user_route() {
        let routes = [target("en", true), target("ja", false)];
        assert_eq!(TranslationTarget::filter_active(&routes, Some("ca")).len(), 1);
    }

    #[test]
    fn auto_and_matching_routes_are_selected() {
        let routes = [target("auto", true), target("en", true), target("ja", true)];
        assert_eq!(TranslationTarget::filter_active(&routes, Some("en")).len(), 2);
    }
}
