use std::collections::{HashMap, HashSet};

use super::Settings;

pub fn append_translation_service_order(settings: &mut Settings, service_id: &str) {
    if !settings
        .general
        .translation_service_order
        .iter()
        .any(|id| id == service_id)
    {
        settings
            .general
            .translation_service_order
            .push(service_id.to_owned());
    }
}

pub fn remove_translation_service_order(settings: &mut Settings, service_id: &str) {
    settings
        .general
        .translation_service_order
        .retain(|id| id != service_id);
}

pub fn remove_provider_from_translation_order(settings: &mut Settings, provider_id: &str) {
    let prefix = format!("{provider_id}+");
    settings
        .general
        .translation_service_order
        .retain(|id| id != provider_id && !id.starts_with(&prefix));
}

pub fn effective_translation_service_order(
    stored: &[String],
    translation_ids: &[String],
    created_at: &HashMap<String, Option<u64>>,
) -> Vec<String> {
    let known: HashSet<&String> = translation_ids.iter().collect();
    let mut order: Vec<String> = stored
        .iter()
        .filter(|id| known.contains(id))
        .cloned()
        .collect();
    let mut missing: Vec<String> = translation_ids
        .iter()
        .filter(|id| !order.iter().any(|existing| existing == *id))
        .cloned()
        .collect();
    missing.sort_by(|a, b| {
        created_at
            .get(a)
            .copied()
            .flatten()
            .cmp(&created_at.get(b).copied().flatten())
            .then(a.cmp(b))
    });
    order.extend(missing);
    order
}
