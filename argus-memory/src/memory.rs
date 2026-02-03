//! Memory types and structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of memory
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    /// Factual information
    Fact,
    /// Procedural knowledge
    Procedure,
    /// User preferences
    Preference,
    /// Conversation context
    Context,
    /// Relationship insights
    Relationship,
    /// Session reflection
    Reflection,
}

/// A single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique identifier
    pub id: Uuid,
    /// Type of memory
    pub memory_type: MemoryType,
    /// The actual content
    pub content: String,
    /// Why this memory was created
    pub reasoning: Option<String>,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// When this memory was created
    pub created_at: DateTime<Utc>,
    /// When this memory was last accessed
    pub last_accessed: DateTime<Utc>,
    /// Number of times recalled
    pub recall_count: u32,
    /// Vector embedding for semantic search (encrypted at rest)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

impl Memory {
    /// Create a new memory
    pub fn new(memory_type: MemoryType, content: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            memory_type,
            content: content.into(),
            reasoning: None,
            importance: 0.5,
            tags: vec![],
            created_at: now,
            last_accessed: now,
            recall_count: 0,
            embedding: None,
        }
    }
    
    /// Set importance
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }
    
    /// Set reasoning
    pub fn with_reasoning(mut self, reasoning: impl Into<String>) -> Self {
        self.reasoning = Some(reasoning.into());
        self
    }
    
    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
    
    /// Record that this memory was recalled
    pub fn mark_recalled(&mut self) {
        self.last_accessed = Utc::now();
        self.recall_count += 1;
    }
}
