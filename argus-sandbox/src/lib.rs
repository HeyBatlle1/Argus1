//! Argus Sandbox - WebAssembly Tool Isolation
//!
//! Every tool execution runs in a WebAssembly sandbox.
//! No filesystem access. No network access. No escape.

mod runtime;
mod capabilities;

pub use runtime::SandboxRuntime;
pub use capabilities::{Capability, CapabilitySet};

/// Errors from sandbox operations
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Failed to compile WASM module: {0}")]
    CompilationFailed(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Timeout after {0}ms")]
    Timeout(u64),
}

pub type Result<T> = std::result::Result<T, SandboxError>;
