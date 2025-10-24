mod types;

use async_trait::async_trait;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use super::provider::{LlmProvider, ProviderAvailability};
use super::types::{
    LlmRequest, LlmResponse, LlmResponseMetadata, TokenUsage,
};
use crate::error::LlmError;
use types::GeminiJsonResponse;

/// Gemini CLI provider
pub struct GeminiProvider {
    timeout_seconds: u64,
}

impl GeminiProvider {
    pub fn new() -> Self {
        // Gemini can be slower, so default to a longer timeout
        Self {
            timeout_seconds: 60,
        }
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Build command for Gemini CLI invocation
    fn build_command(&self, request: &LlmRequest) -> Command {
        let mut cmd = Command::new("gemini");
        cmd.arg(&request.prompt);

        // Session management: use --resume for context continuity
        if let Some(ref session_id) = request.session_id {
            cmd.arg("--resume").arg(session_id);
        }

        if let Some(ref system_prompt) = request.system_prompt {
            cmd.arg("--system-prompt").arg(system_prompt);
        }

        if let Some(ref model) = request.model {
            cmd.arg("--model").arg(model);
        }

        // Always use JSON for metadata
        cmd.arg("--output-format").arg("json");

        cmd
    }

    /// Clean stdout from Gemini CLI
    fn clean_stdout(&self, stdout: &str) -> String {
        stdout
            .lines()
            .filter(|line| !line.starts_with("Loaded cached credentials"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Parse JSON response from Gemini
    fn parse_json_response(&self, json_str: &str) -> Result<LlmResponse, LlmError> {
        let gemini_resp: GeminiJsonResponse = serde_json::from_str(json_str).map_err(|e| {
            LlmError::ParseError {
                format: "json".to_string(),
                source: Box::new(e),
            }
        })?;

        // Extract the primary model and its stats (should only be one)
        let (model_name, model_stats) =
            gemini_resp.stats.models.iter().next().ok_or_else(|| {
                LlmError::ParseError {
                    format: "json".to_string(),
                    source: "No model stats found in Gemini response".into(),
                }
            })?;

        // Calculate total tokens
        let total_tokens = TokenUsage {
            input: model_stats.tokens.prompt,
            output: model_stats.tokens.candidates,
            total: model_stats.tokens.total,
            cache_creation: 0, // Not provided by Gemini
            cache_read: model_stats.tokens.cached,
            extended_thinking: Some(model_stats.tokens.thoughts),
        };

        Ok(LlmResponse {
            content: gemini_resp.response,
            primary_model: model_name.clone(),
            all_models_used: vec![model_name.clone()],
            provider: "gemini".to_string(),
            tokens: Some(total_tokens),
            duration_ms: model_stats.api.total_latency_ms,
            cost_usd: None, // Not provided by Gemini CLI
            model_breakdown: None, // Gemini uses single model per request
            metadata: LlmResponseMetadata {
                session_id: gemini_resp.session_id,
                uuid: gemini_resp.uuid,
                num_turns: gemini_resp.num_turns,
                service_tier: None, // Not provided by Gemini
                api_errors: Some(model_stats.api.total_errors),
                tool_calls: Some(gemini_resp.stats.tools.total_calls),
            },
        })
    }

    /// Classify error from stderr
    fn classify_error(&self, stderr: &str, exit_code: i32) -> LlmError {
        if stderr.contains("GEMINI_API_KEY environment variable not found") {
            LlmError::ProviderUnavailable {
                provider: "gemini".to_string(),
                reason: "GEMINI_API_KEY environment variable not set".to_string(),
            }
        } else if stderr.contains("Error when talking to Gemini API") {
            LlmError::ApiError {
                provider: "gemini".to_string(),
                message: stderr.to_string(),
            }
        } else if stderr.contains("not found") || exit_code == 127 {
            LlmError::ProviderUnavailable {
                provider: "gemini".to_string(),
                reason: "CLI not found in PATH".to_string(),
            }
        } else {
            LlmError::CliExecutionFailed {
                command: "gemini".to_string(),
                stderr: stderr.to_string(),
                exit_code,
            }
        }
    }
}

impl Default for GeminiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        // Use temp directory from request (managed by server)
        // Each session has its own isolated directory
        let temp_dir = request.temp_dir.clone().unwrap_or_else(|| {
            std::env::temp_dir().join("praxio-gemini-default")
        });
        std::fs::create_dir_all(&temp_dir).map_err(LlmError::Io)?;

        let mut cmd = self.build_command(&request);
        cmd.current_dir(&temp_dir);

        // Explicitly configure stdio - close stdin, capture stdout/stderr
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Use timeout from request or provider default
        let timeout_secs = request.timeout_seconds.unwrap_or(self.timeout_seconds);

        // Execute with timeout
        let output = timeout(Duration::from_secs(timeout_secs), cmd.output())
            .await
            .map_err(|_| LlmError::Timeout {
                seconds: timeout_secs,
            })?
            .map_err(LlmError::Io)?;

        // Clean up temp directory
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Check exit status
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);
            return Err(self.classify_error(&stderr, exit_code));
        }

        // Parse response
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let cleaned_stdout = self.clean_stdout(&stdout);
        self.parse_json_response(&cleaned_stdout)
    }

    async fn check_availability(&self) -> ProviderAvailability {
        // 1. Check for GEMINI_API_KEY
        if std::env::var("GEMINI_API_KEY").is_err() {
            return ProviderAvailability::Unavailable {
                reason: "GEMINI_API_KEY environment variable not set".to_string(),
            };
        }

        // 2. Check if CLI exists
        let cli_check = Command::new("which").arg("gemini").output().await;

        match cli_check {
            Ok(output) if output.status.success() => ProviderAvailability::Available,
            _ => ProviderAvailability::Unavailable {
                reason: "gemini CLI not found in PATH".to_string(),
            },
        }
    }

    fn name(&self) -> &str {
        "gemini"
    }
}
