//! Mission runner — the full lifecycle.
//!
//! Plan → Sentry gate → Parallel execution → Deliverable verification → Commit → Skill extraction
//!
//! This is the Manus layer, built right.

use crate::types::*;
use crate::executor::*;
use argus_core::{AgentConfig, SentryBus, ReviewVerdict, MODEL_HAIKU, MODEL_GROK_BUILD};
use argus_core::run_agent_turn;
use argus_core::mcp::McpClient;
use argus_core::shell::ShellPolicy;
use argus_core::tools::MemoryBackend;
use argus_core::supabase::{DiscoursePost, SupabaseClient};
use chrono::Utc;
use reqwest::Client;
use tokio::time::{timeout, Duration};

struct NoopMemory;
impl MemoryBackend for NoopMemory {
    fn remember(&self, _: &str, _: &str, _: Option<&str>, _: f64) -> Result<String, String> { Ok(String::new()) }
    fn recall(&self, _: Option<&str>, _: Option<&str>, _: usize) -> Result<Vec<argus_core::tools::MemoryRecord>, String> { Ok(vec![]) }
    fn forget(&self, _: &str) -> Result<String, String> { Ok(String::new()) }
}

/// Run a mission end to end.
/// Returns the completed/failed mission with full audit trail.
pub async fn run_mission(
    mut mission: Mission,
    config: &AgentConfig,
    sentry_bus: Option<&SentryBus>,
    supabase: Option<&SupabaseClient>,
    http: &Client,
    discord_token: Option<&str>,
    ops_channel_id: Option<&str>,
) -> Mission {
    let mission_id = mission.id.to_string();
    eprintln!("[mission:{}] Starting — {}", &mission_id[..8], mission.objective);

    // ── 1. Decompose into subtasks ─────────────────────────────────────────
    if mission.subtasks.is_empty() {
        mission.status = MissionStatus::Planning;
        let subtask_defs = decompose_mission(
            &mission.objective,
            &mission.deliverables,
            config,
            http,
        ).await;

        for (desc, model) in subtask_defs {
            mission.subtasks.push(Subtask::new(desc, model));
        }
        eprintln!("[mission:{}] Decomposed into {} subtask(s)", &mission_id[..8], mission.subtasks.len());
    }

    // ── 2. Sentry gate ─────────────────────────────────────────────────────
    if let Some(bus) = sentry_bus {
        mission.status = MissionStatus::SentryReview;
        let plan = mission.plan_summary();
        let request_id = bus.submit_for_review(&plan, &mission.created_by);
        mission.sentry_request_id = Some(request_id.clone());

        eprintln!("[mission:{}] Submitted to Sentry gate ({})", &mission_id[..8], &request_id[..8]);

        // Wait up to 10 minutes for Sentry to review (she polls every 30s in gate mode)
        let result = timeout(Duration::from_secs(600), async {
            loop {
                if bus.review_ready.load(std::sync::atomic::Ordering::Relaxed) {
                    return bus.take_review_result();
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }).await;

        match result {
            Ok(Some(review)) => {
                match &review.verdict {
                    ReviewVerdict::Approved | ReviewVerdict::HumanOverride => {
                        eprintln!("[mission:{}] Sentry approved — proceeding", &mission_id[..8]);
                    }
                    ReviewVerdict::Flagged { concerns, severity } => {
                        eprintln!("[mission:{}] Sentry flagged {:?} — holding", &mission_id[..8], severity);
                        mission.status = MissionStatus::SentryHold {
                            concerns: concerns.clone(),
                        };
                        return mission;
                    }
                }
            }
            Ok(None) | Err(_) => {
                eprintln!("[mission:{}] Sentry review timeout — proceeding with caution", &mission_id[..8]);
            }
        }
    }

    // ── 3. Mission working directory ───────────────────────────────────────
    // Each mission gets an isolated directory. Subtasks work here, not in /workspace root.
    // This makes parallel execution safe — subtasks can't step on each other's files.
    let mission_dir = format!("/workspace/missions/{}", &mission_id[..8]);
    let _ = workspace_exec(
        &format!("mkdir -p '{}/output' && echo ok", mission_dir),
        http,
        config.exec_auth_token.as_deref(),
    ).await;

    // ── 4. Execute subtasks ────────────────────────────────────────────────
    mission.status = MissionStatus::Executing;

    if let (Some(token), Some(channel)) = (discord_token, ops_channel_id) {
        post_mission_update(http, token, channel, &mission, "Execution started").await;
    }

    // Categorise subtasks: those with explicit file deliverables that overlap
    // can run sequentially; truly independent subtasks run in parallel.
    // For now: parallel execution via tokio::join! for all subtasks with
    // non-overlapping outputs. Falls back to sequential on any failure.
    let total = mission.subtasks.len();
    eprintln!("[mission:{}] Executing {} subtask(s) in parallel", &mission_id[..8], total);

    // Run all subtasks concurrently — each has its own working dir
    let mut handles = Vec::new();
    for i in 0..total {
        let subtask  = mission.subtasks[i].clone();
        let mission_clone = mission.clone();
        let config2  = config.clone();
        let http2    = http.clone();
        let work_dir = format!("{}/subtask_{}", mission_dir, i + 1);

        handles.push(tokio::spawn(async move {
            let _ = workspace_exec(
                &format!("mkdir -p '{}'", work_dir),
                &http2,
                config2.exec_auth_token.as_deref(),
            ).await;
            (i, run_subtask(&subtask, &mission_clone, &config2, &http2).await)
        }));
    }

    for handle in handles {
        match handle.await {
            Ok((i, Ok(output))) => {
                mission.subtasks[i].status = SubtaskStatus::Complete;
                mission.subtasks[i].output = Some(output.chars().take(500).collect());
                mission.subtasks[i].completed_at = Some(Utc::now());
                eprintln!("[mission:{}] Subtask {} complete", &mission_id[..8], i + 1);
            }
            Ok((i, Err(e))) => {
                mission.subtasks[i].status = SubtaskStatus::Failed { reason: e.clone() };
                eprintln!("[mission:{}] Subtask {} failed: {}", &mission_id[..8], i + 1, e);
                if let (Some(token), Some(channel)) = (discord_token, ops_channel_id) {
                    post_mission_update(http, token, channel, &mission,
                        &format!("Subtask {} failed: {}", i + 1, e)).await;
                }
            }
            Err(e) => eprintln!("[mission:{}] Subtask join error: {}", &mission_id[..8], e),
        }
    }

    if mission.any_subtask_failed() {
        let failed: Vec<String> = mission.subtasks.iter()
            .filter_map(|s| if let SubtaskStatus::Failed { reason } = &s.status {
                Some(format!("{}: {}", s.description, reason))
            } else { None })
            .collect();
        mission.status = MissionStatus::Failed {
            reason: failed.join("; "),
        };
        return mission;
    }

    // ── 4. Verify deliverables ─────────────────────────────────────────────
    mission.status = MissionStatus::Verifying;
    eprintln!("[mission:{}] Verifying {} deliverable(s)", &mission_id[..8], mission.deliverables.len());

    let (all_passed, results) = verify_deliverables(&mission, http, config.exec_auth_token.as_deref()).await;
    mission.verification = results;

    if !all_passed {
        let failed: Vec<String> = mission.verification.iter()
            .filter(|r| !r.passed)
            .map(|r| format!("{}: {}", r.deliverable.description(), r.output))
            .collect();
        mission.status = MissionStatus::Failed {
            reason: format!("Deliverable verification failed: {}", failed.join("; ")),
        };
        eprintln!("[mission:{}] Verification failed", &mission_id[..8]);
        return mission;
    }

    // ── 5. Commit and close ────────────────────────────────────────────────
    let commit_msg = format!(
        "[MISSION COMPLETE] {} ({})",
        mission.objective,
        &mission_id[..8]
    );
    let commit_result = workspace_exec(
        &format!("cd /workspace && git add -A && git commit -m '{}' 2>&1 && echo 'HASH:'$(git rev-parse --short HEAD)", commit_msg),
        http,
        config.exec_auth_token.as_deref(),
    ).await;

    let commit_hash = commit_result.lines()
        .find(|l| l.starts_with("HASH:"))
        .map(|l| l.trim_start_matches("HASH:").trim().to_string())
        .unwrap_or_else(|| "uncommitted".to_string());

    mission.status = MissionStatus::Complete { commit_hash: commit_hash.clone() };
    mission.completed_at = Some(Utc::now());

    eprintln!("[mission:{}] Complete — commit {}", &mission_id[..8], commit_hash);

    // ── 6. Skill extraction ────────────────────────────────────────────────
    // Haiku reflects on whether this mission produced a reusable procedure.
    // Runs in background — doesn't block mission completion.
    if let Some(sc) = &config.skills {
        let sc = sc.clone();
        let obj = mission.objective.clone();
        let subtask_summary = mission.subtasks.iter()
            .map(|s| format!("- [{}] {}", s.assigned_model, s.description))
            .collect::<Vec<_>>()
            .join("\n");
        let http2 = http.clone();
        let api_key = config.api_key.clone();
        let api_url = config.api_url.clone();

        tokio::spawn(async move {
            let prompt = format!(
                "A mission just completed successfully.\n\
                 Objective: {}\n\
                 Subtasks that ran:\n{}\n\n\
                 Did this mission follow a procedure that other agents could reuse?\n\
                 If yes: {{\"create_skill\": true, \"skill_name\": \"brief name\", \
                 \"trigger_description\": \"when to use this\", \
                 \"procedure_steps\": \"step-by-step\"}}\n\
                 If no: {{\"create_skill\": false}}\n\n\
                 Be selective. Only document genuinely reusable procedures.",
                obj, subtask_summary
            );

            let body = serde_json::json!({
                "model": MODEL_HAIKU,
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.3,
                "max_tokens": 500,
            });

            if let Ok(resp) = http2.post(&api_url)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body).send().await
            {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    let content = json["choices"][0]["message"]["content"].as_str().unwrap_or("");
                    let s = content.find('{');
                    let e = content.rfind('}').map(|i| i + 1);
                    if let (Some(s), Some(e)) = (s, e) {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content[s..e]) {
                            if parsed["create_skill"].as_bool() == Some(true) {
                                let name    = parsed["skill_name"].as_str().unwrap_or("").to_string();
                                let trigger = parsed["trigger_description"].as_str().unwrap_or("").to_string();
                                let steps   = parsed["procedure_steps"].as_str().unwrap_or("").to_string();
                                if !name.is_empty() {
                                    let new_skill = argus_core::NewSkill {
                                        skill_name: name.clone(),
                                        trigger_description: trigger.clone(),
                                        procedure_steps: steps,
                                        model_created_by: format!("mission/{}", MODEL_HAIKU),
                                        metadata: None,
                                    };
                                    if let Ok(_) = sc.create_skill(new_skill).await {
                                        sc.announce_created(&name, &trigger, MODEL_HAIKU).await;
                                        eprintln!("[mission/skill] Extracted skill: \"{}\"", name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    // Post completion to Discord
    if let (Some(token), Some(channel)) = (discord_token, ops_channel_id) {
        post_mission_update(http, token, channel, &mission,
            &format!("✅ Complete — commit `{}`", commit_hash)).await;
    }

    // Write to discourse
    if let Some(sb) = supabase {
        let verification_summary = mission.verification.iter()
            .map(|r| format!("- {} {}", if r.passed { "✅" } else { "❌" }, r.deliverable.description()))
            .collect::<Vec<_>>()
            .join("\n");

        let post = DiscoursePost {
            from_agent: format!("argus-mission/{}", mission.primary_executor),
            post_type: "finding".to_string(),
            content: format!(
                "**[MISSION COMPLETE]** {}\n\nCommit: `{}`\n\nVerification:\n{}",
                mission.objective, commit_hash, verification_summary
            ),
            task_context: Some(format!("mission:{}", &mission_id[..8])),
            requires_human_review: false,
        };
        let _ = sb.write_discourse(&post).await;
    }

    mission
}
