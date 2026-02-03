//! LLM provider abstraction

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Configuration for an LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 8192,
            temperature: 0.7,
        }
    }
}

/// A response from the LLM
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub stop_reason: StopReason,
}

/// A tool call requested by the LLM
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Why the LLM stopped generating
#[derive(Debug, Clone, Copy)]
pub enum StopReason {
    EndTurn,
    ToolUse,
    MaxTokens,
}

/// Trait for LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a prompt and get a response
    async fn complete(&self, prompt: &str) -> crate::Result<LlmResponse>;
    
    /// Get the provider name
    fn name(&self) -> &str;
    
    /// Get the current model
    fn model(&self) -> &str;
}

/// Claude/Anthropic provider (placeholder)
pub struct ClaudeProvider {
    config: LlmConfig,
    // api_key retrieved from vault at runtime
}

impl ClaudeProvider {
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn complete(&self, _prompt: &str) -> crate::Result<LlmResponse> {
        // TODO: Implement actual API call
        // API key comes from SecureVault, never stored here
        Ok(LlmResponse {
            content: "Implementation pending".to_string(),
            tool_calls: vec![],
            stop_reason: StopReason::EndTurn,
        })
    }
    
    fn name(&self) -> &str {
        "anthropic"
    }
    
    fn model(&self) -> &str {
        &self.config.model
    }
}
