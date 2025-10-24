use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Provider '{provider}' is unavailable: {reason}")]
    ProviderUnavailable { provider: String, reason: String },

    #[error("Authentication failed for {provider}: {message}")]
    AuthenticationFailed { provider: String, message: String },

    #[error("CLI execution failed: {command}\nExit code: {exit_code}\nStderr: {stderr}")]
    CliExecutionFailed {
        command: String,
        stderr: String,
        exit_code: i32,
    },

    #[error("Failed to parse {format} response: {source}")]
    ParseError {
        format: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Request timeout after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("Model '{model}' not available for provider '{provider}': {reason}")]
    ModelNotAvailable {
        model: String,
        provider: String,
        reason: String,
    },

    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    #[error("API error from {provider}: {message}")]
    ApiError { provider: String, message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// Convert LlmError to McpError via ServerError
impl From<LlmError> for turbomcp::McpError {
    fn from(err: LlmError) -> Self {
        // Use ServerError as intermediary since McpError implements From<ServerError>
        let server_err = turbomcp::ServerError::Internal(err.to_string());
        turbomcp::McpError::from(server_err)
    }
}
