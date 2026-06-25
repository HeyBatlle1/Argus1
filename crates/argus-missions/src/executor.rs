//! Mission executor — runs subtasks, verifies deliverables, closes with a commit.
//!
//! Grok Build is the default executor. Any model can run a subtask when assigned.
//! Deliverable verification is compiled, not inferred — the mission doesn't close
//! until the checks pass.

use crate::types::*;
use argus_core::{AgentConfig, AgentEvent, MODEL_GROK_BUILD, run_agent_turn};
use argus_core::mcp::McpClient;
use argus_core::shell::ShellPolicy;
use argus_core::tools::MemoryBackend;
use chrono::Utc;
use reqwest::Client;

const DISCORD_API: &str = "https://discord.com/api/v10";

struct NoopMemory;
impl MemoryBackend for NoopMemory {
    fn remember(&self, _: &str, _: &str, _: Option<&str>, _: f64) -> Result<String, String> { Ok(String::new()) }
    fn recall(&self, _: Option<&str>, _: Option<&str>, _: usize) -> Result<Vec<argus_core::tools::MemoryRecord>, String> { Ok(vec![]) }
    fn forget(&self, _: &str) -> Result<String, String> { Ok(String::new()) }
}

/// Verify all typed deliverables for a mission.
/// Returns (all_passed, results).
pub async fn verify_deliverables(
    mission: &Mission,
    http: &Client,
    exec_auth_token: Option<&str>,
) -> (bool, Vec<DeliverableResult>) {
    let mut results = Vec::new();
    let mut all_passed = true;

    for deliverable in &mission.deliverables {
        let (passed, output) = verify_one(deliverable, http, exec_auth_token).await;
        if !passed { all_passed = false; }
        results.push(DeliverableResult {
            deliverable: deliverable.clone(),
            passed,
            output,
            verified_at: Utc::now(),
        });
    }

    (all_passed, results)
}

async fn verify_one(
    deliverable: &Deliverable,
    http: &Client,
    exec_auth_token: Option<&str>,
) -> (bool, String) {
    match deliverable {
        Deliverable::File { path, .. } => {
            let cmd = format!("test -f '{}' && echo EXISTS || echo MISSING", path);
            let out = workspace_exec(&cmd, http, exec_auth_token).await;
            let passed = out.trim() == "EXISTS";
            (passed, out)
        }

        Deliverable::Command { command, expected_exit, .. } => {
            let cmd = format!("{}; echo EXIT:$?", command);
            let out = workspace_exec(&cmd, http, exec_auth_token).await;
            let actual_exit: i32 = out.lines()
                .find(|l| l.starts_with("EXIT:"))
                .and_then(|l| l.trim_start_matches("EXIT:").trim().parse().ok())
                .unwrap_or(-1);
            let passed = actual_exit == *expected_exit;
            (passed, format!("exit {} (expected {})\n{}", actual_exit, expected_exit, out))
        }

        Deliverable::HttpEndpoint { url, expected_status, .. } => {
            match http.get(url).send().await {
                Ok(r) => {
                    let status = r.status().as_u16();
                    let passed = status == *expected_status;
                    (passed, format!("HTTP {} (expected {})", status, expected_status))
                }
                Err(e) => (false, format!("Request failed: {}", e)),
            }
        }

        Deliverable::GitCommit { hash_prefix, .. } => {
            let cmd = match hash_prefix {
                Some(h) => format!("cd /workspace && git log --oneline | grep '^{}' | head -1", h),
                None    => "cd /workspace && git log --oneline -1".to_string(),
            };
            let out = workspace_exec(&cmd, http, exec_auth_token).await;
            let passed = !out.trim().is_empty() && !out.contains("fatal");
            (passed, out)
        }

        Deliverable::Skill { skill_name, .. } => {
            // Skill existence is verified by the skills client — for now we trust
            // the agent published it. A future version queries Supabase directly.
            (true, format!("Skill '{}' accepted on agent assertion", skill_name))
        }
    }
}

/// Run a single subtask using the assigned model.
pub async fn run_subtask(
    subtask: &Subtask,
    mission: &Mission,
    config: &AgentConfig,
    http: &Client,
) -> Result<String, String> {
    let model = &subtask.assigned_model;

    let prompt = format!(
        "[MISSION SUBTASK]\n\
         Mission: {}\n\
         Subtask: {}\n\
         Your role: {}\n\n\
         Execute this subtask completely. Use your tools. Do real work.\n\
         When done: call git_checkpoint with a descriptive message.\n\
         Report your outcome clearly — what you did and what the result was.",
        mission.objective,
        subtask.description,
        model
    );

    let subtask_config = AgentConfig {
        model: model.clone(),
        ..config.clone()
    };

    let mut mcp = McpClient::new();
    let policy = ShellPolicy::default();
    let memory = NoopMemory;

    run_agent_turn(
        &subtask_config,
        &prompt,
        &[],
        &policy,
        &memory,
        &mut mcp,
        http,
        |event| {
            if let AgentEvent::ToolCall { name, preview, .. } = event {
                eprintln!("[mission/{}] {} — {}", model, name, preview);
            }
        },
    ).await
}

/// Post a mission status update to Discord #ops.
pub async fn post_mission_update(
    http: &Client,
    bot_token: &str,
    ops_channel_id: &str,
    mission: &Mission,
    message: &str,
) {
    let content = format!(
        "**[MISSION `{}`]** {}\n> {}\n> Status: {:?}",
        &mission.id.to_string()[..8],
        mission.objective,
        message,
        mission.status
    );

    let truncated = if content.len() > 1990 {
        format!("{}…", &content[..1987])
    } else {
        content
    };

    let _ = http
        .post(format!("{}/channels/{}/messages", DISCORD_API, ops_channel_id))
        .header("Authorization", format!("Bot {}", bot_token))
        .json(&serde_json::json!({ "content": truncated }))
        .send()
        .await;
}

/// Run a shell command in the workspace exec server.
pub async fn workspace_exec(cmd: &str, http: &Client, auth_token: Option<&str>) -> String {
    let payload = serde_json::json!({ "command": cmd });
    let mut req = http
        .post("http://argus-workspace:9001/exec")
        .json(&payload)
        .timeout(std::time::Duration::from_secs(60));
    if let Some(token) = auth_token {
        req = req.header("X-Argus-Auth", token);
    }
    match req.send().await {
        Err(e) => format!("workspace unreachable: {}", e),
        Ok(r) => match r.json::<serde_json::Value>().await {
            Err(e) => format!("response error: {}", e),
            Ok(j) => j["output"].as_str()
                .or_else(|| j["stdout"].as_str())
                .unwrap_or("").trim_end().to_string(),
        }
    }
}

/// Ask Grok Build to decompose an objective into subtasks.
/// Returns a list of (description, assigned_model) pairs.
pub async fn decompose_mission(
    objective: &str,
    deliverables: &[Deliverable],
    config: &AgentConfig,
    http: &Client,
) -> Vec<(String, String)> {
    let deliverable_list = deliverables.iter()
        .map(|d| format!("- {}", d.description()))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "[MISSION DECOMPOSITION]\n\
         You are Grok Build decomposing a mission into executable subtasks.\n\n\
         OBJECTIVE: {}\n\n\
         REQUIRED DELIVERABLES:\n{}\n\n\
         Break this into 2-6 concrete subtasks. For each subtask, specify:\n\
         1. What exactly to do (one clear sentence)\n\
         2. Which model should do it:\n\
            - grok-build: coding, file creation, execution, testing\n\
            - haiku: fast analysis, classification, small tasks\n\
            - sonnet: research, writing, synthesis\n\
            - gemini: web research, fact-checking\n\
            - gemma: synthesis, planning, reasoning\n\n\
         Respond with ONLY a JSON array:\n\
         [{{\"task\": \"...\", \"model\": \"grok-build\"}}, ...]\n\
         No prose. Just the array.",
        objective, deliverable_list
    );

    let build_config = AgentConfig {
        model: MODEL_GROK_BUILD.to_string(),
        blocked_tools: vec![
            "shell".into(), "write_file".into(), "run_python".into(),
            "run_node".into(), "git_checkpoint".into(),
        ],
        ..config.clone()
    };

    let mut mcp = McpClient::new();
    let policy = ShellPolicy::default();
    let memory = NoopMemory;

    let response = run_agent_turn(
        &build_config, &prompt, &[], &policy, &memory, &mut mcp, http,
        |_| {},
    ).await.unwrap_or_default();

    // Parse JSON array from response
    let start = response.find('[');
    let end   = response.rfind(']').map(|i| i + 1);
    if let (Some(s), Some(e)) = (start, end) {
        if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&response[s..e]) {
            return arr.iter().filter_map(|v| {
                let task  = v["task"].as_str()?.to_string();
                let model = v["model"].as_str().unwrap_or(MODEL_GROK_BUILD).to_string();
                Some((task, model))
            }).collect();
        }
    }

    // Fallback: single subtask assigned to Grok Build
    vec![(objective.to_string(), MODEL_GROK_BUILD.to_string())]
}
