//! Conversation context management

use argus_memory::Memory;
use chrono::{DateTime, Utc};

/// A message in the conversation
#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    System,
}

/// The conversation context including memories
pub struct ConversationContext {
    messages: Vec<Message>,
    memories: Vec<Memory>,
    system_prompt: Option<String>,
}

impl ConversationContext {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            memories: vec![],
            system_prompt: None,
        }
    }
    
    /// Set the system prompt
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = Some(prompt.into());
    }
    
    /// Add a memory to the context
    pub fn add_memory(&mut self, memory: Memory) {
        // Avoid duplicates
        if !self.memories.iter().any(|m| m.id == memory.id) {
            self.memories.push(memory);
        }
    }
    
    /// Add a user message
    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(Message {
            role: Role::User,
            content: content.to_string(),
            timestamp: Utc::now(),
        });
    }
    
    /// Add an assistant message
    pub fn add_assistant_message(&mut self, content: &str) {
        self.messages.push(Message {
            role: Role::Assistant,
            content: content.to_string(),
            timestamp: Utc::now(),
        });
    }
    
    /// Get memory count
    pub fn memory_count(&self) -> usize {
        self.memories.len()
    }
    
    /// Build the full context for the LLM
    pub fn build_prompt(&self) -> String {
        let mut parts = vec![];
        
        // System prompt
        if let Some(ref system) = self.system_prompt {
            parts.push(format!("[SYSTEM]\n{}\n", system));
        }
        
        // Relevant memories
        if !self.memories.is_empty() {
            parts.push("[MEMORIES]\n".to_string());
            for memory in &self.memories {
                parts.push(format!("- {}\n", memory.content));
            }
            parts.push("\n".to_string());
        }
        
        // Conversation history
        parts.push("[CONVERSATION]\n".to_string());
        for msg in &self.messages {
            let role = match msg.role {
                Role::User => "User",
                Role::Assistant => "Assistant",
                Role::System => "System",
            };
            parts.push(format!("{}: {}\n", role, msg.content));
        }
        
        parts.join("")
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self::new()
    }
}
