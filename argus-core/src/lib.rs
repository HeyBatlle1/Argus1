//! Argus Core - Agent Orchestration Runtime
//!
//! The brain of Argus. Coordinates memory, tools, and LLM interactions
//! while maintaining security invariants.

mod agent;
mod context;
mod llm;

pub use agent::Agent;
pub use context::ConversationContext;
pub use llm::{LlmProvider, LlmConfig};

/// Core errors
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Memory error: {0}")]
    Memory(#[from] argus_memory::MemoryError),
    
    #[error("Crypto error: {0}")]
    Crypto(#[from] argus_crypto::CryptoError),
    
    #[error("Sandbox error: {0}")]
    Sandbox(#[from] argus_sandbox::SandboxError),
    
    #[error("LLM error: {0}")]
    Llm(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
