//! Hardware keychain integration
//! 
//! Supports:
//! - macOS Keychain
//! - Windows Credential Manager  
//! - Linux Secret Service (via D-Bus)

use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeychainError {
    #[error("Keychain not available")]
    NotAvailable,
    
    #[error("Access denied")]
    AccessDenied,
    
    #[error("Item not found")]
    NotFound,
    
    #[error("Platform error: {0}")]
    Platform(String),
}

/// Provider for hardware keychain operations
pub struct KeychainProvider {
    service_name: String,
}

impl KeychainProvider {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }
    
    /// Store the master key in the hardware keychain
    pub fn store_master_key(&self, key: &[u8]) -> Result<(), KeychainError> {
        let entry = keyring::Entry::new(&self.service_name, "master_key")
            .map_err(|e| KeychainError::Platform(e.to_string()))?;
        
        // Store as hex since keyring expects strings
        let encoded = hex_encode(key);
        entry.set_password(&encoded)
            .map_err(|e| KeychainError::Platform(e.to_string()))?;
        
        Ok(())
    }
    
    /// Retrieve the master key from the hardware keychain
    pub fn retrieve_master_key(&self) -> Result<Vec<u8>, KeychainError> {
        let entry = keyring::Entry::new(&self.service_name, "master_key")
            .map_err(|e| KeychainError::Platform(e.to_string()))?;
        
        let encoded = entry.get_password()
            .map_err(|e| match e {
                keyring::Error::NoEntry => KeychainError::NotFound,
                _ => KeychainError::Platform(e.to_string()),
            })?;
        
        hex_decode(&encoded)
            .map_err(|e| KeychainError::Platform(e))
    }
    
    /// Delete the master key from keychain
    pub fn delete_master_key(&self) -> Result<(), KeychainError> {
        let entry = keyring::Entry::new(&self.service_name, "master_key")
            .map_err(|e| KeychainError::Platform(e.to_string()))?;
        
        entry.delete_password()
            .map_err(|e| KeychainError::Platform(e.to_string()))?;
        
        Ok(())
    }
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("Invalid hex string length".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i+2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
