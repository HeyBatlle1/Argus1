//! Capability-based permission system
//!
//! Tools request capabilities, we grant only what's needed.
//! This is how you prevent a "weather skill" from reading SSH keys.

use serde::{Deserialize, Serialize};

/// Capabilities a tool can request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Read from filesystem (with path restrictions)
    FileRead { paths: Vec<String> },
    
    /// Write to filesystem (with path restrictions)
    FileWrite { paths: Vec<String> },
    
    /// Network access (with domain restrictions)
    Network { domains: Vec<String> },
    
    /// Environment variable access (specific vars only)
    Environment { vars: Vec<String> },
    
    /// Execute subprocess (highly restricted)
    Subprocess { allowed_commands: Vec<String> },
}

/// A set of capabilities granted to a tool
#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    capabilities: Vec<Capability>,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn grant(&mut self, cap: Capability) {
        if !self.capabilities.contains(&cap) {
            self.capabilities.push(cap);
        }
    }
    
    pub fn has(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }
    
    /// Check if a file read is allowed
    pub fn can_read_file(&self, path: &str) -> bool {
        self.capabilities.iter().any(|c| match c {
            Capability::FileRead { paths } => {
                paths.iter().any(|p| path.starts_with(p))
            }
            _ => false,
        })
    }
    
    /// Check if a network request is allowed
    pub fn can_access_domain(&self, domain: &str) -> bool {
        self.capabilities.iter().any(|c| match c {
            Capability::Network { domains } => {
                domains.iter().any(|d| domain.ends_with(d))
            }
            _ => false,
        })
    }
}
