//! SentryBus — shared state between Sentry and the Daemon.
//!
//! Phase 1: Sentry watches, Daemon reads threat posture. Listen mode.
//! Phase 3: Daemon submits plans for Sentry red-team review before execution.
//!          Sentry gates with Approved / Flagged. Human override via #sentry "APPROVED".

use std::collections::VecDeque;
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::{DateTime, Utc};

// ── Gate types ─────────────────────────────────────────────────────────────

/// What the Daemon is asking Sentry to review.
#[derive(Debug, Clone)]
pub struct ReviewRequest {
    pub id: String,
    pub plan: String,
    pub submitted_at: DateTime<Utc>,
    pub requester_model: String,
}

/// Sentry's verdict on a plan.
#[derive(Debug, Clone)]
pub enum ReviewVerdict {
    Approved,
    Flagged { concerns: String, severity: ThreatSeverity },
    /// Human typed "APPROVED" in #sentry — override regardless of Sentry's findings.
    HumanOverride,
}

/// Sentry's full review result.
#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub request_id: String,
    pub verdict: ReviewVerdict,
    pub report: String,
    pub reviewed_at: DateTime<Utc>,
}

/// How serious Sentry thinks something is.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ThreatSeverity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Low      => "LOW",
            Self::Medium   => "MEDIUM",
            Self::High     => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "CRITICAL" => Self::Critical,
            "HIGH"     => Self::High,
            "MEDIUM"   => Self::Medium,
            _          => Self::Low,
        }
    }
}

/// A threat assessment written by Sentry.
#[derive(Debug, Clone)]
pub struct SentryThreat {
    pub severity: ThreatSeverity,
    pub summary: String,
    pub full_report: String,
    pub raised_at: DateTime<Utc>,
    /// True once the Daemon has acknowledged it in a turn context.
    pub acknowledged: bool,
}

/// Shared communication channel between Sentry and the Daemon.
/// Clone is cheap — it's all Arc-wrapped.
#[derive(Clone)]
pub struct SentryBus {
    /// Active threats raised by Sentry. Daemon injects these into turn context.
    /// Capped at 10 — oldest are dropped when full.
    threats: Arc<RwLock<VecDeque<SentryThreat>>>,
    /// Flipped to true when Sentry raises a HIGH or CRITICAL threat.
    pub alert_flag: Arc<AtomicBool>,
    /// Current threat posture formatted for prompt injection.
    posture_cache: Arc<RwLock<Option<String>>>,
    /// Phase 3 gate — pending plan awaiting Sentry review.
    /// Only one plan at a time; Daemon queues sequentially.
    pub pending_review: Arc<Mutex<Option<ReviewRequest>>>,
    /// Sentry writes her verdict here after reviewing the pending plan.
    pub review_result: Arc<Mutex<Option<ReviewResult>>>,
    /// Flipped to true when Sentry has finished a review. Daemon wakes on this.
    pub review_ready: Arc<AtomicBool>,
}

impl SentryBus {
    pub fn new() -> Self {
        Self {
            threats: Arc::new(RwLock::new(VecDeque::new())),
            alert_flag: Arc::new(AtomicBool::new(false)),
            posture_cache: Arc::new(RwLock::new(None)),
            pending_review: Arc::new(Mutex::new(None)),
            review_result: Arc::new(Mutex::new(None)),
            review_ready: Arc::new(AtomicBool::new(false)),
        }
    }

    // ── Phase 3: Gate ──────────────────────────────────────────────────────

    /// Daemon submits a plan for Sentry to red-team before execution.
    /// Returns a request ID. Call `wait_for_review` to block until verdict.
    pub fn submit_for_review(&self, plan: impl Into<String>, requester_model: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let request = ReviewRequest {
            id: id.clone(),
            plan: plan.into(),
            submitted_at: Utc::now(),
            requester_model: requester_model.to_string(),
        };
        if let Ok(mut pending) = self.pending_review.lock() {
            *pending = Some(request);
        }
        self.review_ready.store(false, Ordering::Relaxed);
        id
    }

    /// Sentry calls this after completing a review.
    pub fn complete_review(&self, request_id: &str, verdict: ReviewVerdict, report: impl Into<String>) {
        let result = ReviewResult {
            request_id: request_id.to_string(),
            verdict,
            report: report.into(),
            reviewed_at: Utc::now(),
        };
        if let Ok(mut rv) = self.review_result.lock() {
            *rv = Some(result);
        }
        if let Ok(mut pending) = self.pending_review.lock() {
            *pending = None;
        }
        self.review_ready.store(true, Ordering::Relaxed);
    }

    /// Human override — called when Bradlee types "APPROVED" in #sentry.
    pub fn human_override(&self, request_id: &str) {
        self.complete_review(
            request_id,
            ReviewVerdict::HumanOverride,
            "Human override — proceed regardless of threat assessment.",
        );
    }

    /// Check if a review is pending (for Sentry to poll).
    pub fn has_pending_review(&self) -> Option<ReviewRequest> {
        self.pending_review.lock().ok()?.clone()
    }

    /// Daemon calls this to get the verdict once review_ready is true.
    pub fn take_review_result(&self) -> Option<ReviewResult> {
        self.review_ready.store(false, Ordering::Relaxed);
        self.review_result.lock().ok()?.take()
    }

    /// Sentry calls this when she finds something.
    pub fn raise(&self, severity: ThreatSeverity, summary: impl Into<String>, full_report: impl Into<String>) {
        let threat = SentryThreat {
            severity: severity.clone(),
            summary: summary.into(),
            full_report: full_report.into(),
            raised_at: Utc::now(),
            acknowledged: false,
        };

        if let Ok(mut threats) = self.threats.write() {
            threats.push_front(threat);
            // Keep the list bounded — oldest drop off
            while threats.len() > 10 {
                threats.pop_back();
            }
        }

        if severity >= ThreatSeverity::High {
            self.alert_flag.store(true, Ordering::Relaxed);
        }

        // Rebuild the posture cache
        self.rebuild_cache();
    }

    /// Sentry calls this when a watch cycle comes back clean.
    pub fn report_clean(&self) {
        // Don't touch existing threats — just update the cache with a clean signal
        // and reset the alert flag if no HIGH+ threats remain
        if let Ok(threats) = self.threats.read() {
            let has_high = threats.iter().any(|t| t.severity >= ThreatSeverity::High);
            if !has_high {
                self.alert_flag.store(false, Ordering::Relaxed);
            }
        }
    }

    /// Daemon calls this to mark threats as seen so they don't keep shouting.
    pub fn acknowledge_all(&self) {
        if let Ok(mut threats) = self.threats.write() {
            for t in threats.iter_mut() {
                t.acknowledged = true;
            }
        }
        self.alert_flag.store(false, Ordering::Relaxed);
        self.rebuild_cache();
    }

    /// Returns the current threat posture as a system prompt block.
    /// Returns None if everything is clean (no active threats).
    pub fn posture_for_prompt(&self) -> Option<String> {
        if let Ok(cache) = self.posture_cache.read() {
            cache.clone()
        } else {
            None
        }
    }

    fn rebuild_cache(&self) {
        let posture = if let Ok(threats) = self.threats.read() {
            if threats.is_empty() {
                None
            } else {
                let mut lines = Vec::new();
                for (i, t) in threats.iter().enumerate() {
                    let ack = if t.acknowledged { " (seen)" } else { "" };
                    let age_mins = (Utc::now() - t.raised_at).num_minutes();
                    lines.push(format!(
                        "{}. [{}]{} {}m ago — {}",
                        i + 1,
                        t.severity.label(),
                        ack,
                        age_mins,
                        t.summary
                    ));
                }
                Some(format!(
                    "[SENTRY THREAT POSTURE]\n{}\nSee #sentry for full attack chains.\n[END SENTRY]",
                    lines.join("\n")
                ))
            }
        } else {
            None
        };

        if let Ok(mut cache) = self.posture_cache.write() {
            *cache = posture;
        }
    }

    /// How many unacknowledged threats are active.
    pub fn active_count(&self) -> usize {
        self.threats.read()
            .map(|t| t.iter().filter(|th| !th.acknowledged).count())
            .unwrap_or(0)
    }
}

impl Default for SentryBus {
    fn default() -> Self {
        Self::new()
    }
}
