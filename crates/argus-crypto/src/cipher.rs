//! ChaCha20-Poly1305 authenticated encryption
//!
//! Why ChaCha20-Poly1305?
//! - Constant-time (no timing attacks)
//! - No weak keys
//! - Faster than AES on systems without AES-NI
//! - Used by TLS 1.3, WireGuard, Signal

use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use thiserror::Error;
use zeroize::Zeroizing;

/// 256-bit key (32 bytes)
pub const KEY_SIZE: usize = 32;
/// 96-bit nonce (12 bytes)  
pub const NONCE_SIZE: usize = 12;

#[derive(Error, Debug)]
pub enum CipherError {
    #[error("Encryption failed")]
    EncryptionFailed,
    
    #[error("Decryption failed - data may be corrupted or tampered")]
    DecryptionFailed,
    
    #[error("Invalid key size: expected {KEY_SIZE}, got {0}")]
    InvalidKeySize(usize),
    
    #[error("Invalid nonce size: expected {NONCE_SIZE}, got {0}")]
    InvalidNonceSize(usize),
}

/// Generate a cryptographically secure random key
pub fn generate_key() -> Zeroizing<[u8; KEY_SIZE]> {
    let mut key = Zeroizing::new([0u8; KEY_SIZE]);
    OsRng.fill_bytes(key.as_mut());
    key
}

/// Generate a cryptographically secure random nonce
pub fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Encrypt plaintext with ChaCha20-Poly1305
/// 
/// Returns: nonce || ciphertext || tag
/// The nonce is prepended so we can decrypt without external state
pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CipherError> {
    if key.len() != KEY_SIZE {
        return Err(CipherError::InvalidKeySize(key.len()));
    }
    
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| CipherError::InvalidKeySize(key.len()))?;
    
    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| CipherError::EncryptionFailed)?;
    
    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

/// Decrypt ciphertext encrypted with encrypt()
/// 
/// Expects: nonce || ciphertext || tag
pub fn decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Zeroizing<Vec<u8>>, CipherError> {
    if key.len() != KEY_SIZE {
        return Err(CipherError::InvalidKeySize(key.len()));
    }
    
    if ciphertext.len() < NONCE_SIZE {
        return Err(CipherError::DecryptionFailed);
    }
    
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| CipherError::InvalidKeySize(key.len()))?;
    
    let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, encrypted)
        .map_err(|_| CipherError::DecryptionFailed)?;
    
    Ok(Zeroizing::new(plaintext))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_key();
        let plaintext = b"ANTHROPIC_API_KEY=sk-ant-super-secret-key";
        
        let ciphertext = encrypt(&key, plaintext).unwrap();
        
        // Ciphertext should be different from plaintext
        assert_ne!(&ciphertext[NONCE_SIZE..], plaintext);
        
        // Should decrypt back to original
        let decrypted = decrypt(&key, &ciphertext).unwrap();
        assert_eq!(decrypted.as_slice(), plaintext);
    }
    
    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = generate_key();
        let plaintext = b"secret data";
        
        let mut ciphertext = encrypt(&key, plaintext).unwrap();
        
        // Tamper with the ciphertext
        if let Some(byte) = ciphertext.last_mut() {
            *byte ^= 0xFF;
        }
        
        // Decryption should fail
        assert!(decrypt(&key, &ciphertext).is_err());
    }
    
    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_key();
        let key2 = generate_key();
        let plaintext = b"secret data";
        
        let ciphertext = encrypt(&key1, plaintext).unwrap();
        
        // Decryption with wrong key should fail
        assert!(decrypt(&key2, &ciphertext).is_err());
    }
}
