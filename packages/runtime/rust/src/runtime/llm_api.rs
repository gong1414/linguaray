/// One event read on the caller's async executor; no foreign-thread Dart callbacks.
#[derive(Clone, uniffi::Record)]
pub struct TranslationEvent {
    pub content: String,
    pub finish_reason: Option<String>,
    pub error: Option<String>,
}

#[derive(uniffi::Object)]
pub struct TranslationTask {
    cancelled: std::sync::atomic::AtomicBool,
    wake: tokio::sync::Notify,
    events: tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<TranslationEvent>>,
}

#[uniffi::export]
impl TranslationTask {
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Release);
        self.wake.notify_waiters();
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl TranslationTask {
    pub async fn next(&self) -> Option<TranslationEvent> {
        let mut events = self.events.lock().await;
        tokio::select! {
            biased;
            _ = self.wait_cancelled() => None,
            event = events.recv() => event,
        }
    }
}

impl TranslationTask {
    fn new() -> (
        Arc<Self>,
        tokio::sync::mpsc::UnboundedSender<TranslationEvent>,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        (
            Arc::new(Self {
                cancelled: std::sync::atomic::AtomicBool::new(false),
                wake: tokio::sync::Notify::new(),
                events: tokio::sync::Mutex::new(receiver),
            }),
            sender,
        )
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Acquire)
    }

    async fn wait_cancelled(&self) {
        let notified = self.wake.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        if !self.is_cancelled() {
            notified.await;
        }
    }
}

struct TranslationQueueCallback(tokio::sync::mpsc::UnboundedSender<TranslationEvent>);
impl StreamCallback for TranslationQueueCallback {
    fn on_chunk(&self, content: String) {
        let _ = self.0.send(TranslationEvent {
            content,
            finish_reason: None,
            error: None,
        });
    }
    fn on_finish(&self, reason: String) {
        let _ = self.0.send(TranslationEvent {
            content: String::new(),
            finish_reason: Some(reason),
            error: None,
        });
    }
    fn on_error(&self, error: String) {
        let _ = self.0.send(TranslationEvent {
            content: String::new(),
            finish_reason: None,
            error: Some(error),
        });
    }
}

impl RuntimeLlm {
    async fn chat_impl(
        &self,
        model: String,
        messages: Vec<linguaray_core::ChatMessage>,
    ) -> Result<linguaray_core::ChatResponse, String> {
        let service_id = self.service_id.clone();
        let runtime = self.runtime.clone();
        run_on_worker_thread(move || async move {
            let model = validate_required("model", model)?;
            if messages.is_empty() {
                return Err("messages must not be empty".to_owned());
            }
            let resolved = runtime.resolve_llm_service(&service_id).await?;
            let provider_id = resolved.provider_id;
            let provider = {
                let state = runtime.inner.state.read().await;
                state
                    .engine
                    .require(&provider_id)
                    .map_err(|error| error.to_string())?
                    .clone()
            };
            let llm_service = provider
                .llm()
                .ok_or_else(|| format!("provider `{provider_id}` does not support llm"))?;

            llm_service
                .chat(linguaray_core::ChatRequest {
                    model,
                    messages,
                    temperature: None,
                    max_tokens: None,
                    stream: None,
                    response_format: None,
                })
                .await
                .map_err(|error| error.to_string())
        })
        .await
    }

    async fn polish_impl(&self, text: String, style: String) -> Result<String, String> {
        let service_id = self.service_id.clone();
        let runtime = self.runtime.clone();
        run_on_worker_thread(move || async move {
            let text = validate_required("text", text)?;
            let style = validate_required("style", style)?;
            let resolved = runtime.resolve_llm_service(&service_id).await?;
            let provider_id = resolved.provider_id.clone();
            let provider = {
                let state = runtime.inner.state.read().await;
                state
                    .engine
                    .require(&provider_id)
                    .map_err(|error| error.to_string())?
                    .clone()
            };
            let llm_service = provider
                .llm()
                .ok_or_else(|| format!("provider `{provider_id}` does not support llm"))?;

            let model = resolved
                .field("model")
                .map(str::to_owned)
                .or_else(|| llm_service.available_models().into_iter().next())
                .ok_or_else(|| "llm default model must be configured".to_owned())?;
            let system_prompt = linguaray_engine::prompt::polish_translation_system_prompt(&style);

            let response = llm_service
                .chat(linguaray_core::ChatRequest {
                    model,
                    messages: vec![
                        linguaray_core::ChatMessage::system(system_prompt),
                        linguaray_core::ChatMessage::user(text),
                    ],
                    temperature: None,
                    max_tokens: None,
                    stream: None,
                    response_format: None,
                })
                .await
                .map_err(|error| error.to_string())?;

            response
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .ok_or_else(|| "no response from llm".to_owned())
        })
        .await
    }

    async fn explain_impl(&self, source: String, translation: String) -> Result<String, String> {
        let service_id = self.service_id.clone();
        let runtime = self.runtime.clone();
        run_on_worker_thread(move || async move {
            let source = validate_required("source", source)?;
            let translation = validate_required("translation", translation)?;
            let resolved = runtime.resolve_llm_service(&service_id).await?;
            let provider_id = resolved.provider_id.clone();
            let provider = {
                let state = runtime.inner.state.read().await;
                state
                    .engine
                    .require(&provider_id)
                    .map_err(|error| error.to_string())?
                    .clone()
            };
            let llm_service = provider
                .llm()
                .ok_or_else(|| format!("provider `{provider_id}` does not support llm"))?;

            let model = resolved
                .field("model")
                .map(str::to_owned)
                .or_else(|| llm_service.available_models().into_iter().next())
                .ok_or_else(|| "llm default model must be configured".to_owned())?;
            let system_prompt = linguaray_engine::prompt::explain_translation_system_prompt();
            let user_prompt = format!("Source text: {source}\n\nTranslation: {translation}");

            let response = llm_service
                .chat(linguaray_core::ChatRequest {
                    model,
                    messages: vec![
                        linguaray_core::ChatMessage::system(system_prompt),
                        linguaray_core::ChatMessage::user(user_prompt),
                    ],
                    temperature: None,
                    max_tokens: None,
                    stream: None,
                    response_format: None,
                })
                .await
                .map_err(|error| error.to_string())?;

            response
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .ok_or_else(|| "no response from llm".to_owned())
        })
        .await
    }

    async fn alternatives_impl(
        &self,
        text: String,
        source_lang: String,
        target_lang: String,
        count: u32,
        style: Option<String>,
    ) -> Result<Vec<String>, String> {
        let service_id = self.service_id.clone();
        let runtime = self.runtime.clone();
        run_on_worker_thread(move || async move {
            let text = validate_required("text", text)?;
            let source_lang = validate_required("source_lang", source_lang)?;
            let target_lang = validate_required("target_lang", target_lang)?;
            if count == 0 {
                return Err("count must be greater than 0".to_owned());
            }
            let resolved = runtime.resolve_llm_service(&service_id).await?;
            let provider_id = resolved.provider_id.clone();
            let provider = {
                let state = runtime.inner.state.read().await;
                state
                    .engine
                    .require(&provider_id)
                    .map_err(|error| error.to_string())?
                    .clone()
            };
            let llm_service = provider
                .llm()
                .ok_or_else(|| format!("provider `{provider_id}` does not support llm"))?;

            let model = resolved
                .field("model")
                .map(str::to_owned)
                .or_else(|| llm_service.available_models().into_iter().next())
                .ok_or_else(|| "llm default model must be configured".to_owned())?;
            let system_prompt = linguaray_engine::prompt::alternative_translations_system_prompt(
                count,
                style.as_deref(),
            );
            let user_prompt = format!(
                "Source language: {source_lang}\nTarget language: {target_lang}\nText: {text}"
            );

            let response = llm_service
                .chat(linguaray_core::ChatRequest {
                    model,
                    messages: vec![
                        linguaray_core::ChatMessage::system(system_prompt),
                        linguaray_core::ChatMessage::user(user_prompt),
                    ],
                    temperature: None,
                    max_tokens: None,
                    stream: None,
                    response_format: None,
                })
                .await
                .map_err(|error| error.to_string())?;

            let content = response
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .ok_or_else(|| "no response from llm".to_owned())?;

            parse_alternatives_json(&content)
        })
        .await
    }

    async fn translate_stream_impl(
        &self,
        source_lang: String,
        target_lang: String,
        text: String,
        callback: Arc<dyn StreamCallback>,
        cancellation: Arc<TranslationTask>,
    ) -> Result<(), String> {
        let service_id = self.service_id.clone();
        let runtime = self.runtime.clone();
        run_on_worker_thread(move || async move {
            let worker_cancellation = cancellation.clone();
            let operation = async move {
                let text = validate_required("text", text)?;
                let resolved = match runtime
                    .resolve_service(&service_id, ServiceType::Translation)
                    .await
                {
                    Ok(service) => service,
                    Err(_) => runtime.resolve_llm_service(&service_id).await?,
                };
                let provider_id = resolved.provider_id.clone();
                let provider = {
                    let state = runtime.inner.state.read().await;
                    state
                        .engine
                        .require(&provider_id)
                        .map_err(|error| error.to_string())?
                        .clone()
                };
                let matches = runtime
                    .glossary_matches(&text, Some(&source_lang), Some(&target_lang))
                    .await;
                runtime.record_glossary_hits(&matches).await;

                if let Some(llm_service) = provider.llm() {
                    // LLM-based streaming translation
                    let model = resolved
                        .field("model")
                        .map(str::to_owned)
                        .or_else(|| llm_service.available_models().into_iter().next())
                        .ok_or_else(|| "llm default model must be configured".to_owned())?;
                    let terms = glossary_terms(&matches);
                    let system_prompt = if let Some(system_prompt) = resolved.field("systemPrompt")
                    {
                        render_prompt_template(
                            system_prompt,
                            &source_lang,
                            &target_lang,
                            &text,
                            &terms,
                        )
                    } else {
                        linguaray_engine::prompt::translate_text_system_prompt(
                            &source_lang,
                            &target_lang,
                            None,
                            &terms,
                        )
                    };
                    let user_prompt = linguaray_engine::prompt::translate_text_user_prompt(&text);

                    let receiver = llm_service
                        .chat_stream(linguaray_core::ChatRequest {
                            model,
                            messages: vec![
                                linguaray_core::ChatMessage::system(system_prompt),
                                linguaray_core::ChatMessage::user(user_prompt),
                            ],
                            temperature: Some(0.3),
                            max_tokens: Some(4096),
                            stream: Some(true),
                            response_format: None,
                        })
                        .await
                        .map_err(|error| error.to_string())?;

                    tokio::task::spawn_blocking(move || loop {
                        if worker_cancellation.is_cancelled() {
                            break;
                        }
                        match receiver
                            .rx
                            .recv_timeout(std::time::Duration::from_millis(40))
                        {
                            Ok(chunk) => {
                                if chunk.finish_reason.as_deref() == Some("error") {
                                    callback.on_error(chunk.content);
                                    break;
                                }
                                if !chunk.content.is_empty() {
                                    callback.on_chunk(chunk.content);
                                }
                                if let Some(reason) = chunk.finish_reason {
                                    callback.on_finish(reason);
                                    break;
                                }
                            }
                            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                                callback.on_error("stream ended before completion".into());
                                break;
                            }
                        }
                    })
                    .await
                    .map_err(|error| format!("stream worker failed: {error}"))?;
                } else {
                    // Fallback to non-streaming translation via the translation service
                    let translation_service = provider.translation().ok_or_else(|| {
                        format!("provider `{provider_id}` does not support translation")
                    })?;
                    let response = translation_service
                        .translate(linguaray_core::TranslateRequest {
                            source_language: (source_lang != "auto").then_some(source_lang.clone()),
                            target_language: Some(target_lang),
                            text: text.clone(),
                        })
                        .await
                        .map_err(|error| error.to_string())?;
                    for translation in response.translations {
                        callback.on_chunk(translation.text);
                    }
                    callback.on_finish("stop".to_string());
                }

                Ok(())
            };
            tokio::select! {
                biased;
                _ = cancellation.wait_cancelled() => Ok(()),
                result = operation => result,
            }
        })
        .await
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RuntimeLlm {
    pub async fn chat(
        &self,
        model: String,
        messages: Vec<linguaray_core::ChatMessage>,
    ) -> Result<linguaray_core::ChatResponse, RuntimeError> {
        self.chat_impl(model, messages).await.map_err(Into::into)
    }

    pub async fn polish(&self, text: String, style: String) -> Result<String, RuntimeError> {
        self.polish_impl(text, style).await.map_err(Into::into)
    }

    pub async fn explain(
        &self,
        source: String,
        translation: String,
    ) -> Result<String, RuntimeError> {
        self.explain_impl(source, translation)
            .await
            .map_err(Into::into)
    }

    pub async fn alternatives(
        &self,
        text: String,
        source_lang: String,
        target_lang: String,
        count: u32,
        style: Option<String>,
    ) -> Result<Vec<String>, RuntimeError> {
        self.alternatives_impl(text, source_lang, target_lang, count, style)
            .await
            .map_err(Into::into)
    }
}

#[uniffi::export]
impl RuntimeLlm {
    /// Legacy callback API retained for existing native clients.
    /// Dart clients must use start_translation and TranslationTask.next.
    pub fn translate_stream(
        &self,
        source_lang: String,
        target_lang: String,
        text: String,
        callback: Box<dyn StreamCallback>,
    ) {
        let (task, _) = TranslationTask::new();
        spawn_translation_task(
            self.clone(),
            task,
            source_lang,
            target_lang,
            text,
            callback.into(),
        );
    }

    pub fn start_translation(
        &self,
        source_lang: String,
        target_lang: String,
        text: String,
    ) -> Arc<TranslationTask> {
        let (task, sender) = TranslationTask::new();
        spawn_translation_task(
            self.clone(),
            task.clone(),
            source_lang,
            target_lang,
            text,
            Arc::new(TranslationQueueCallback(sender)),
        );
        task
    }
}

fn spawn_translation_task(
    this: RuntimeLlm,
    cancellation: Arc<TranslationTask>,
    source_lang: String,
    target_lang: String,
    text: String,
    callback: Arc<dyn StreamCallback>,
) {
    let callback_for_worker = callback.clone();
    if let Err(error) = thread::Builder::new()
        .name("linguaray-translation".to_owned())
        .spawn(move || {
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| format!("failed to build tokio runtime: {error}"))
                .and_then(|runtime| {
                    runtime.block_on(this.translate_stream_impl(
                        source_lang,
                        target_lang,
                        text,
                        callback_for_worker.clone(),
                        cancellation.clone(),
                    ))
                });
            if !cancellation.is_cancelled() {
                if let Err(error) = result {
                    callback_for_worker.on_error(error);
                }
            }
        })
    {
        callback.on_error(format!("failed to spawn runtime worker thread: {error}"));
    }
}

fn parse_alternatives_json(content: &str) -> Result<Vec<String>, String> {
    #[derive(serde::Deserialize)]
    struct AlternativesContainer {
        alternatives: Vec<AlternativeEntry>,
    }

    #[derive(serde::Deserialize)]
    struct AlternativeEntry {
        text: String,
    }

    let parsed: AlternativesContainer = serde_json::from_str(content)
        .map_err(|error| format!("failed to parse alternatives response: {error}"))?;

    Ok(parsed.alternatives.into_iter().map(|a| a.text).collect())
}
