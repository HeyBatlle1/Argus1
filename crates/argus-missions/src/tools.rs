//! Mission tools — the interface every model uses to create, track, and interact with missions.
//!
//! Any model can start a mission. Grok Build executes by default.
//! Sentry gates every plan before it runs.

use crate::types::*;
use crate::runner::run_mission;
use argus_core::{AgentConfig, SentryBus, MODEL_GROK_BUILD};
use argus_core::supabase::SupabaseClient;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use reqwest::Client;

/// Global in-process mission registry.
/// In production this would be backed by Supabase. For now it's in-memory.
pub struct MissionRegistry {
    missions: Mutex<HashMap<String, Mission>>,
}

impl MissionRegistry {
    pub fn new() -> Self {
        Self { missions: Mutex::new(HashMap::new()) }
    }

    pub fn insert(&self, mission: Mission) {
        if let Ok(mut m) = self.missions.lock() {
            m.insert(mission.id.to_string(), mission);
        }
    }

    pub fn get(&self, id: &str) -> Option<Mission> {
        self.missions.lock().ok()?.get(id).cloned()
    }

    pub fn update(&self, mission: Mission) {
        if let Ok(mut m) = self.missions.lock() {
            m.insert(mission.id.to_string(), mission);
        }
    }

    pub fn list_active(&self) -> Vec<Mission> {
        self.missions.lock()
            .map(|m| m.values().filter(|m| !m.is_terminal()).cloned().collect())
            .unwrap_or_default()
    }
}

// ── Tool schemas ───────────────────────────────────────────────────────────

pub fn mission_tool_schemas() -> Vec<Value> {
    serde_json::json!([
        {
            "type": "function",
            "function": {
                "name": "start_mission",
                "description": "Start a mission — a structured task with typed deliverables and verified completion. Grok Build executes by default. Sentry red-teams the plan before anything runs. Use this for any multi-step work that needs to be provably done, not just talked about.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "objective":    { "type": "string", "description": "One clear sentence: what does done look like?" },
                        "deliverables": {
                            "type": "array",
                            "description": "What must be verifiably true when this mission closes",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "type":        { "type": "string", "enum": ["file", "command", "http_endpoint", "git_commit", "skill"] },
                                    "description": { "type": "string" },
                                    "path":        { "type": "string", "description": "For file deliverables" },
                                    "command":     { "type": "string", "description": "For command deliverables" },
                                    "url":         { "type": "string", "description": "For http_endpoint deliverables" },
                                    "skill_name":  { "type": "string", "description": "For skill deliverables" }
                                },
                                "required": ["type", "description"]
                            }
                        },
                        "executor": { "type": "string", "description": "Primary execution model (default: grok-build)" }
                    },
                    "required": ["objective"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "mission_status",
                "description": "Check the status of a running mission — subtask progress, verification results, current phase.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "mission_id": { "type": "string", "description": "Mission ID from start_mission" }
                    },
                    "required": ["mission_id"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "list_missions",
                "description": "List all active (non-terminal) missions.",
                "parameters": { "type": "object", "properties": {} }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "add_subtask",
                "description": "Add a subtask to an existing mission in Planning state.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "mission_id":   { "type": "string" },
                        "description":  { "type": "string", "description": "What to do — one concrete sentence" },
                        "model":        { "type": "string", "description": "Which model to assign (default: grok-build)" }
                    },
                    "required": ["mission_id", "description"]
                }
            }
        }
    ]).as_array().unwrap().clone()
}

// ── Tool execution ─────────────────────────────────────────────────────────

pub async fn execute_mission_tool(
    name: &str,
    args: &Value,
    config: &AgentConfig,
    registry: &Arc<MissionRegistry>,
    sentry_bus: Option<&SentryBus>,
    supabase: Option<&SupabaseClient>,
    http: &Client,
    current_model: &str,
    discord_token: Option<&str>,
    ops_channel_id: Option<&str>,
) -> Option<String> {
    match name {
        "start_mission" => Some(tool_start_mission(
            args, config, registry, sentry_bus, supabase, http,
            current_model, discord_token, ops_channel_id
        ).await),
        "mission_status" => Some(tool_mission_status(args, registry)),
        "list_missions"  => Some(tool_list_missions(registry)),
        "add_subtask"    => Some(tool_add_subtask(args, registry)),
        _ => None,
    }
}

async fn tool_start_mission(
    args: &Value,
    config: &AgentConfig,
    registry: &Arc<MissionRegistry>,
    sentry_bus: Option<&SentryBus>,
    supabase: Option<&SupabaseClient>,
    http: &Client,
    current_model: &str,
    discord_token: Option<&str>,
    ops_channel_id: Option<&str>,
) -> String {
    let objective = match args["objective"].as_str() {
        Some(o) if !o.is_empty() => o.to_string(),
        _ => return "start_mission requires an objective.".to_string(),
    };

    let executor = args["executor"].as_str()
        .unwrap_or(MODEL_GROK_BUILD)
        .to_string();

    // Parse deliverables
    let mut deliverables: Vec<Deliverable> = Vec::new();
    if let Some(arr) = args["deliverables"].as_array() {
        for d in arr {
            let desc = d["description"].as_str().unwrap_or("").to_string();
            let del = match d["type"].as_str() {
                Some("file") => Deliverable::File {
                    path: d["path"].as_str().unwrap_or("").to_string(),
                    description: desc,
                },
                Some("command") => Deliverable::Command {
                    command: d["command"].as_str().unwrap_or("").to_string(),
                    description: desc,
                    expected_exit: d["expected_exit"].as_i64().unwrap_or(0) as i32,
                },
                Some("http_endpoint") => Deliverable::HttpEndpoint {
                    url: d["url"].as_str().unwrap_or("").to_string(),
                    expected_status: d["expected_status"].as_u64().unwrap_or(200) as u16,
                    description: desc,
                },
                Some("git_commit") => Deliverable::GitCommit {
                    hash_prefix: d["hash_prefix"].as_str().map(|s| s.to_string()),
                    description: desc,
                },
                Some("skill") => Deliverable::Skill {
                    skill_name: d["skill_name"].as_str().unwrap_or("").to_string(),
                    description: desc,
                },
                _ => continue,
            };
            deliverables.push(del);
        }
    }

    // Default deliverable: git commit
    if deliverables.is_empty() {
        deliverables.push(Deliverable::GitCommit {
            hash_prefix: None,
            description: "Work committed to /workspace git".to_string(),
        });
    }

    let mission = Mission::new(&objective, current_model, &executor, deliverables);
    let mission_id = mission.id.to_string();
    let id_short = &mission_id[..8];

    eprintln!("[mission:{}] Created — {}", id_short, objective);
    registry.insert(mission.clone());

    // Run mission in background — don't block the agent turn
    let registry2  = registry.clone();
    let config2    = config.clone();
    let http2      = http.clone();
    let sentry2    = sentry_bus.map(|b| b.clone());
    let supabase2  = supabase.cloned();
    let token2     = discord_token.map(|s| s.to_string());
    let channel2   = ops_channel_id.map(|s| s.to_string());

    let mission_id2 = mission_id.clone();
    tokio::spawn(async move {
        let completed = run_mission(
            registry2.get(&mission_id2).unwrap_or(mission),
            &config2,
            sentry2.as_ref(),
            supabase2.as_ref(),
            &http2,
            token2.as_deref(),
            channel2.as_deref(),
        ).await;
        registry2.update(completed);
    });

    format!(
        "Mission `{}` started — {}\n\
         Executor: {} | Sentry gate: {}\n\
         Use mission_status(\"{}\") to check progress.\n\
         Posting updates to #ops.",
        id_short, objective,
        executor,
        if sentry_bus.is_some() { "active" } else { "off" },
        id_short
    )
}

fn tool_mission_status(args: &Value, registry: &Arc<MissionRegistry>) -> String {
    let id = args["mission_id"].as_str().unwrap_or("");
    let mission = registry.get(id)
        .or_else(|| {
            // Try short ID prefix match
            registry.list_active().into_iter()
                .find(|m| m.id.to_string().starts_with(id))
        });

    match mission {
        None => format!("Mission `{}` not found.", id),
        Some(m) => {
            let subtask_lines = m.subtasks.iter().enumerate()
                .map(|(i, s)| format!("  {}. [{}] {} — {:?}",
                    i + 1, s.assigned_model, s.description, s.status))
                .collect::<Vec<_>>()
                .join("\n");

            let verify_lines = m.verification.iter()
                .map(|r| format!("  {} {}", if r.passed { "✅" } else { "❌" }, r.deliverable.description()))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                "**Mission `{}`**\n{}\nStatus: {:?}\n\nSubtasks:\n{}\n\nVerification:\n{}",
                &m.id.to_string()[..8],
                m.objective,
                m.status,
                if subtask_lines.is_empty() { "  (none yet)".to_string() } else { subtask_lines },
                if verify_lines.is_empty() { "  (not yet verified)".to_string() } else { verify_lines }
            )
        }
    }
}

fn tool_list_missions(registry: &Arc<MissionRegistry>) -> String {
    let active = registry.list_active();
    if active.is_empty() {
        return "No active missions.".to_string();
    }
    active.iter()
        .map(|m| format!("- `{}` {:?} — {}", &m.id.to_string()[..8], m.status, m.objective))
        .collect::<Vec<_>>()
        .join("\n")
}

fn tool_add_subtask(args: &Value, registry: &Arc<MissionRegistry>) -> String {
    let id   = args["mission_id"].as_str().unwrap_or("");
    let desc = args["description"].as_str().unwrap_or("").trim();
    let model = args["model"].as_str().unwrap_or(MODEL_GROK_BUILD);

    if desc.is_empty() {
        return "add_subtask requires a description.".to_string();
    }

    let mut mission = match registry.get(id) {
        Some(m) => m,
        None    => return format!("Mission `{}` not found.", id),
    };

    if mission.status != MissionStatus::Planning {
        return format!("Mission `{}` is past planning phase — can't add subtasks.", id);
    }

    mission.subtasks.push(Subtask::new(desc, model));
    let count = mission.subtasks.len();
    registry.update(mission);

    format!("Subtask added to mission `{}` — now has {} subtask(s).", id, count)
}
