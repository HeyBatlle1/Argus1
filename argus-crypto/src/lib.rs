//! Argus Crypto - Post-Quantum Cryptography Layer
//!
//! This crate provides quantum-resistant encryption and signing
//! for the Argus agent runtime. No plaintext secrets. Ever.

mod vault;
mod keys;
mod envelope;

pub use vault::SecureVault;
pub use keys::{KeyPair, PublicKey, SecretKey};
pub use envelope::EncryptedEnvelope;

/// Errors that can occur in cryptographic operations
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Key generation failed: {0}")]
    KeyGeneration(String),
    
    #[error("Encryption failed: {0}")]
    Encryption(String),
    
    #[error("Decryption failed: {0}")]
    Decryption(String),
    
    #[error("Signature verification failed")]
    SignatureInvalid,
    
    #[error("Keychain access denied: {0}")]
    KeychainDenied(String),
    
    #[error("Vault is locked")]
    VaultLocked,
    
    #[error("Secret not found: {0}")]
    SecretNotFound(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;
