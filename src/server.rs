use turbomcp::prelude::*;
use std::sync::Arc;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

use crate::llm::{ClaudeProvider, GeminiProvider, LlmProvider, LlmRequest, OutputFormat, ProviderAvailability};

#[derive(Clone)]
pub struct PraxioServer {
    claude: Arc<ClaudeProvider>,
    gemini: Arc<GeminiProvider>,
    sessions: Arc<RwLock<HashMap<String, PathBuf>>>,  // session_id -> temp_dir
}

impl PraxioServer {
    pub async fn new() -> Self {
        let claude = Arc::new(ClaudeProvider::new());
        let gemini = Arc::new(GeminiProvider::new());

        // Check provider availability
        match claude.check_availability().await {
            ProviderAvailability::Available => {
                tracing::info!("✅ Claude provider available");
            }
            ProviderAvailability::Unavailable { reason } => {
                tracing::warn!("⚠️  Claude provider unavailable: {}", reason);
            }
        }

        match gemini.check_availability().await {
            ProviderAvailability::Available => {
                tracing::info!("✅ Gemini provider available");
            }
            ProviderAvailability::Unavailable { reason } => {
                tracing::warn!("⚠️  Gemini provider unavailable: {}", reason);
            }
        }

        Self {
            claude,
            gemini,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[turbomcp::server(name = "praxio", version = "0.1.0")]
impl PraxioServer {
    /// Invoke Claude CLI for a task with full control over parameters
    #[tool(description = "Delegate a task to Claude CLI with session continuity, fallback, and timeout control")]
    async fn invoke_claude(
        &self,
        prompt: String,
        system_prompt: Option<String>,
        model: Option<String>,
        session_id: Option<String>,
        fallback_model: Option<String>,
        timeout_seconds: Option<u64>,
    ) -> McpResult<serde_json::Value> {
        // Determine temp directory for this session
        let temp_dir = if let Some(ref sid) = session_id {
            // Look up existing session
            let sessions = self.sessions.read().await;
            let dir = sessions.get(sid).cloned().ok_or_else(|| {
                McpError::from(ServerError::Internal(
                    format!("Session not found: {}", sid)
                ))
            })?;

            tracing::info!(
                "Resuming session {}: {}...",
                sid.chars().take(8).collect::<String>(),
                prompt.chars().take(50).collect::<String>()
            );
            dir
        } else {
            // Create new temp directory
            let new_id = uuid::Uuid::new_v4();
            let dir = std::env::temp_dir().join(format!("praxio-{}", new_id));

            tracing::info!(
                "Creating new session: {}...",
                prompt.chars().take(50).collect::<String>()
            );
            dir
        };

        let is_new_session = session_id.is_none();

        let request = LlmRequest {
            prompt,
            system_prompt,
            model,
            output_format: OutputFormat::Json,
            max_tokens: None,
            session_id,
            temp_dir: Some(temp_dir.clone()),
            fallback_model,
            timeout_seconds,
        };

        let start = std::time::Instant::now();
        let response = self.claude.invoke(request).await?;
        let elapsed = start.elapsed();

        // Store session mapping if this was a new session
        if is_new_session {
            if let Some(ref new_sid) = response.metadata.session_id {
                let mut sessions = self.sessions.write().await;
                sessions.insert(new_sid.clone(), temp_dir.clone());
                tracing::info!("Mapped session {} → {:?}",
                    new_sid.chars().take(8).collect::<String>(),
                    temp_dir
                );
            }
        }

        tracing::info!(
            "Claude response received in {}ms (API: {}ms)",
            elapsed.as_millis(),
            response.duration_ms
        );

        if let Some(cost) = response.cost_usd {
            tracing::info!("Cost: ${:.6}", cost);
        }

        if let Some(ref tokens) = response.tokens {
            tracing::info!(
                "Tokens: {} input, {} output, {} total",
                tokens.input, tokens.output, tokens.total
            );
        }

        Ok(serde_json::to_value(&response)?)
    }

    /// Invoke Gemini CLI for a task with session continuity
    #[tool(description = "Delegate a task to Gemini CLI with session continuity and timeout control")]
    async fn invoke_gemini(
        &self,
        prompt: String,
        system_prompt: Option<String>,
        model: Option<String>,
        session_id: Option<String>,
        timeout_seconds: Option<u64>,
    ) -> McpResult<serde_json::Value> {
        // Determine temp directory for this session
        let temp_dir = if let Some(ref sid) = session_id {
            // Resume: look up existing session
            let sessions = self.sessions.read().await;
            let dir = sessions.get(sid).cloned().ok_or_else(|| {
                McpError::from(ServerError::Internal(
                    format!("Session not found: {}", sid)
                ))
            })?;

            tracing::info!(
                "Resuming Gemini session {}: {}...",
                sid.chars().take(8).collect::<String>(),
                prompt.chars().take(50).collect::<String>()
            );
            dir
        } else {
            // New: create unique temp dir
            let new_id = uuid::Uuid::new_v4();
            let dir = std::env::temp_dir().join(format!("praxio-gemini-{}", new_id));

            tracing::info!(
                "Creating new Gemini session: {}...",
                prompt.chars().take(50).collect::<String>()
            );
            dir
        };

        let is_new_session = session_id.is_none();

        let request = LlmRequest {
            prompt,
            system_prompt,
            model,
            output_format: OutputFormat::Json,
            max_tokens: None,
            session_id,
            temp_dir: Some(temp_dir.clone()),
            fallback_model: None, // Not supported by Gemini CLI
            timeout_seconds,
        };

        let start = std::time::Instant::now();
        let response = self.gemini.invoke(request).await?;
        let elapsed = start.elapsed();

        // Store session mapping if this was a new session
        if is_new_session {
            if let Some(ref new_sid) = response.metadata.session_id {
                let mut sessions = self.sessions.write().await;
                sessions.insert(new_sid.clone(), temp_dir.clone());
                tracing::info!("Mapped Gemini session {} → {:?}",
                    new_sid.chars().take(8).collect::<String>(),
                    temp_dir
                );
            }
        }

        tracing::info!(
            "Gemini response received in {}ms (API: {}ms)",
            elapsed.as_millis(),
            response.duration_ms
        );

        if let Some(ref tokens) = response.tokens {
            tracing::info!(
                "Tokens: {} input, {} output, {} total ({} thoughts)",
                tokens.input,
                tokens.output,
                tokens.total,
                tokens.extended_thinking.unwrap_or(0)
            );
        }

        Ok(serde_json::to_value(&response)?)
    }
}
