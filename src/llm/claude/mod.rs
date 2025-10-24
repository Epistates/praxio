mod types;

use async_trait::async_trait;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use super::provider::{LlmProvider, ProviderAvailability};
use super::types::{
    LlmRequest, LlmResponse, LlmResponseMetadata, ModelBreakdown, OutputFormat, TokenUsage,
};
use crate::error::LlmError;
use types::ClaudeJsonResponse;

/// Claude CLI provider
pub struct ClaudeProvider {
    timeout_seconds: u64,
}

impl ClaudeProvider {
    pub fn new() -> Self {
        Self {
            timeout_seconds: 30,
        }
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Build command for Claude CLI invocation
    fn build_command(&self, request: &LlmRequest) -> Command {
        let mut cmd = Command::new("claude");
        cmd.arg("--print");
        cmd.arg(&request.prompt);

        // Session management: use --resume for context continuity
        // Note: Use session_id from previous response's metadata.session_id
        if let Some(ref session_id) = request.session_id {
            cmd.arg("--resume").arg(session_id);
        }

        if let Some(ref sys_prompt) = request.system_prompt {
            cmd.arg("--system-prompt").arg(sys_prompt);
        }

        if let Some(ref model) = request.model {
            cmd.arg("--model").arg(model);
        }

        // Fallback model for reliability (only works with --print mode)
        if let Some(ref fallback) = request.fallback_model {
            cmd.arg("--fallback-model").arg(fallback);
        }

        // Always use JSON for metadata
        match request.output_format {
            OutputFormat::Json => {
                cmd.arg("--output-format").arg("json");
            }
            OutputFormat::Text => {
                cmd.arg("--output-format").arg("json");
            }
        }

        // Skip permissions for MCP usage (delegation context)
        // This is safe because the delegated Claude runs in an isolated subprocess
        cmd.arg("--dangerously-skip-permissions");

        // Note: current_dir will be set in invoke() to a unique temp directory
        cmd
    }

    /// Parse JSON response from Claude
    fn parse_json_response(&self, json_str: &str) -> Result<LlmResponse, LlmError> {
        let claude_resp: ClaudeJsonResponse = serde_json::from_str(json_str).map_err(|e| {
            LlmError::ParseError {
                format: "json".to_string(),
                source: Box::new(e),
            }
        })?;

        // Check if response is an error
        if claude_resp.is_error {
            return Err(LlmError::ApiError {
                provider: "claude".to_string(),
                message: claude_resp.result,
            });
        }

        // Extract primary model (one with highest output tokens)
        let primary_model = claude_resp
            .model_usage
            .iter()
            .max_by_key(|(_, usage)| usage.output_tokens)
            .map(|(model, _)| model.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Get all models used
        let all_models_used: Vec<String> = claude_resp.model_usage.keys().cloned().collect();

        // Build model breakdown
        let model_breakdown: Vec<ModelBreakdown> = claude_resp
            .model_usage
            .into_iter()
            .map(|(model, usage)| ModelBreakdown {
                model,
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                cache_read_tokens: usage.cache_read_input_tokens,
                cache_creation_tokens: usage.cache_creation_input_tokens,
                cost_usd: usage.cost_usd,
                context_window: usage.context_window,
            })
            .collect();

        // Calculate total tokens
        let total_tokens = TokenUsage {
            input: claude_resp.usage.input_tokens,
            output: claude_resp.usage.output_tokens,
            total: claude_resp.usage.input_tokens + claude_resp.usage.output_tokens,
            cache_creation: claude_resp.usage.cache_creation_input_tokens,
            cache_read: claude_resp.usage.cache_read_input_tokens,
            extended_thinking: None,
        };

        Ok(LlmResponse {
            content: claude_resp.result,
            primary_model,
            all_models_used,
            provider: "claude".to_string(),
            tokens: Some(total_tokens),
            duration_ms: claude_resp.duration_ms,
            cost_usd: Some(claude_resp.total_cost_usd),
            model_breakdown: Some(model_breakdown),
            metadata: LlmResponseMetadata {
                session_id: Some(claude_resp.session_id),
                uuid: Some(claude_resp.uuid),
                num_turns: Some(claude_resp.num_turns),
                service_tier: Some(claude_resp.usage.service_tier),
                api_errors: None,
                tool_calls: None,
            },
        })
    }

    /// Classify error from stderr
    fn classify_error(&self, stderr: &str, exit_code: i32) -> LlmError {
        if stderr.contains("Authentication failed") || stderr.contains("setup-token") {
            LlmError::AuthenticationFailed {
                provider: "claude".to_string(),
                message: stderr.to_string(),
            }
        } else if stderr.contains("not found") || exit_code == 127 {
            LlmError::ProviderUnavailable {
                provider: "claude".to_string(),
                reason: "CLI not found in PATH".to_string(),
            }
        } else {
            LlmError::CliExecutionFailed {
                command: "claude".to_string(),
                stderr: stderr.to_string(),
                exit_code,
            }
        }
    }
}

impl Default for ClaudeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        // Use temp directory from request (managed by server)
        // Each session has its own isolated directory
        let temp_dir = request.temp_dir.clone().unwrap_or_else(|| {
            std::env::temp_dir().join("praxio-default")
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
        self.parse_json_response(&stdout)
    }

    async fn check_availability(&self) -> ProviderAvailability {
        // Check if CLI exists
        let cli_check = Command::new("which")
            .arg("claude")
            .output()
            .await;

        match cli_check {
            Ok(output) if output.status.success() => {
                // CLI exists, try to get version
                let version_check = Command::new("claude")
                    .arg("--version")
                    .output()
                    .await;

                match version_check {
                    Ok(output) if output.status.success() => ProviderAvailability::Available,
                    Ok(_) => ProviderAvailability::Unavailable {
                        reason: "claude CLI found but not responding correctly".to_string(),
                    },
                    Err(e) => ProviderAvailability::Unavailable {
                        reason: format!("claude CLI error: {}", e),
                    },
                }
            }
            _ => ProviderAvailability::Unavailable {
                reason: "claude CLI not found in PATH".to_string(),
            },
        }
    }

    fn name(&self) -> &str {
        "claude"
    }
}
