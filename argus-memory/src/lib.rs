//! Argus Memory - Encrypted Persistent Memory
//!
//! Memories are encrypted before leaving the device.
//! Supports both local encrypted cache and remote Supabase storage.

mod memory;
mod store;

pub use memory::{Memory, MemoryType};
pub use store::MemoryStore;

/// Errors from memory operations
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Encryption error: {0}")]
    Encryption(#[from] argus_crypto::CryptoError),
    
    #[error("Memory not found: {0}")]
    NotFound(uuid::Uuid),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, MemoryError>;
