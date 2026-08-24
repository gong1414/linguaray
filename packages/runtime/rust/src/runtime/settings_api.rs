impl RuntimeSettings {
    async fn get_json_impl(&self) -> Result<String, String> {
        let state = self.runtime.inner.state.read().await;
        state.settings.to_pretty_json()
    }

    async fn get_section<T: Clone>(&self, select: impl FnOnce(&Settings) -> &T) -> T {
        select(&self.runtime.inner.state.read().await.settings).clone()
    }

    async fn update_section<T, P>(
        &self,
        change: SettingsChange,
        patch: P,
        select: impl FnOnce(&mut Settings) -> &mut T + Send + 'static,
    ) -> Result<T, String>
    where
        T: Clone + ApplyPatch<P>,
        P: Send + 'static,
    {
        self.commit_settings(change, move |settings| {
            let section = select(settings);
            section.apply(patch);
            Ok(section.clone())
        })
        .await
    }

    async fn commit_settings<F, T>(&self, change: SettingsChange, update: F) -> Result<T, String>
    where
        F: FnOnce(&mut Settings) -> Result<T, String>,
    {
        let mut state = self.runtime.inner.state.write().await;
        let mut next_settings = state.settings.clone();
        let result = update(&mut next_settings)?;
        next_settings.touch_last_updated()?;

        let engine_changed = next_settings.providers != state.settings.providers
            || next_settings.advanced.proxy_mode != state.settings.advanced.proxy_mode
            || next_settings.advanced.proxy_url != state.settings.advanced.proxy_url
            || next_settings.advanced.proxy_bypass != state.settings.advanced.proxy_bypass;

        let settings_file_path = self.runtime.inner.settings_file_path.as_ref();
        if engine_changed {
            let mut next_provider_secrets = state.provider_secrets.clone();
            next_provider_secrets
                .retain(|provider_id, _| next_settings.providers.contains_key(provider_id));
            let previous_proxy = linguaray_engine::current_network_proxy()?;
            let next_engine =
                engine::build_from_settings_with_secrets(&next_settings, &next_provider_secrets)?;
            if let Err(error) = next_settings.save(settings_file_path) {
                let _ = linguaray_engine::configure_network_proxy(previous_proxy);
                return Err(error);
            }
            *state = RuntimeState {
                settings: next_settings,
                engine: next_engine,
                provider_secrets: next_provider_secrets,
            };
        } else {
            next_settings.save(settings_file_path)?;
            state.settings = next_settings;
        }

        // Release the write lock before broadcasting so a subscriber that
        // immediately re-reads doesn't block on the same lock.
        drop(state);

        // `send` only fails when there are zero active receivers, which is
        // a benign condition (no one is listening yet); ignore it.
        let _ = self.runtime.inner.events.send(change);

        Ok(result)
    }
}

include!("translation_support.rs");

#[uniffi::export(async_runtime = "tokio")]
impl RuntimeSettings {
    /// Returns the active subset of translation targets based on the
    /// detected source language.
    ///
    /// * `Always` targets are always included.
    /// * `AutoDetect` targets are included only when their source matches
    ///   the detected language (or when no detected language is available).
    pub async fn get_active_translation_targets(
        &self,
        targets: Vec<TranslationTarget>,
        detected_language: Option<String>,
    ) -> Vec<TranslationTarget> {
        TranslationTarget::filter_active(&targets, detected_language.as_deref())
    }

    pub async fn get_json(&self) -> Result<String, RuntimeError> {
        self.get_json_impl().await.map_err(Into::into)
    }

    pub async fn get_general(&self) -> Result<GeneralSettings, RuntimeError> {
        Ok(self.get_section(|s| &s.general).await)
    }

    pub async fn update_general(
        &self,
        patch: GeneralSettingsPatch,
    ) -> Result<GeneralSettings, RuntimeError> {
        self.update_section(SettingsChange::General, patch, |s| &mut s.general)
            .await
            .map_err(Into::into)
    }

    pub async fn get_appearance(&self) -> Result<AppearanceSettings, RuntimeError> {
        Ok(self.get_section(|s| &s.appearance).await)
    }

    pub async fn update_appearance(
        &self,
        patch: AppearanceSettingsPatch,
    ) -> Result<AppearanceSettings, RuntimeError> {
        self.update_section(SettingsChange::Appearance, patch, |s| &mut s.appearance)
            .await
            .map_err(Into::into)
    }

    pub async fn get_shortcuts(&self) -> Result<ShortcutSettings, RuntimeError> {
        Ok(self.get_section(|s| &s.shortcuts).await)
    }

    pub async fn update_shortcuts(
        &self,
        patch: ShortcutSettingsPatch,
    ) -> Result<ShortcutSettings, RuntimeError> {
        self.update_section(SettingsChange::Shortcuts, patch, |s| &mut s.shortcuts)
            .await
            .map_err(Into::into)
    }

    pub async fn reset_shortcuts(&self) -> Result<ShortcutSettings, RuntimeError> {
        self.commit_settings(SettingsChange::Shortcuts, |settings| {
            settings.shortcuts = ShortcutSettings::default();
            Ok(settings.shortcuts.clone())
        })
        .await
        .map_err(Into::into)
    }

    pub async fn get_advanced(&self) -> Result<AdvancedSettings, RuntimeError> {
        Ok(self.get_section(|s| &s.advanced).await)
    }

    pub async fn update_advanced(
        &self,
        patch: AdvancedSettingsPatch,
    ) -> Result<AdvancedSettings, RuntimeError> {
        self.update_section(SettingsChange::Advanced, patch, |s| &mut s.advanced)
            .await
            .map_err(Into::into)
    }

    pub async fn generate_provider_id(
        &self,
        provider_type: String,
    ) -> Result<String, RuntimeError> {
        let base_id = provider_type.trim().to_lowercase();
        if base_id.is_empty() {
            return Err(RuntimeError::from("provider_type is required".to_owned()));
        }

        let state = self.runtime.inner.state.read().await;
        let existing_ids: Vec<&String> = state.settings.providers.keys().collect();

        // If the base ID is free, use it as-is
        if !existing_ids.contains(&&base_id) {
            return Ok(base_id);
        }

        // Find the first available numeric suffix starting from 1
        for suffix in 1.. {
            let candidate = format!("{base_id}{suffix}");
            if !existing_ids.contains(&&candidate) {
                return Ok(candidate);
            }
        }

        unreachable!()
    }

    pub async fn list_providers(&self) -> Result<Vec<ProviderConfigEntry>, RuntimeError> {
        let state = self.runtime.inner.state.read().await;
        Ok(state
            .settings
            .providers
            .iter()
            .map(|(provider_id, provider)| normalized_provider_entry(provider_id, provider))
            .collect())
    }

    /// Hydrates one provider with credentials read by Flutter from the OS
    /// secure store. Values live only in process memory and are never written
    /// to settings or returned by `list_providers` / `get_json`.
    pub async fn set_provider_secrets(
        &self,
        provider_id: String,
        secrets: HashMap<String, String>,
    ) -> Result<(), RuntimeError> {
        let provider_id = validate_provider_id(provider_id).map_err(RuntimeError::from)?;
        let mut state = self.runtime.inner.state.write().await;
        if !state.settings.providers.contains_key(&provider_id) {
            return Err(RuntimeError::from(format!(
                "provider `{provider_id}` does not exist"
            )));
        }

        let secrets = secrets
            .into_iter()
            .filter(|(key, value)| !key.trim().is_empty() && !value.is_empty())
            .collect::<HashMap<_, _>>();
        if secrets.is_empty() {
            state.provider_secrets.remove(&provider_id);
        } else {
            state.provider_secrets.insert(provider_id, secrets);
        }
        state.engine =
            engine::build_from_settings_with_secrets(&state.settings, &state.provider_secrets)?;
        Ok(())
    }

    pub async fn list_services(&self) -> Result<Vec<ServiceConfigEntry>, RuntimeError> {
        let state = self.runtime.inner.state.read().await;
        let mut by_id: HashMap<String, ServiceConfigEntry> = HashMap::new();

        for (provider_id, provider) in &state.settings.providers {
            let entry = normalized_provider_entry(provider_id, provider);
            if let Ok(engine_provider) = state.engine.require(provider_id) {
                if provider_supports_service(engine_provider.as_ref(), ServiceType::Dictionary)
                    && advertises_system_capability(provider_id, ServiceType::Dictionary)
                {
                    let service = service_entry_for_provider_type(
                        &format!("{provider_id}+dictionary"),
                        &entry,
                        ServiceType::Dictionary,
                    );
                    by_id.insert(service.id.clone(), service);
                }
                if provider_supports_service(engine_provider.as_ref(), ServiceType::Translation)
                    && advertises_system_capability(provider_id, ServiceType::Translation)
                {
                    let service = service_entry_for_provider_type(
                        &format!("{provider_id}+translation"),
                        &entry,
                        ServiceType::Translation,
                    );
                    by_id.insert(service.id.clone(), service);
                }
                if provider_supports_service(engine_provider.as_ref(), ServiceType::Ocr)
                    && advertises_system_capability(provider_id, ServiceType::Ocr)
                {
                    let service = service_entry_for_provider_type(
                        &format!("{provider_id}+ocr"),
                        &entry,
                        ServiceType::Ocr,
                    );
                    by_id.insert(service.id.clone(), service);
                }
            }
        }

        for (service_id, service) in &state.settings.services {
            if !state.settings.providers.contains_key(&service.provider_id) {
                continue;
            }
            let Ok(engine_provider) = state.engine.require(&service.provider_id) else {
                continue;
            };
            if !provider_supports_service(engine_provider.as_ref(), service.r#type) {
                continue;
            }
            if !advertises_system_capability(&service.provider_id, service.r#type) {
                continue;
            }
            let mut entry = service.clone();
            entry.id = service_id.clone();
            by_id.insert(service_id.clone(), entry);
        }

        let mut services: Vec<ServiceConfigEntry> = by_id.into_values().collect();
        let translation_ids: Vec<String> = services
            .iter()
            .filter(|service| service.r#type == ServiceType::Translation)
            .map(|service| service.id.clone())
            .collect();
        let created_at = services
            .iter()
            .map(|service| (service.id.clone(), service.created_at))
            .collect::<HashMap<_, _>>();
        let order = effective_translation_service_order(
            &state.settings.general.translation_service_order,
            &translation_ids,
            &created_at,
        );
        services.sort_by(|a, b| {
            let translation_rank = |service: &ServiceConfigEntry| {
                if service.r#type == ServiceType::Translation {
                    order
                        .iter()
                        .position(|id| id == &service.id)
                        .unwrap_or(usize::MAX)
                } else {
                    usize::MAX
                }
            };
            translation_rank(a)
                .cmp(&translation_rank(b))
                .then_with(|| service_type_rank(a.r#type).cmp(&service_type_rank(b.r#type)))
                .then_with(|| a.created_at.cmp(&b.created_at))
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(services)
    }

    pub async fn list_models(&self, provider_id: String) -> Result<Vec<String>, RuntimeError> {
        let provider_id = validate_provider_id(provider_id).map_err(RuntimeError::from)?;
        let runtime = self.runtime.clone();
        run_on_worker_thread(move || async move {
            let provider = {
                let state = runtime.inner.state.read().await;
                state
                    .engine
                    .require(&provider_id)
                    .map_err(|e| e.to_string())?
                    .clone()
            };
            provider.list_models().await.map_err(|e| e.to_string())
        })
        .await
        .map_err(|e: String| RuntimeError::from(e))
    }

    /// Validates and probes a provider without changing persisted settings or
    /// the process-wide runtime engine. The caller supplies credentials read
    /// from secure storage; they live only in this temporary provider object.
    ///
    /// LLM providers are probed by listing models. Translation, dictionary,
    /// and OCR-only providers execute a small capability-specific request.
    /// The returned number is the model count for LLM providers and zero for
    /// traditional providers.
    pub async fn test_provider(
        &self,
        provider_id: String,
        provider_type: String,
        fields: HashMap<String, String>,
    ) -> Result<u32, RuntimeError> {
        let provider_id = validate_provider_id(provider_id).map_err(RuntimeError::from)?;
        let provider_type =
            validate_required("provider_type", provider_type).map_err(RuntimeError::from)?;
        let provider_type = crate::domain::settings::parse_provider_type(&provider_type)
            .map_err(RuntimeError::from)?;
        let entry = ProviderConfigEntry {
            id: provider_id.clone(),
            r#type: provider_type,
            fields,
            created_at: None,
            preset_id: None,
        };
        let config = crate::domain::settings::provider_config_from_settings(&entry)
            .map_err(RuntimeError::from)?;
        let mut providers = std::collections::BTreeMap::new();
        providers.insert(provider_id.clone(), config);
        let engine = engine::build_from_engine_config(linguaray_engine::EngineConfig { providers })
            .map_err(RuntimeError::from)?;
        let provider = engine
            .require(&provider_id)
            .map_err(|error| RuntimeError::from(error.to_string()))?
            .clone();

        run_on_worker_thread(move || async move {
            if provider.llm().is_some() {
                let models = provider
                    .list_models()
                    .await
                    .map_err(|error| error.to_string())?;
                return u32::try_from(models.len())
                    .map_err(|_| "provider returned too many models".to_owned());
            }

            if matches!(provider.name(), "libretranslate" | "mtranserver") {
                provider
                    .list_models()
                    .await
                    .map_err(|error| error.to_string())?;
                return Ok(0);
            }

            if let Some(translation) = provider.translation() {
                translation
                    .translate(TranslateRequest {
                        source_language: Some("en".to_owned()),
                        target_language: Some("zh-Hans".to_owned()),
                        text: "Hello".to_owned(),
                    })
                    .await
                    .map_err(|error| error.to_string())?;
                return Ok(0);
            }
            if let Some(dictionary) = provider.dictionary() {
                dictionary
                    .look_up(LookUpRequest {
                        source_language: "en".to_owned(),
                        target_language: "zh-Hans".to_owned(),
                        word: "hello".to_owned(),
                    })
                    .await
                    .map_err(|error| error.to_string())?;
                return Ok(0);
            }
            if let Some(ocr) = provider.ocr() {
                let fixture = base64::engine::general_purpose::STANDARD
                    .encode(include_bytes!("../../test/fixtures/system_ocr_stable.png"));
                ocr.recognize_text(RecognizeTextRequest {
                    image_path: None,
                    base64_image: Some(fixture),
                })
                .await
                .map_err(|error| error.to_string())?;
                return Ok(0);
            }
            Err("provider does not expose a testable capability".to_owned())
        })
        .await
        .map_err(RuntimeError::from)
    }

    /// Discovers models using an in-memory provider draft. Unlike
    /// `list_models`, this does not require the provider to be saved first and
    /// never writes credentials or provisional settings to disk.
    pub async fn discover_provider_models(
        &self,
        provider_id: String,
        provider_type: String,
        fields: HashMap<String, String>,
    ) -> Result<Vec<String>, RuntimeError> {
        let provider_id = validate_provider_id(provider_id).map_err(RuntimeError::from)?;
        let provider_type =
            validate_required("provider_type", provider_type).map_err(RuntimeError::from)?;
        let provider_type = crate::domain::settings::parse_provider_type(&provider_type)
            .map_err(RuntimeError::from)?;
        let entry = ProviderConfigEntry {
            id: provider_id.clone(),
            r#type: provider_type,
            fields,
            created_at: None,
            preset_id: None,
        };
        let config = crate::domain::settings::provider_config_from_settings(&entry)
            .map_err(RuntimeError::from)?;
        let mut providers = std::collections::BTreeMap::new();
        providers.insert(provider_id.clone(), config);
        let engine = engine::build_from_engine_config(linguaray_engine::EngineConfig { providers })
            .map_err(RuntimeError::from)?;
        let provider = engine
            .require(&provider_id)
            .map_err(|error| RuntimeError::from(error.to_string()))?
            .clone();

        run_on_worker_thread(move || async move {
            provider
                .list_models()
                .await
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(RuntimeError::from)
    }

    pub async fn get_service(
        &self,
        service_id: String,
    ) -> Result<Option<ServiceConfigEntry>, RuntimeError> {
        let service_id = validate_provider_id(service_id).map_err(RuntimeError::from)?;
        let state = self.runtime.inner.state.read().await;
        if let Some(service) = state.settings.services.get(&service_id) {
            let mut entry = service.clone();
            entry.id = service_id;
            return Ok(Some(entry));
        }

        let Some(provider) = state.settings.providers.get(&service_id) else {
            return Ok(None);
        };
        let entry = normalized_provider_entry(&service_id, provider);
        let service_type = state.engine.require(&service_id).ok().and_then(|provider| {
            if provider.translation().is_some() || provider.llm().is_some() {
                Some(ServiceType::Translation)
            } else if provider.dictionary().is_some() {
                Some(ServiceType::Dictionary)
            } else if provider.ocr().is_some() {
                Some(ServiceType::Ocr)
            } else {
                None
            }
        });

        Ok(service_type
            .map(|service_type| service_entry_for_provider_type(&service_id, &entry, service_type)))
    }

    pub async fn get_provider(
        &self,
        provider_id: String,
    ) -> Result<Option<ProviderConfigEntry>, RuntimeError> {
        let provider_id = validate_provider_id(provider_id).map_err(RuntimeError::from)?;
        let state = self.runtime.inner.state.read().await;
        Ok(state
            .settings
            .providers
            .get(&provider_id)
            .map(|provider| normalized_provider_entry(&provider_id, provider)))
    }

    pub async fn update_provider(
        &self,
        provider_id: String,
        provider_type: String,
        fields: HashMap<String, String>,
        preset_id: Option<String>,
    ) -> Result<ProviderConfigEntry, RuntimeError> {
        let provider_id = validate_provider_id(provider_id).map_err(RuntimeError::from)?;
        let provider_type =
            validate_required("provider_type", provider_type).map_err(RuntimeError::from)?;
        let provider_type = crate::domain::settings::parse_provider_type(&provider_type)
            .map_err(RuntimeError::from)?;
        let entry = ProviderConfigEntry {
            id: provider_id.clone(),
            r#type: provider_type,
            fields,
            created_at: None,
            preset_id: preset_id.clone(),
        };
        let config = crate::domain::settings::provider_config_from_settings(&entry)
            .map_err(RuntimeError::from)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .ok();

        self.commit_settings(SettingsChange::Providers, move |settings| {
            let mut entry = provider_entry_from_config(&provider_id, &config)?;
            entry.preset_id = preset_id.clone().or_else(|| {
                settings
                    .providers
                    .get(&provider_id)
                    .and_then(|existing| existing.preset_id.clone())
            });
            let implicit_translation = format!("{provider_id}+translation");
            if let Some(existing) = settings.providers.get_mut(&provider_id) {
                existing.id = provider_id.clone();
                existing.r#type = entry.r#type;
                existing.fields = entry.fields.clone();
                existing.preset_id = entry.preset_id.clone();
            } else {
                let mut new_entry = entry.clone();
                new_entry.created_at = now;
                settings.providers.insert(provider_id.clone(), new_entry);
                append_translation_service_order(settings, &implicit_translation);
            }
            Ok(entry)
        })
        .await
        .map_err(Into::into)
    }

    pub async fn update_service(
        &self,
        service_id: String,
        provider_id: String,
        service_type: ServiceType,
        name: String,
        fields: HashMap<String, String>,
    ) -> Result<ServiceConfigEntry, RuntimeError> {
        let service_id = validate_provider_id(service_id).map_err(RuntimeError::from)?;
        let provider_id = validate_provider_id(provider_id).map_err(RuntimeError::from)?;
        let name = name.trim().to_owned();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .ok();

        {
            let state = self.runtime.inner.state.read().await;
            if !state.settings.providers.contains_key(&provider_id) {
                return Err(RuntimeError::from(format!(
                    "provider `{provider_id}` does not exist"
                )));
            }
            let engine_provider = state
                .engine
                .require(&provider_id)
                .map_err(|error| RuntimeError::from(error.to_string()))?;
            if !provider_supports_service(engine_provider.as_ref(), service_type)
                || !advertises_system_capability(&provider_id, service_type)
            {
                return Err(RuntimeError::from(format!(
                    "provider `{provider_id}` does not support {service_type:?}"
                )));
            }
        }

        self.commit_settings(SettingsChange::Providers, move |settings| {
            if !settings.providers.contains_key(&provider_id) {
                return Err(format!("provider `{provider_id}` does not exist"));
            }
            if service_id == provider_id {
                return Err("custom service id must be different from provider id".to_owned());
            }

            let mut entry = ServiceConfigEntry {
                id: service_id.clone(),
                provider_id,
                r#type: service_type,
                name,
                fields,
                created_at: None,
            };
            if let Some(existing) = settings.services.get(&service_id) {
                entry.created_at = existing.created_at.or(now);
            } else {
                entry.created_at = now;
            }
            let is_new = !settings.services.contains_key(&service_id);
            settings.services.insert(service_id.clone(), entry.clone());
            if is_new && service_type == ServiceType::Translation {
                append_translation_service_order(settings, &service_id);
            }
            Ok(entry)
        })
        .await
        .map_err(Into::into)
    }

    pub async fn delete_provider(
        &self,
        provider_id: String,
    ) -> Result<Option<ProviderConfigEntry>, RuntimeError> {
        let provider_id = validate_provider_id(provider_id).map_err(RuntimeError::from)?;
        self.commit_settings(SettingsChange::Providers, move |settings| {
            let removed = settings
                .providers
                .remove(&provider_id)
                .map(|provider| normalized_provider_entry(&provider_id, &provider));
            let removed_service_ids = settings
                .services
                .values()
                .filter(|service| service.provider_id == provider_id)
                .map(|service| service.id.clone())
                .collect::<std::collections::HashSet<_>>();
            settings
                .services
                .retain(|_, service| service.provider_id != provider_id);
            remove_provider_from_translation_order(settings, &provider_id);
            if removed_service_ids.contains(&settings.general.default_translation_service) {
                settings.general.default_translation_service.clear();
            }
            if removed_service_ids.contains(&settings.general.default_ocr_service) {
                settings.general.default_ocr_service.clear();
            }
            if removed_service_ids.contains(&settings.general.default_directory_service) {
                settings.general.default_directory_service.clear();
            }
            Ok(removed)
        })
        .await
        .map_err(Into::into)
    }

    pub async fn delete_service(
        &self,
        service_id: String,
    ) -> Result<Option<ServiceConfigEntry>, RuntimeError> {
        let service_id = validate_provider_id(service_id).map_err(RuntimeError::from)?;
        self.commit_settings(SettingsChange::Providers, move |settings| {
            let removed = settings.services.remove(&service_id).map(|mut service| {
                service.id = service_id.clone();
                service
            });
            remove_translation_service_order(settings, &service_id);
            if settings.general.default_translation_service == service_id {
                settings.general.default_translation_service.clear();
            }
            if settings.general.default_ocr_service == service_id {
                settings.general.default_ocr_service.clear();
            }
            if settings.general.default_directory_service == service_id {
                settings.general.default_directory_service.clear();
            }
            Ok(removed)
        })
        .await
        .map_err(Into::into)
    }

    pub async fn set_translation_service_order(
        &self,
        order: Vec<String>,
    ) -> Result<Vec<String>, RuntimeError> {
        self.commit_settings(SettingsChange::General, move |settings| {
            settings.general.translation_service_order = order.clone();
            Ok(settings.general.translation_service_order.clone())
        })
        .await
        .map_err(Into::into)
    }

    /// Returns a fresh subscription that starts receiving
    /// [`SettingsChange`] events emitted **after** this call. Existing
    /// state should be loaded eagerly via the corresponding `get_*`
    /// methods; subscriptions are intentionally not replayed.
    pub fn subscribe(&self) -> Arc<SettingsSubscription> {
        Arc::new(SettingsSubscription {
            receiver: AsyncMutex::new(self.runtime.inner.events.subscribe()),
        })
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl SettingsSubscription {
    /// Awaits the next [`SettingsChange`] event. Returns `None` when the
    /// owning [`Runtime`] has been dropped (no further events will arrive).
    /// If this subscription falls behind, missed events are silently
    /// skipped and the next available event is returned.
    pub async fn next(&self) -> Result<Option<SettingsChange>, RuntimeError> {
        let mut rx = self.receiver.lock().await;
        loop {
            match rx.recv().await {
                Ok(change) => return Ok(Some(change)),
                Err(broadcast::error::RecvError::Closed) => return Ok(None),
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl ExternalActionSubscription {
    pub async fn next(&self) -> Result<Option<ExternalActionRequest>, RuntimeError> {
        let mut receiver = self.receiver.lock().await;
        loop {
            match receiver.recv().await {
                Ok(request) => return Ok(Some(request)),
                Err(broadcast::error::RecvError::Closed) => return Ok(None),
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    }
}
