impl Runtime {
    /// Terms from the enabled books that occur in `text`. Used by the
    /// translation path to constrain the model and by the UI to highlight.
    pub(crate) async fn glossary_matches(
        &self,
        text: &str,
        source_language: Option<&str>,
        target_language: Option<&str>,
    ) -> Vec<GlossaryMatch> {
        self.inner
            .glossary
            .read()
            .await
            .match_text(text, source_language, target_language)
    }

    /// Counts one use of each matched term. Failures here are never worth
    /// failing a translation over, so they stay inside the store.
    pub(crate) async fn record_glossary_hits(&self, matches: &[GlossaryMatch]) {
        if matches.is_empty() {
            return;
        }
        self.inner.glossary.write().await.record_hits(matches);
    }
}

impl RuntimeGlossary {
    /// Applies a change, persists it and tells subscribers the glossary moved.
    async fn commit<T>(
        &self,
        update: impl FnOnce(&mut GlossaryStore) -> Result<T, String>,
    ) -> Result<T, String> {
        let result = {
            let mut store = self.runtime.inner.glossary.write().await;
            update(&mut store)?
        };
        // `send` only fails when nobody is subscribed yet, which is benign.
        let _ = self.runtime.inner.events.send(SettingsChange::Glossary);
        Ok(result)
    }
}

impl RuntimeHistory {
    async fn commit<T>(
        &self,
        update: impl FnOnce(&mut HistoryStore) -> Result<T, String>,
    ) -> Result<T, String> {
        let result = {
            let mut store = self.runtime.inner.history.write().await;
            update(&mut store)?
        };
        let _ = self.runtime.inner.events.send(SettingsChange::History);
        Ok(result)
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RuntimeHistory {
    pub async fn list_entries(
        &self,
        filter: HistoryFilter,
        query: Option<String>,
    ) -> Result<Vec<HistoryEntry>, RuntimeError> {
        self.runtime
            .inner
            .history
            .write()
            .await
            .list_entries(filter, query.as_deref())
            .map_err(Into::into)
    }

    pub async fn counts(&self) -> Result<HistoryCounts, RuntimeError> {
        self.runtime
            .inner
            .history
            .write()
            .await
            .counts()
            .map_err(Into::into)
    }

    pub async fn upsert_entry(
        &self,
        input: HistoryEntryInput,
    ) -> Result<HistoryEntry, RuntimeError> {
        self.commit(|store| store.upsert_entry(input))
            .await
            .map_err(Into::into)
    }

    pub async fn set_favorite(
        &self,
        entry_id: String,
        favorite: bool,
    ) -> Result<Option<HistoryEntry>, RuntimeError> {
        self.commit(|store| store.set_favorite(&entry_id, favorite))
            .await
            .map_err(Into::into)
    }

    pub async fn delete_entries(&self, entry_ids: Vec<String>) -> Result<u32, RuntimeError> {
        self.commit(|store| store.delete_entries(&entry_ids))
            .await
            .map_err(Into::into)
    }

    pub async fn clear(&self) -> Result<u32, RuntimeError> {
        self.commit(|store| store.clear()).await.map_err(Into::into)
    }
}

impl RuntimeVocabulary {
    async fn commit<T>(
        &self,
        update: impl FnOnce(&mut VocabularyStore) -> Result<T, String>,
    ) -> Result<T, String> {
        let result = {
            let mut store = self.runtime.inner.vocabulary.write().await;
            update(&mut store)?
        };
        let _ = self.runtime.inner.events.send(SettingsChange::Vocabulary);
        Ok(result)
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RuntimeVocabulary {
    pub async fn list_entries(
        &self,
        filter: VocabularyFilter,
        query: Option<String>,
    ) -> Result<Vec<VocabularyEntry>, RuntimeError> {
        Ok(self
            .runtime
            .inner
            .vocabulary
            .read()
            .await
            .list_entries(filter, query.as_deref()))
    }

    pub async fn upsert_entry(
        &self,
        input: VocabularyEntryInput,
    ) -> Result<VocabularyEntry, RuntimeError> {
        self.commit(|store| store.upsert_entry(input))
            .await
            .map_err(Into::into)
    }

    pub async fn set_favorite(
        &self,
        entry_id: String,
        favorite: bool,
    ) -> Result<Option<VocabularyEntry>, RuntimeError> {
        self.commit(|store| store.set_favorite(&entry_id, favorite))
            .await
            .map_err(Into::into)
    }

    pub async fn set_note(
        &self,
        entry_id: String,
        note: Option<String>,
    ) -> Result<Option<VocabularyEntry>, RuntimeError> {
        self.commit(|store| store.set_note(&entry_id, note))
            .await
            .map_err(Into::into)
    }

    pub async fn delete_entries(&self, entry_ids: Vec<String>) -> Result<u32, RuntimeError> {
        self.commit(|store| store.delete_entries(&entry_ids))
            .await
            .map_err(Into::into)
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RuntimeGlossary {
    pub async fn list_books(&self) -> Result<Vec<GlossaryBook>, RuntimeError> {
        Ok(self.runtime.inner.glossary.read().await.list_books())
    }

    pub async fn get_book(&self, book_id: String) -> Result<Option<GlossaryBook>, RuntimeError> {
        Ok(self.runtime.inner.glossary.read().await.get_book(&book_id))
    }

    /// Creates a book when `input.id` is empty, otherwise replaces the named
    /// book's metadata.
    pub async fn upsert_book(
        &self,
        input: GlossaryBookInput,
    ) -> Result<GlossaryBook, RuntimeError> {
        self.commit(|store| store.upsert_book(input))
            .await
            .map_err(Into::into)
    }

    /// Returns `false` when the book was already gone.
    pub async fn delete_book(&self, book_id: String) -> Result<bool, RuntimeError> {
        self.commit(|store| store.delete_book(&book_id))
            .await
            .map_err(Into::into)
    }

    /// Terms in a book, newest first. `query` filters on term, translation,
    /// forbidden list and note; `limit` of 0 means no limit.
    pub async fn list_entries(
        &self,
        book_id: String,
        query: Option<String>,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<GlossaryEntry>, RuntimeError> {
        self.runtime
            .inner
            .glossary
            .read()
            .await
            .list_entries(&book_id, query.as_deref(), offset, limit)
            .map_err(Into::into)
    }

    /// How many terms `list_entries` would return with the same `query`.
    pub async fn count_entries(
        &self,
        book_id: String,
        query: Option<String>,
    ) -> Result<u32, RuntimeError> {
        self.runtime
            .inner
            .glossary
            .read()
            .await
            .count_entries(&book_id, query.as_deref())
            .map_err(Into::into)
    }

    /// Creates a term when `input.id` is empty, otherwise replaces it. An
    /// empty id whose term already exists updates that term in place.
    pub async fn upsert_entry(
        &self,
        book_id: String,
        input: GlossaryEntryInput,
    ) -> Result<GlossaryEntry, RuntimeError> {
        self.commit(|store| store.upsert_entry(&book_id, input))
            .await
            .map_err(Into::into)
    }

    /// Returns `false` when the term was already gone.
    pub async fn delete_entry(
        &self,
        book_id: String,
        entry_id: String,
    ) -> Result<bool, RuntimeError> {
        self.commit(|store| store.delete_entry(&book_id, &entry_id))
            .await
            .map_err(Into::into)
    }

    /// Exports a complete book as UTF-8 CSV or TBX.
    pub async fn export_entries(
        &self,
        book_id: String,
        format: GlossaryExchangeFormat,
    ) -> Result<String, RuntimeError> {
        let store = self.runtime.inner.glossary.read().await;
        let book = store.get_book(&book_id).ok_or_else(|| {
            RuntimeError::from(format!("glossary book `{book_id}` does not exist"))
        })?;
        let entries = store
            .list_entries(&book_id, None, 0, 0)
            .map_err(RuntimeError::from)?;
        glossary_exchange::encode(&book, &entries, format).map_err(Into::into)
    }

    /// Merges UTF-8 CSV or TBX into a book. A matching source term updates the
    /// existing row; malformed or empty rows are counted as skipped.
    pub async fn import_entries(
        &self,
        book_id: String,
        content: String,
        format: GlossaryExchangeFormat,
    ) -> Result<GlossaryImportReport, RuntimeError> {
        let book = self
            .runtime
            .inner
            .glossary
            .read()
            .await
            .get_book(&book_id)
            .ok_or_else(|| {
                RuntimeError::from(format!("glossary book `{book_id}` does not exist"))
            })?;
        let entries =
            glossary_exchange::decode(&content, &book, format).map_err(RuntimeError::from)?;
        self.commit(|store| store.import_entries(&book_id, entries))
            .await
            .map_err(Into::into)
    }

    /// Terms present in `text`, for highlighting the source before or during
    /// a translation.
    pub async fn match_text(
        &self,
        text: String,
        source_language: Option<String>,
        target_language: Option<String>,
    ) -> Result<Vec<GlossaryMatch>, RuntimeError> {
        Ok(self
            .runtime
            .glossary_matches(
                &text,
                source_language.as_deref(),
                target_language.as_deref(),
            )
            .await)
    }

    /// Which glossary rules `translated` breaks. Engines that cannot be
    /// constrained up front are checked here instead, so the result can be
    /// flagged rather than silently rewritten.
    pub async fn check(
        &self,
        source: String,
        translated: String,
        source_language: Option<String>,
        target_language: Option<String>,
    ) -> Result<Vec<GlossaryComplianceIssue>, RuntimeError> {
        let matches = self
            .runtime
            .glossary_matches(
                &source,
                source_language.as_deref(),
                target_language.as_deref(),
            )
            .await;
        Ok(check_compliance(&matches, &translated))
    }

    /// Writes any hit counts still held in memory. Worth calling before the
    /// app quits; everything else flushes on its own schedule.
    pub async fn flush_hits(&self) -> Result<(), RuntimeError> {
        self.runtime
            .inner
            .glossary
            .write()
            .await
            .flush_hits()
            .map_err(Into::into)
    }
}
