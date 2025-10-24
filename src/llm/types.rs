use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Request to invoke an LLM
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub model: Option<String>,
    pub output_format: OutputFormat,
    pub max_tokens: Option<u32>,

    /// Optional session ID to continue a previous conversation
    /// When provided, the LLM will have context from previous calls in that session
    pub session_id: Option<String>,

    /// Temp directory where Claude CLI should execute
    /// Used for session isolation - each session has its own directory
    pub temp_dir: Option<PathBuf>,

    /// Fallback model if primary is overloaded (Claude only)
    pub fallback_model: Option<String>,

    /// Timeout in seconds for this specific request
    /// Overrides provider default if specified
    pub timeout_seconds: Option<u64>,
}

/// Output format for LLM response
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Text,
    Json,
}

/// Unified response from any LLM provider
#[derive(Debug, Clone, Serialize)]
pub struct LlmResponse {
    /// The actual response content
    pub content: String,

    /// Primary model used (highest output tokens)
    pub primary_model: String,

    /// All models involved in generating the response
    pub all_models_used: Vec<String>,

    /// Provider name (claude, gemini)
    pub provider: String,

    /// Token usage breakdown
    pub tokens: Option<TokenUsage>,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Cost in USD (only available from Claude)
    pub cost_usd: Option<f64>,

    /// Per-model breakdown (only from Claude)
    pub model_breakdown: Option<Vec<ModelBreakdown>>,

    /// Provider-specific metadata
    pub metadata: LlmResponseMetadata,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input: u32,
    pub output: u32,
    pub total: u32,
    pub cache_creation: u32,
    pub cache_read: u32,

    /// Extended thinking tokens (Gemini only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_thinking: Option<u32>,
}

/// Per-model token and cost breakdown (Claude only)
#[derive(Debug, Clone, Serialize)]
pub struct ModelBreakdown {
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: u32,
    pub cache_creation_tokens: u32,
    pub cost_usd: f64,
    pub context_window: u32,
}

/// Provider-specific metadata
#[derive(Debug, Clone, Serialize, Default)]
pub struct LlmResponseMetadata {
    /// Session ID (Claude)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// UUID (Claude)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,

    /// Number of turns (Claude)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_turns: Option<u32>,

    /// Service tier (Claude)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,

    /// API errors count (Gemini)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_errors: Option<u32>,

    /// Total tool calls (Gemini)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<u32>,
}
