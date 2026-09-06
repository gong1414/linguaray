impl RuntimeOcr {
    async fn recognize_text_impl(
        &self,
        request: RecognizeTextRequest,
    ) -> Result<RecognizeTextResponse, String> {
        let service_id = self.service_id.clone();
        let runtime = self.runtime.clone();
        run_on_worker_thread(move || async move {
            let resolved = runtime
                .resolve_service(&service_id, ServiceType::Ocr)
                .await?;
            let provider_id = resolved.provider_id;
            let provider = {
                let state = runtime.inner.state.read().await;
                state
                    .engine
                    .require(&provider_id)
                    .map_err(|error| error.to_string())?
                    .clone()
            };
            let ocr_service = provider
                .ocr()
                .ok_or_else(|| format!("provider `{provider_id}` does not support ocr"))?;

            ocr_service
                .recognize_text(request)
                .await
                .map_err(|error| error.to_string())
        })
        .await
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RuntimeOcr {
    pub async fn recognize_text(
        &self,
        request: RecognizeTextRequest,
    ) -> Result<RecognizeTextResponse, RuntimeError> {
        self.recognize_text_impl(request).await.map_err(Into::into)
    }

    /// Recognizes an image currently stored in the system clipboard.
    pub async fn recognize_clipboard_image(&self) -> Result<RecognizeTextResponse, RuntimeError> {
        let image = text_extractor::extract_image_from_clipboard().map_err(|error| {
            RuntimeError::Error {
                msg: error.to_string(),
            }
        })?;
        self.recognize_text_impl(RecognizeTextRequest {
            image_path: None,
            base64_image: Some(image),
        })
        .await
        .map_err(Into::into)
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RuntimePermission {
    /// macOS only: check if Screen Recording permission is granted.
    /// Returns `true` on other platforms.
    pub async fn is_screen_recording_permission_granted(&self) -> bool {
        permission::is_screen_recording_permission_granted()
    }

    /// macOS only: request Screen Recording permission.
    /// If `only_open_system_settings` is true, just opens System Settings.
    /// No-op on other platforms.
    pub async fn request_screen_recording_permission(&self, only_open_system_settings: bool) {
        permission::request_screen_recording_permission(only_open_system_settings);
    }

    /// macOS only: check if Accessibility permission is granted.
    /// Returns `true` on other platforms.
    pub async fn is_accessibility_permission_granted(&self) -> bool {
        permission::is_accessibility_permission_granted()
    }

    /// macOS only: request Accessibility permission.
    /// If `only_open_system_settings` is true, just opens System Settings.
    /// No-op on other platforms.
    pub async fn request_accessibility_permission(&self, only_open_system_settings: bool) {
        permission::request_accessibility_permission(only_open_system_settings);
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RuntimeTextExtractor {
    /// Read the current clipboard text.
    pub async fn extract_from_clipboard(&self) -> Result<String, RuntimeError> {
        text_extractor::extract_from_clipboard()
            .map_err(|e| RuntimeError::Error { msg: e.to_string() })
    }

    /// Extract text from the current screen selection.
    ///
    /// **macOS / Windows:** Simulates Cmd+C / Ctrl+C, polls the clipboard
    /// until content changes (or 3s timeout), then returns the text.
    pub async fn extract_from_screen_selection(&self) -> Result<String, RuntimeError> {
        text_extractor::extract_from_screen_selection()
            .map_err(|e| RuntimeError::Error { msg: e.to_string() })
    }

    /// Detailed selection extraction with method and non-fatal recovery
    /// warnings. The clipboard is restored before this future completes.
    pub async fn extract_from_screen_selection_detailed(
        &self,
    ) -> Result<text_extractor::SelectionExtraction, RuntimeError> {
        text_extractor::extract_from_screen_selection_detailed()
            .map_err(|e| RuntimeError::Error { msg: e.to_string() })
    }
}
