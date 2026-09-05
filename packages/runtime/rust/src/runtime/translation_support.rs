#[derive(Clone)]
struct ResolvedService {
    provider_id: String,
    entry: ServiceConfigEntry,
}

impl ResolvedService {
    fn field(&self, key: &str) -> Option<&str> {
        self.entry
            .fields
            .get(key)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
    }
}

fn service_entry_for_provider_type(
    service_id: &str,
    provider: &ProviderConfigEntry,
    service_type: ServiceType,
) -> ServiceConfigEntry {
    ServiceConfigEntry {
        id: service_id.to_owned(),
        provider_id: provider.id.clone(),
        r#type: service_type,
        name: provider.id.clone(),
        fields: HashMap::new(),
        created_at: provider.created_at,
    }
}

fn render_prompt_template(
    template: &str,
    source_language: &str,
    target_language: &str,
    text: &str,
    glossary: &[GlossaryTerm],
) -> String {
    const GLOSSARY_PLACEHOLDER: &str = "{{glossary}}";
    let rendered = template
        .replace("{{sourceLanguage}}", source_language)
        .replace("{{targetLanguage}}", target_language)
        .replace("{{text}}", text);
    let block = linguaray_engine::prompt::glossary_constraints(glossary);
    if rendered.contains(GLOSSARY_PLACEHOLDER) {
        return rendered.replace(GLOSSARY_PLACEHOLDER, block.as_deref().unwrap_or_default());
    }
    match block {
        Some(block) => format!("{rendered}\n\n{block}"),
        None => rendered,
    }
}

fn glossary_terms(matches: &[GlossaryMatch]) -> Vec<GlossaryTerm> {
    matches
        .iter()
        .map(|hit| GlossaryTerm {
            term: hit.term.clone(),
            translation: hit.translation.clone(),
            forbidden: hit.forbidden.clone(),
        })
        .collect()
}
