use serde::Deserialize;
use std::collections::HashMap;

/// Top-level Claude JSON response
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeJsonResponse {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub response_type: String,
    #[allow(dead_code)]
    pub subtype: String,
    pub is_error: bool,
    pub duration_ms: u64,
    #[allow(dead_code)]
    pub duration_api_ms: u64,
    pub num_turns: u32,
    pub result: String, // The actual content
    pub session_id: String,
    pub total_cost_usd: f64,
    pub usage: ClaudeUsage,
    #[serde(rename = "modelUsage")]
    pub model_usage: HashMap<String, ClaudeModelUsage>,
    #[allow(dead_code)]
    pub permission_denials: Vec<String>,
    pub uuid: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeUsage {
    pub input_tokens: u32,
    pub cache_creation_input_tokens: u32,
    pub cache_read_input_tokens: u32,
    pub output_tokens: u32,
    pub service_tier: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeModelUsage {
    #[serde(rename = "inputTokens")]
    pub input_tokens: u32,
    #[serde(rename = "outputTokens")]
    pub output_tokens: u32,
    #[serde(rename = "cacheReadInputTokens")]
    pub cache_read_input_tokens: u32,
    #[serde(rename = "cacheCreationInputTokens")]
    pub cache_creation_input_tokens: u32,
    #[serde(rename = "costUSD")]
    pub cost_usd: f64,
    #[serde(rename = "contextWindow")]
    pub context_window: u32,
}
