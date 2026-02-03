//! Argus Crypto - Post-quantum cryptography layer
//! 
//! This crate provides:
//! - ML-KEM (Kyber) key encapsulation
//! - ML-DSA (Dilithium) digital signatures
//! - SLH-DSA (SPHINCS+) stateless signatures
//! - Hardware keychain integration
//! - Encrypted vault for secrets

pub mod vault;
pub mod keychain;
pub mod pq;

pub use vault::SecureVault;
pub use keychain::KeychainProvider;
