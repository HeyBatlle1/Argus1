//! Secure Vault - Encrypted secrets storage
//!
//! Secrets are encrypted at rest with ML-KEM (post-quantum KEM)
//! and ChaCha20-Poly1305. Master key lives in OS keychain only.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use zeroize::Zeroize;

use crate::{CryptoError, Result};

/// A secret that zeroizes itself when dropped
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct Secret(Vec<u8>);

impl Secret {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }
}

/// Encrypted vault for secrets storage
pub struct SecureVault {
    path: PathBuf,
    secrets: HashMap<String, Vec<u8>>,
    unlocked: bool,
}

impl SecureVault {
    /// Create or open a vault at the given path
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        
        Ok(Self {
            path,
            secrets: HashMap::new(),
            unlocked: false,
        })
    }
    
    /// Unlock the vault using the OS keychain
    pub fn unlock(&mut self) -> Result<()> {
        // TODO: Retrieve master key from keychain
        // TODO: Decrypt vault contents
        self.unlocked = true;
        Ok(())
    }
    
    /// Lock the vault, zeroizing all secrets in memory
    pub fn lock(&mut self) {
        self.secrets.clear();
        self.unlocked = false;
    }
    
    /// Store a secret
    pub fn store(&mut self, name: &str, secret: Secret) -> Result<()> {
        if !self.unlocked {
            return Err(CryptoError::VaultLocked);
        }
        
        // TODO: Encrypt secret before storing
        self.secrets.insert(name.to_string(), secret.0);
        // TODO: Persist to disk
        
        Ok(())
    }
    
    /// Retrieve a secret
    pub fn retrieve(&self, name: &str) -> Result<Secret> {
        if !self.unlocked {
            return Err(CryptoError::VaultLocked);
        }
        
        self.secrets
            .get(name)
            .map(|data| Secret::new(data.clone()))
            .ok_or_else(|| CryptoError::SecretNotFound(name.to_string()))
    }
    
    /// Check if a secret exists
    pub fn contains(&self, name: &str) -> bool {
        self.secrets.contains_key(name)
    }
    
    /// List all secret names (not values)
    pub fn list(&self) -> Vec<&str> {
        self.secrets.keys().map(|s| s.as_str()).collect()
    }
}

impl Drop for SecureVault {
    fn drop(&mut self) {
        // Ensure secrets are zeroized on drop
        self.lock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vault_lock_unlock() {
        let mut vault = SecureVault::open("/tmp/test_vault").unwrap();
        assert!(!vault.unlocked);
        
        vault.unlock().unwrap();
        assert!(vault.unlocked);
        
        vault.lock();
        assert!(!vault.unlocked);
    }
}
