//! Capability-based security for sandbox permissions

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// A specific capability that can be granted to a sandboxed tool
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Read a specific file
    ReadFile(PathBuf),
    /// Write to a specific file
    WriteFile(PathBuf),
    /// Read from a directory
    ReadDirectory(PathBuf),
    /// Make HTTP requests to specific domains
    HttpRequest(String),
    /// Access environment variable (by name)
    EnvVar(String),
    /// Use specific amount of memory (bytes)
    Memory(usize),
    /// Execute for specific duration (ms)
    ExecutionTime(u64),
}

/// A set of capabilities granted to a tool
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    capabilities: HashSet<Capability>,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Grant a capability
    pub fn grant(&mut self, cap: Capability) -> &mut Self {
        self.capabilities.insert(cap);
        self
    }
    
    /// Check if a capability is granted
    pub fn has(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }
    
    /// Check if HTTP access to a domain is allowed
    pub fn can_access_domain(&self, domain: &str) -> bool {
        self.capabilities.iter().any(|cap| {
            matches!(cap, Capability::HttpRequest(d) if d == domain || d == "*")
        })
    }
    
    /// Check if file read is allowed
    pub fn can_read_file(&self, path: &PathBuf) -> bool {
        self.capabilities.iter().any(|cap| {
            match cap {
                Capability::ReadFile(p) => p == path,
                Capability::ReadDirectory(dir) => path.starts_with(dir),
                _ => false,
            }
        })
    }
}

/// Builder for creating capability sets with a fluent API
impl CapabilitySet {
    pub fn with_file_read(mut self, path: impl Into<PathBuf>) -> Self {
        self.grant(Capability::ReadFile(path.into()));
        self
    }
    
    pub fn with_file_write(mut self, path: impl Into<PathBuf>) -> Self {
        self.grant(Capability::WriteFile(path.into()));
        self
    }
    
    pub fn with_http(mut self, domain: impl Into<String>) -> Self {
        self.grant(Capability::HttpRequest(domain.into()));
        self
    }
    
    pub fn with_memory(mut self, bytes: usize) -> Self {
        self.grant(Capability::Memory(bytes));
        self
    }
    
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.grant(Capability::ExecutionTime(ms));
        self
    }
}
