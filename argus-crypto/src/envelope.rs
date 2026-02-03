//! Encrypted envelope format for secure data transport

use serde::{Deserialize, Serialize};

/// An encrypted envelope containing ciphertext and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    /// Version of the envelope format
    pub version: u8,
    /// ML-KEM encapsulated key
    pub encapsulated_key: Vec<u8>,
    /// ChaCha20-Poly1305 nonce
    pub nonce: [u8; 12],
    /// Encrypted payload
    pub ciphertext: Vec<u8>,
    /// Poly1305 authentication tag
    pub tag: [u8; 16],
    /// Optional ML-DSA signature over the envelope
    pub signature: Option<Vec<u8>>,
}

impl EncryptedEnvelope {
    /// Create a new envelope (encryption happens in keys.rs)
    pub fn new(
        encapsulated_key: Vec<u8>,
        nonce: [u8; 12],
        ciphertext: Vec<u8>,
        tag: [u8; 16],
    ) -> Self {
        Self {
            version: 1,
            encapsulated_key,
            nonce,
            ciphertext,
            tag,
            signature: None,
        }
    }
    
    /// Add a signature to the envelope
    pub fn with_signature(mut self, signature: Vec<u8>) -> Self {
        self.signature = Some(signature);
        self
    }
    
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Use a proper binary format (maybe bincode)
        serde_json::to_vec(self).unwrap_or_default()
    }
    
    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        serde_json::from_slice(data).ok()
    }
}
