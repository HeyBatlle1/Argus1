//! Context management - loading memories into the conversation

/// Manages what context gets loaded for each conversation
pub struct ContextManager {
    // TODO: Implement context management
}

impl ContextManager {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Build context for a new conversation
    pub async fn build_context(&self, _user_message: &str) -> anyhow::Result<String> {
        todo!("Build context from memories")
    }
}
