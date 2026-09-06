impl RuntimeTranslation {
    async fn translate_impl(&self, request: TranslateRequest) -> Result<TranslateResponse, String> {
        let service_id = self.service_id.clone();
        let runtime = self.runtime.clone();
        run_on_worker_thread(move || async move {
            let target_language =
                validate_optional_required("target_language", request.target_language)?;
            let text = validate_required("text", request.text)?;
            let source_language = optional_trimmed(request.source_language);
            let resolved = runtime
                .resolve_service(&service_id, ServiceType::Translation)
                .await?;
            let provider_id = resolved.provider_id.clone();
            let provider = {
                let state = runtime.inner.state.read().await;
                state
                    .engine
                    .require(&provider_id)
                    .map_err(|error| error.to_string())?
                    .clone()
            };

            // Terms are looked up once for both branches: an LLM gets them as
            // constraints, and either way their use is counted.
            let matches = runtime
                .glossary_matches(&text, source_language.as_deref(), Some(&target_language))
                .await;
            runtime.record_glossary_hits(&matches).await;

            if let Some(translation_service) = provider.translation() {
                // Use the dedicated translation service.
                translation_service
                    .translate(TranslateRequest {
                        source_language,
                        target_language: Some(target_language),
                        text,
                    })
                    .await
                    .map_err(|error| error.to_string())
            } else if let Some(llm_service) = provider.llm() {
                // LLM-based translation via prompts.
                let model = resolved
                    .field("model")
                    .map(str::to_owned)
                    .or_else(|| llm_service.available_models().into_iter().next())
                    .ok_or_else(|| "llm default model must be configured".to_owned())?;
                let terms = glossary_terms(&matches);
                let system_prompt = if let Some(system_prompt) = resolved.field("systemPrompt") {
                    render_prompt_template(
                        system_prompt,
                        source_language.as_deref().unwrap_or("auto"),
                        &target_language,
                        &text,
                        &terms,
                    )
                } else {
                    linguaray_engine::prompt::translate_text_system_prompt(
                        source_language.as_deref().unwrap_or("auto"),
                        &target_language,
                        None,
                        &terms,
                    )
                };
                let user_prompt = linguaray_engine::prompt::translate_text_user_prompt(&text);
                let response = llm_service
                    .chat(linguaray_core::ChatRequest {
                        model,
                        messages: vec![
                            ChatMessage::system(system_prompt),
                            ChatMessage::user(user_prompt),
                        ],
                        temperature: Some(0.3),
                        max_tokens: Some(4096),
                        stream: None,
                        response_format: None,
                    })
                    .await
                    .map_err(|error| error.to_string())?;
                let content = response
                    .choices
                    .first()
                    .map(|choice| choice.message.content.clone())
                    .ok_or_else(|| "no response from llm".to_owned())?;
                Ok(TranslateResponse {
                    translations: vec![linguaray_core::TextTranslation {
                        text: content,
                        detected_source_language: None,
                        audio_url: None,
                    }],
                })
            } else {
                Err(format!(
                    "provider `{provider_id}` does not support translation"
                ))
            }
        })
        .await
    }
}

impl RuntimeTranslation {
    async fn detect_language_impl(
        &self,
        request: DetectLanguageRequest,
    ) -> Result<DetectLanguageResponse, String> {
        let service_id = self.service_id.clone();
        let runtime = self.runtime.clone();
        run_on_worker_thread(move || async move {
            let texts = request
                .texts
                .into_iter()
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>();
            if texts.is_empty() {
                return Err("texts must not be empty".to_owned());
            }
            let resolved = runtime
                .resolve_service(&service_id, ServiceType::Translation)
                .await?;
            let provider_id = resolved.provider_id.clone();
            let provider = {
                let state = runtime.inner.state.read().await;
                state
                    .engine
                    .require(&provider_id)
                    .map_err(|error| error.to_string())?
                    .clone()
            };
            if let Some(translation_service) = provider.translation() {
                translation_service
                    .detect_language(DetectLanguageRequest { texts })
                    .await
                    .map_err(|error| error.to_string())
            } else if let Some(llm_service) = provider.llm() {
                // LLM-based language detection.
                let model = resolved
                    .field("model")
                    .map(str::to_owned)
                    .or_else(|| llm_service.available_models().into_iter().next())
                    .ok_or_else(|| "llm default model must be configured".to_owned())?;

                let text = texts.join(" ");
                let system_prompt = concat!(
                        "You are a language detection expert. ",
                        "Identify the language of the following text. ",
                        "Return ONLY the ISO 639-1 language code (e.g. \"en\", \"zh\", \"ja\", \"fr\", \"de\", \"es\"). ",
                        "If unsure, return \"auto\"."
                    )
                    .to_string();
                let user_prompt = format!("Detect the language of this text:\n\n{text}");

                let response = llm_service
                    .chat(linguaray_core::ChatRequest {
                        model,
                        messages: vec![
                            linguaray_core::ChatMessage::system(system_prompt),
                            linguaray_core::ChatMessage::user(user_prompt),
                        ],
                        temperature: Some(0.0),
                        max_tokens: Some(16),
                        stream: None,
                        response_format: None,
                    })
                    .await
                    .map_err(|error| error.to_string())?;

                let detected = response
                    .choices
                    .first()
                    .map(|choice| choice.message.content.trim().to_lowercase())
                    .filter(|code| !code.is_empty())
                    .unwrap_or_else(|| "auto".to_string());

                // Validate the detected code is a reasonable ISO 639-1 code
                let code = if detected.len() == 2 && detected.chars().all(|c| c.is_ascii_alphabetic()) {
                    detected
                } else {
                    "auto".to_string()
                };

                let detections: Vec<linguaray_core::TextDetection> = texts
                    .iter()
                    .map(|t| linguaray_core::TextDetection {
                        detected_language: code.clone(),
                        text: t.clone(),
                    })
                    .collect();

                Ok(DetectLanguageResponse {
                    detections: Some(detections),
                })
            } else {
                Err(format!(
                    "provider `{provider_id}` does not support translation"
                ))
            }
        })
        .await
    }
}

impl RuntimeDictionary {
    async fn lookup_impl(&self, request: LookUpRequest) -> Result<LookUpResponse, String> {
        let service_id = self.service_id.clone();
        let runtime = self.runtime.clone();
        run_on_worker_thread(move || async move {
            let source_language = validate_required("source_language", request.source_language)?;
            let target_language = validate_required("target_language", request.target_language)?;
            let word = validate_required("word", request.word)?;
            let resolved = runtime
                .resolve_service(&service_id, ServiceType::Dictionary)
                .await?;
            let provider_id = resolved.provider_id.clone();
            let provider = {
                let state = runtime.inner.state.read().await;
                state
                    .engine
                    .require(&provider_id)
                    .map_err(|error| error.to_string())?
                    .clone()
            };
            let dictionary_service = provider
                .dictionary()
                .ok_or_else(|| format!("provider `{provider_id}` does not support dictionary"))?;

            dictionary_service
                .look_up(LookUpRequest {
                    source_language,
                    target_language,
                    word,
                })
                .await
                .map_err(|error| error.to_string())
        })
        .await
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RuntimeTranslation {
    pub async fn translate(
        &self,
        request: TranslateRequest,
    ) -> Result<TranslateResponse, RuntimeError> {
        self.translate_impl(request).await.map_err(Into::into)
    }

    pub async fn detect_language(
        &self,
        request: DetectLanguageRequest,
    ) -> Result<DetectLanguageResponse, RuntimeError> {
        self.detect_language_impl(request).await.map_err(Into::into)
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RuntimeDictionary {
    pub async fn lookup(&self, request: LookUpRequest) -> Result<LookUpResponse, RuntimeError> {
        self.lookup_impl(request).await.map_err(Into::into)
    }
}
