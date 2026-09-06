#[uniffi::export(async_runtime = "tokio")]
impl RuntimeBackup {
    pub async fn export_to(&self, destination_path: String) -> Result<BackupSummary, RuntimeError> {
        let data_dir = runtime_data_dir(&self.runtime)?;
        // Hold read guards so a settings/history/glossary write cannot race
        // the archive snapshot.
        let _state = self.runtime.inner.state.read().await;
        let _glossary = self.runtime.inner.glossary.read().await;
        let _history = self.runtime.inner.history.read().await;
        let _vocabulary = self.runtime.inner.vocabulary.read().await;
        let (created_at, file_count) =
            backup::export_backup(&data_dir, PathBuf::from(destination_path).as_path())?;
        Ok(BackupSummary {
            created_at,
            file_count,
            includes_secrets: false,
        })
    }

    pub async fn restore_from(&self, source_path: String) -> Result<RestoreSummary, RuntimeError> {
        let staged = backup::stage_backup(PathBuf::from(source_path).as_path())?;
        let result = self.restore_staged(&staged).await;
        backup::discard_staging(&staged.directory);
        result
    }
}

impl RuntimeBackup {
    async fn restore_staged(
        &self,
        staged: &backup::StagedBackup,
    ) -> Result<RestoreSummary, RuntimeError> {
        let data_dir = runtime_data_dir(&self.runtime)?;
        let mut next_settings = Settings::load(staged.directory.join("settings.json"))?;
        let seeded = apply_catalog_seed(&mut next_settings);
        if seeded {
            next_settings.save(staged.directory.join("settings.json"))?;
        }

        // Parse all persisted stores before replacing the live directory.
        // Their temporary paths are discarded; the installed copies are
        // reloaded below so future writes target the real v2 directory.
        let _ = GlossaryStore::load(&staged.directory)?;
        let _ = HistoryStore::load(&staged.directory);
        let _ = VocabularyStore::load(&staged.directory);

        let mut state = self.runtime.inner.state.write().await;
        let mut glossary = self.runtime.inner.glossary.write().await;
        let mut history = self.runtime.inner.history.write().await;
        let mut vocabulary = self.runtime.inner.vocabulary.write().await;

        let mut provider_secrets = state.provider_secrets.clone();
        provider_secrets.retain(|provider_id, _| next_settings.providers.contains_key(provider_id));
        let previous_proxy = linguaray_engine::current_network_proxy()?;
        let next_engine =
            engine::build_from_settings_with_secrets(&next_settings, &provider_secrets)?;

        let installed = match backup::install_staged_backup(&data_dir, &staged.directory) {
            Ok(installed) => installed,
            Err(error) => {
                let _ = linguaray_engine::configure_network_proxy(previous_proxy);
                return Err(error.into());
            }
        };
        let next_glossary = match GlossaryStore::load(&data_dir) {
            Ok(glossary) => glossary,
            Err(error) => {
                drop(installed);
                let _ = linguaray_engine::configure_network_proxy(previous_proxy);
                return Err(error.into());
            }
        };
        let next_history = HistoryStore::load(&data_dir);
        let next_vocabulary = VocabularyStore::load(&data_dir);
        installed.commit();
        *state = RuntimeState {
            settings: next_settings,
            engine: next_engine,
            provider_secrets,
        };
        *glossary = next_glossary;
        *history = next_history;
        *vocabulary = next_vocabulary;
        drop(vocabulary);
        drop(history);
        drop(glossary);
        drop(state);

        for change in [
            SettingsChange::General,
            SettingsChange::Appearance,
            SettingsChange::Shortcuts,
            SettingsChange::Advanced,
            SettingsChange::Providers,
            SettingsChange::Glossary,
            SettingsChange::History,
            SettingsChange::Vocabulary,
        ] {
            let _ = self.runtime.inner.events.send(change);
        }
        Ok(RestoreSummary {
            created_at: staged.manifest.created_at,
            includes_secrets: staged.manifest.includes_secrets,
        })
    }
}

fn runtime_data_dir(runtime: &Runtime) -> Result<PathBuf, RuntimeError> {
    PathBuf::from(runtime.inner.settings_file_path.as_ref())
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| RuntimeError::from("runtime settings path has no parent".to_owned()))
}
