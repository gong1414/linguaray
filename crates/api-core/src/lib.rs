mod error;
mod handlers;
mod normalization;

pub use error::{error_envelope, success_envelope};
pub use error::{ApiError, ErrorBody, ErrorResponse, SuccessResponse};
pub use handlers::{detect_language_request, lookup_request, translate_request};
pub use handlers::{health, index, openapi_document, reference_html, supported_language_pairs};
pub use handlers::{HealthResponse, IndexResponse};
