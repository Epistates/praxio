
use serde::Deserialize;
use std::collections::HashMap;

/// Top-level Gemini JSON response
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiJsonResponse {
    pub response: String,
    pub stats: GeminiStats,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    pub uuid: Option<String>,
    #[serde(rename = "numTurns")]
    pub num_turns: Option<u32>,
}

/// Main stats block
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiStats {
    pub models: HashMap<String, GeminiModelStats>,
    pub tools: GeminiToolStats,
    #[allow(dead_code)]
    pub files: GeminiFileStats,
}

/// Per-model stats
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiModelStats {
    pub api: GeminiApiStats,
    pub tokens: GeminiTokenStats,
}

/// API-level stats (latency, errors)
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiApiStats {
    #[serde(rename = "totalRequests")]
    #[allow(dead_code)]
    pub total_requests: u32,
    #[serde(rename = "totalErrors")]
    pub total_errors: u32,
    #[serde(rename = "totalLatencyMs")]
    pub total_latency_ms: u64,
}

/// Token usage stats
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiTokenStats {
    pub prompt: u32,
    pub candidates: u32,
    pub total: u32,
    pub cached: u32,
    pub thoughts: u32, // Extended thinking
    #[allow(dead_code)]
    pub tool: u32,
}

/// Tool usage stats
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiToolStats {
    #[serde(rename = "totalCalls")]
    pub total_calls: u32,
}

/// File modification stats
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiFileStats {
    #[serde(rename = "totalLinesAdded")]
    #[allow(dead_code)]
    pub total_lines_added: u32,
    #[serde(rename = "totalLinesRemoved")]
    #[allow(dead_code)]
    pub total_lines_removed: u32,
}
