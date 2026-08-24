use async_trait::async_trait;

use crate::{DictionaryService, LlmError, LlmService, OcrService, TranslationService};

/// Runtime capabilities exposed by one configured service provider.
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;

    fn translation(&self) -> Option<&dyn TranslationService> {
        None
    }

    fn dictionary(&self) -> Option<&dyn DictionaryService> {
        None
    }

    fn ocr(&self) -> Option<&dyn OcrService> {
        None
    }

    fn llm(&self) -> Option<&dyn LlmService> {
        None
    }

    async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        Err(LlmError::UnsupportedOperation("list_models".to_owned()))
    }
}
