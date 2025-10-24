use async_trait::async_trait;

use super::types::{LlmRequest, LlmResponse};
use crate::error::LlmError;

/// Provider availability status
#[derive(Debug, Clone)]
pub enum ProviderAvailability {
    Available,
    Unavailable { reason: String },
}

/// Core abstraction for LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Invoke the LLM with a request
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;

    /// Check if this provider is available and ready to use
    async fn check_availability(&self) -> ProviderAvailability;

    /// Get the provider name
    fn name(&self) -> &str;
}
