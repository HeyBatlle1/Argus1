//! Encrypted local cache
//!
//! Memory is cached locally but always encrypted.
//! Even if someone gets your disk, they get garbage.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache not initialized")]
    NotInitialized,
    
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Encrypted local cache for agent memory
pub struct EncryptedCache {
    // TODO: Implement encrypted storage
}

impl EncryptedCache {
    pub fn new(_cache_dir: &std::path::Path) -> Result<Self, CacheError> {
        todo!("Initialize encrypted cache")
    }
}
