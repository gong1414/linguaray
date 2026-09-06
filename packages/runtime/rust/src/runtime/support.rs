fn normalized_provider_entry(
    provider_id: &str,
    provider: &ProviderConfigEntry,
) -> ProviderConfigEntry {
    let mut provider = provider.clone();
    if provider.id.trim().is_empty() {
        provider.id = provider_id.to_owned();
    }
    provider
}

fn service_type_rank(service_type: ServiceType) -> u8 {
    match service_type {
        ServiceType::Translation => 0,
        ServiceType::Dictionary => 1,
        ServiceType::Ocr => 2,
        ServiceType::Llm => 3,
    }
}

fn provider_supports_service(
    provider: &dyn linguaray_core::Provider,
    service_type: ServiceType,
) -> bool {
    match service_type {
        ServiceType::Dictionary => provider.dictionary().is_some(),
        ServiceType::Ocr => provider.ocr().is_some(),
        ServiceType::Translation => provider.translation().is_some() || provider.llm().is_some(),
        ServiceType::Llm => provider.llm().is_some(),
    }
}

fn advertises_system_capability(provider_id: &str, service_type: ServiceType) -> bool {
    if provider_id != "system" {
        return true;
    }
    match service_type {
        ServiceType::Ocr => true,
        ServiceType::Translation | ServiceType::Dictionary | ServiceType::Llm => {
            cfg!(target_os = "macos")
        }
    }
}

fn validate_provider_id(provider_id: String) -> Result<String, String> {
    validate_required("provider_id", provider_id)
}

fn validate_service_provider_id(provider_id: String, suffix: &str) -> Result<String, String> {
    let provider_id = validate_provider_id(provider_id)?;
    Ok(provider_id
        .strip_suffix(suffix)
        .or_else(|| {
            let alternatives: &[&str] = match suffix {
                "+translation" => &["+llm"],
                "+llm" => &["+translation"],
                _ => &[],
            };
            alternatives
                .iter()
                .find_map(|alt| provider_id.strip_suffix(alt))
        })
        .unwrap_or(&provider_id)
        .to_owned())
}

fn validate_optional_required(name: &str, value: Option<String>) -> Result<String, String> {
    validate_required(name, value.unwrap_or_default())
}

fn validate_required(name: &str, value: String) -> Result<String, String> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(format!("{name} is required"));
    }
    Ok(value)
}

fn optional_trimmed(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

async fn run_on_worker_thread<F, Fut, T>(task: F) -> Result<T, String>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T, String>> + 'static,
    T: Send + 'static,
{
    let (sender, receiver) = tokio::sync::oneshot::channel();
    thread::Builder::new()
        .name("linguaray-engine-bridge".to_owned())
        .spawn(move || {
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| format!("failed to build tokio runtime: {error}"))
                .and_then(|runtime| runtime.block_on(task()));
            let _ = sender.send(result);
        })
        .map_err(|error| format!("failed to spawn runtime worker thread: {error}"))?;
    receiver
        .await
        .map_err(|error| format!("runtime worker thread ended unexpectedly: {error}"))?
}
