//! Mission types — the typed contract between intent and verified reality.
//!
//! A Mission is not a conversation. It is a commitment:
//! - A stated objective
//! - Typed deliverables with compiled verification
//! - An audit chain that proves every step
//! - A Sentry gate before anything runs
//! - A git commit when it closes
//!
//! This is how Argus beats Manus: not more tools, a different trust model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Deliverables ───────────────────────────────────────────────────────────
//
// Not "the agent said it's done." Compiled checks.

/// A typed deliverable with a verification function.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Deliverable {
    /// A file must exist at this path in /workspace.
    File {
        path: String,
        description: String,
    },
    /// A shell command must exit 0 (tests pass, lint clean, etc.)
    Command {
        command: String,
        description: String,
        expected_exit: i32,
    },
    /// An HTTP endpoint must respond with an expected status.
    HttpEndpoint {
        url: String,
        expected_status: u16,
        description: String,
    },
    /// A git commit must exist with the given hash prefix in /workspace.
    GitCommit {
        hash_prefix: Option<String>,
        description: String,
    },
    /// A skill must exist in the library matching this name.
    Skill {
        skill_name: String,
        description: String,
    },
}

impl Deliverable {
    pub fn description(&self) -> &str {
        match self {
            Self::File        { description, .. } => description,
            Self::Command     { description, .. } => description,
            Self::HttpEndpoint{ description, .. } => description,
            Self::GitCommit   { description, .. } => description,
            Self::Skill       { description, .. } => description,
        }
    }
}

/// Result of verifying a single deliverable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliverableResult {
    pub deliverable: Deliverable,
    pub passed: bool,
    pub output: String,
    pub verified_at: DateTime<Utc>,
}

// ── Subtasks ───────────────────────────────────────────────────────────────

/// A unit of work within a mission. Assigned to a specific model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub id: Uuid,
    pub description: String,
    /// The model assigned to execute this subtask.
    /// Defaults to Grok Build for coding/execution tasks.
    pub assigned_model: String,
    pub status: SubtaskStatus,
    pub output: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubtaskStatus {
    Pending,
    Running,
    Complete,
    Failed { reason: String },
    Skipped { reason: String },
}

impl Subtask {
    pub fn new(description: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            description: description.into(),
            assigned_model: model.into(),
            status: SubtaskStatus::Pending,
            output: None,
            created_at: Utc::now(),
            completed_at: None,
        }
    }
}

// ── Mission ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MissionStatus {
    /// Plan is being decomposed into subtasks.
    Planning,
    /// Waiting for Sentry to review the plan before execution.
    SentryReview,
    /// Sentry flagged something — waiting for resolution or human override.
    SentryHold { concerns: String },
    /// Subtasks are executing.
    Executing,
    /// Subtasks complete — verifying typed deliverables.
    Verifying,
    /// All deliverables verified. Committed to git.
    Complete { commit_hash: String },
    /// Mission failed — reason documented.
    Failed { reason: String },
}

/// The top-level mission — a stated objective backed by verified deliverables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: Uuid,
    /// One clear sentence. What does done look like?
    pub objective: String,
    /// The model that created this mission (any model can create).
    pub created_by: String,
    /// The model primarily executing this mission (default: Grok Build).
    pub primary_executor: String,
    /// Typed deliverables — what must be verifiably true when this closes.
    pub deliverables: Vec<Deliverable>,
    /// Decomposed work units.
    pub subtasks: Vec<Subtask>,
    /// Deliverable verification results — set when status moves to Verifying.
    pub verification: Vec<DeliverableResult>,
    pub status: MissionStatus,
    /// Sentry review request ID — set when plan is submitted to gate.
    pub sentry_request_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Mission {
    pub fn new(
        objective: impl Into<String>,
        created_by: impl Into<String>,
        executor: impl Into<String>,
        deliverables: Vec<Deliverable>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            objective: objective.into(),
            created_by: created_by.into(),
            primary_executor: executor.into(),
            deliverables,
            subtasks: Vec::new(),
            verification: Vec::new(),
            status: MissionStatus::Planning,
            sentry_request_id: None,
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self.status, MissionStatus::Complete { .. } | MissionStatus::Failed { .. })
    }

    pub fn all_subtasks_done(&self) -> bool {
        self.subtasks.iter().all(|s| {
            matches!(s.status, SubtaskStatus::Complete | SubtaskStatus::Skipped { .. })
        })
    }

    pub fn any_subtask_failed(&self) -> bool {
        self.subtasks.iter().any(|s| matches!(s.status, SubtaskStatus::Failed { .. }))
    }

    /// Format a plan summary for Sentry to red-team.
    pub fn plan_summary(&self) -> String {
        let subtask_list = self.subtasks.iter().enumerate()
            .map(|(i, s)| format!("  {}. [{}] {}", i + 1, s.assigned_model, s.description))
            .collect::<Vec<_>>()
            .join("\n");

        let deliverable_list = self.deliverables.iter().enumerate()
            .map(|(i, d)| format!("  {}. {}", i + 1, d.description()))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "OBJECTIVE: {}\nCREATED BY: {}\nPRIMARY EXECUTOR: {}\n\n\
             SUBTASKS:\n{}\n\nDELIVERABLES:\n{}",
            self.objective, self.created_by, self.primary_executor,
            if subtask_list.is_empty() { "  (not yet decomposed)".to_string() } else { subtask_list },
            if deliverable_list.is_empty() { "  (none specified)".to_string() } else { deliverable_list }
        )
    }
}
