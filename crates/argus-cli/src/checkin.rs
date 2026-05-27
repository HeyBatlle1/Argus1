//! Autonomous check-in loop — active agent mode
//!
//! ## Overview
//!
//! The check-in loop wakes on `interval_minutes`, respects quiet hours, and
//! performs two distinct roles:
//!
//! **Alert mode** — fires immediately when thresholds are exceeded:
//!   - Disk usage > 80 %
//!   - Memory usage > 90 %
//!   - Any container unhealthy or exited unexpectedly
//!
//! **Daily heartbeat** — fires once per calendar day (first tick after midnight).
//!   Previously this sent only a system-status summary.  After the May 2026
//!   upgrade it also runs `run_agent_checkin`, which calls `run_agent_turn` so
//!   the agent can make a genuine observation rather than just reporting metrics.
//!
//! **Weekly tech sweep** — replaces the daily heartbeat on Sunday midnight.
//!   The agent searches for Rust crate updates, AI/agent developments, and open
//!   codebase items, then posts a tagged `[WEEKLY SWEEP]` finding to Discord.
//!
//! ## Telegram message format (after upgrade)
//!
//! ```text
//! ✅ Argus Daily — 2026-05-26 00:00:00
//!
//! Finding: I noticed the semantic memory prefetch fired 38 times on
//! queries under 5 words despite the short-query guard. Possible guard
//! threshold drift. Posted details to #findings.
//!
//! Disk: 42% | Memory: 61% | Containers: healthy
//! [Full finding posted to Discord #findings]
//! ```
//!
//! Alert messages suppress the finding — the alert takes priority.

use argus_core::agent::{AgentConfig, AgentEvent, MODEL_GEMINI, MODEL_OPUS, MODEL_SONNET};
use argus_core::mcp::McpClient;
use argus_core::shell::ShellPolicy;
use argus_core::supabase::{CheckinLogEntry, DiscoursePost, SupabaseClient};
use argus_core::tools::MemoryBackend;
use argus_core::run_agent_turn;
use chrono::{Datelike, Local, NaiveDate, Timelike};
use reqwest::Client;
use tokio::time::{sleep, Duration};

const TELEGRAM_API: &str = "https://api.telegram.org";
const DISK_ALERT_PCT: u8 = 80;
const MEM_ALERT_PCT: u8 = 90;

/// Entry point — spawns the check-in loop as a background task.
///
/// `agent_config` gives the loop full agent capabilities (API key, model,
/// embedding, skills, audit chain) so it can call `run_agent_turn` on the
/// daily and weekly heartbeats rather than only reporting system metrics.
pub fn spawn_checkin_loop(
    supabase: SupabaseClient,
    bot_token: String,
    chat_id: i64,
    agent_config: AgentConfig,
) {
    tokio::spawn(async move {
        run_checkin_loop(supabase, bot_token, chat_id, agent_config).await;
    });
}

async fn run_checkin_loop(
    supabase: SupabaseClient,
    bot_token: String,
    chat_id: i64,
    agent_config: AgentConfig,
) {
    let client = Client::new();
    let mut last_daily: Option<NaiveDate> = None;

    loop {
        let checkin_cfg = supabase.read_checkin_config().await;

        if checkin_cfg.telegram_enabled && !in_quiet_hours(&checkin_cfg) {
            let health = collect_system_health().await;
            let today = Local::now().date_naive();

            let needs_alert = health.disk_pct > DISK_ALERT_PCT
                || health.mem_pct > MEM_ALERT_PCT
                || health.has_unhealthy_container;

            // Daily heartbeat: first tick after midnight each calendar day
            let needs_daily = last_daily.map_or(true, |d| d < today);

            // Weekly tech sweep: replaces the daily heartbeat on Sunday midnight.
            // Rotates models by ISO week number so different models run each week.
            let is_weekly_sweep = needs_daily && {
                let now = Local::now();
                now.weekday() == chrono::Weekday::Sun && now.hour() == 0
            };

            if needs_alert || needs_daily {
                let schedule_summary = read_schedule_summary(&supabase).await;

                // Run the agent turn on daily heartbeats (not on pure alerts — alert
                // messages are time-sensitive and we don't want to delay them).
                let agent_finding: Option<String> = if needs_daily && !needs_alert {
                    run_agent_checkin(&supabase, &agent_config, &health, is_weekly_sweep).await
                } else {
                    None
                };

                // Post finding to the discourse table — pg_net trigger fires automatically
                // and relays the post to Discord #findings.
                if let Some(ref finding) = agent_finding {
                    post_finding_to_discourse(&supabase, &agent_config, finding, is_weekly_sweep).await;
                }

                let message = format_checkin_message(
                    &health,
                    &schedule_summary,
                    needs_alert,
                    needs_daily,
                    agent_finding.as_deref(),
                );

                if let Err(e) = send_telegram_message(&client, &bot_token, chat_id, &message).await
                {
                    eprintln!("[checkin] Failed to send Telegram message: {}", e);
                } else {
                    if needs_daily {
                        last_daily = Some(today);
                    }

                    let entry = CheckinLogEntry {
                        checkin_type: checkin_cfg.checkin_type.clone(),
                        status: if needs_alert { "alert" } else { "daily" }.to_string(),
                        message_sent: message,
                        system_health: Some(serde_json::to_value(&health).unwrap_or_default()),
                    };
                    if let Err(e) = supabase.write_checkin_log(&entry).await {
                        eprintln!("[checkin] Failed to write checkin log: {}", e);
                    }
                }
            }
            // else: everything healthy, not daily time — silent pass
        }

        let interval = Duration::from_secs(checkin_cfg.interval_minutes.max(1) as u64 * 60);
        sleep(interval).await;
    }
}

fn in_quiet_hours(config: &argus_core::supabase::CheckinConfig) -> bool {
    let now_h = Local::now().hour();
    let start_h = config.quiet_hours_start.unwrap_or(23) as u32;
    let end_h   = config.quiet_hours_end.unwrap_or(7) as u32;

    // start_h > end_h means the window spans midnight (e.g. 23 → 07)
    if start_h > end_h {
        now_h >= start_h || now_h < end_h
    } else {
        now_h >= start_h && now_h < end_h
    }
}

// ── NoopMemory ─────────────────────────────────────────────────────────────
//
// The check-in agent turn doesn't need persistent memory — it has the full
// semantic embedding client for recall via pgvector.  We satisfy the
// MemoryBackend parameter with a noop implementation so the call compiles
// without pulling in argus-memory as a checkin dependency.

struct NoopMemory;

impl MemoryBackend for NoopMemory {
    fn remember(&self, _: &str, _: &str, _: Option<&str>, _: f64) -> Result<String, String> {
        Ok("(memory disabled in check-in mode — use the embedding client)".to_string())
    }
    fn recall(&self, _: Option<&str>, _: Option<&str>, _: usize) -> Result<Vec<argus_core::tools::MemoryRecord>, String> {
        Ok(vec![])
    }
    fn forget(&self, _: &str) -> Result<String, String> {
        Ok("(memory disabled in check-in mode)".to_string())
    }
}

// ── Agent check-in ─────────────────────────────────────────────────────────

/// Drive one agentic check-in turn — the heart of the May 2026 upgrade.
///
/// ## What it does
///
/// Builds three context blocks and passes them to `run_agent_turn` as the
/// user message (the agent self-initiates; there is no human user in this
/// path):
///
/// 1. **System health** — current disk/memory/container snapshot.
/// 2. **Recent intranet activity** — last 5 discourse posts from any agent.
/// 3. **Audit activity** — today's logged tool-call count and chain integrity.
///
/// The agent is then free to use any tool (web_search, recall, read_file, …)
/// to investigate something worth noting.  Its text response is returned as
/// the "finding" string.
///
/// ## Weekly sweep vs daily observation
///
/// When `is_weekly` is true the prompt switches to a deeper tech sweep:
/// crate-update search, AI/agent developments, open codebase items.
/// The model is also rotated by ISO week number so the sweep runs on a
/// different brain each week (Sonnet → Gemini → Opus → repeat).
///
/// ## Failure handling
///
/// Returns `None` on any failure so the Telegram message can still be sent
/// (without a finding block). Errors are logged to stderr.
async fn run_agent_checkin(
    supabase: &SupabaseClient,
    config: &AgentConfig,
    health: &SystemHealth,
    is_weekly: bool,
) -> Option<String> {
    // ── Context: intranet discourse ─────────────────────────────────────
    let discourse_block = match supabase.read_recent_discourse(5, None).await {
        Ok(posts) if !posts.is_empty() => {
            let mut block = String::from("RECENT INTRANET ACTIVITY (last 5 posts):\n");
            for post in &posts {
                let ts = post.created_at.as_deref().unwrap_or("unknown time");
                let snippet = if post.content.chars().count() > 200 {
                    format!("{}…", post.content.chars().take(200).collect::<String>())
                } else {
                    post.content.clone()
                };
                block.push_str(&format!(
                    "\n[{} | {}] {}: {}",
                    ts, post.post_type, post.from_agent, snippet
                ));
            }
            block
        }
        Ok(_) => "RECENT INTRANET ACTIVITY: No posts in the last period.".to_string(),
        Err(e) => {
            eprintln!("[checkin] Discourse read failed (continuing): {}", e);
            "RECENT INTRANET ACTIVITY: Unavailable.".to_string()
        }
    };

    // ── Context: audit chain activity ────────────────────────────────────
    let audit_block = if let Some(ref audit) = config.audit {
        match audit.entry_count_today() {
            Ok(n)  => format!("AUDIT ACTIVITY TODAY: {} tool/model calls logged — chain intact.", n),
            Err(e) => format!("AUDIT ACTIVITY TODAY: Read error — {}", e),
        }
    } else {
        "AUDIT ACTIVITY TODAY: Audit chain not configured.".to_string()
    };

    // ── Context: system health ───────────────────────────────────────────
    let health_block = format!(
        "SYSTEM HEALTH:\nDisk: {} | Memory: {} | Containers: {}",
        health.disk,
        health.memory,
        if health.has_unhealthy_container {
            format!("ISSUE DETECTED\n{}", health.containers)
        } else {
            "healthy".to_string()
        }
    );

    // ── Determine effective model ────────────────────────────────────────
    // Weekly sweep rotates across Sonnet → Gemini → Opus by ISO week number.
    // Daily observation uses whatever model the daemon is currently configured with.
    let weekly_config;
    let effective_config: &AgentConfig = if is_weekly {
        let week_num = Local::now().iso_week().week();
        let sweep_model = match week_num % 3 {
            0 => MODEL_SONNET,
            1 => MODEL_GEMINI,
            _ => MODEL_OPUS,
        };
        eprintln!("[checkin] Weekly sweep — using model: {} (ISO week {})", sweep_model, week_num);
        weekly_config = AgentConfig {
            model: sweep_model.to_string(),
            ..config.clone()
        };
        &weekly_config
    } else {
        config
    };

    // ── Build prompt ─────────────────────────────────────────────────────
    let prompt = if is_weekly {
        format!(
            r#"[WEEKLY SWEEP] You are running the weekly technology sweep for Argus.
This replaces the daily check-in this week.

{}

{}

{}

YOUR TASKS:
1. Search for updates to the key Rust crates Argus depends on:
   tokio, axum, wasmtime, reqwest, serde, rusqlite, uuid
2. Search for relevant AI/agent developments this week that could
   affect Argus architecture or capability.
3. Note any open codebase items you're aware of:
   - .unwrap() cleanup
   - Ollama provider support
   - Linux keychain fallback
4. Write a concise findings report:
   - Any crate updates worth applying (name version numbers)
   - Any architectural observations
   - Any open item progress recommendations
   - Tag anything requiring architectural discussion with [NEEDS REVIEW]

Be specific. Name versions. This report goes directly to Discord #findings."#,
            health_block, discourse_block, audit_block
        )
    } else {
        format!(
            r#"You are running your daily check-in. This is your window to be useful,
not just to report status.

{}

{}

{}

Your task:
1. Review the above context.
2. Identify ONE thing worth investigating, noting, or improving — something
   concrete you can actually act on in the next few minutes.
3. Do it — use your tools if needed (web_search, recall, http_request).
4. Write a brief finding of 3–5 sentences.

Do not summarise system status — that is handled separately.
Do not report that everything is fine — find something worth saying.
Return only your finding, ready to be posted to Discord #findings."#,
            health_block, discourse_block, audit_block
        )
    };

    // ── Run the agent turn ───────────────────────────────────────────────
    // Empty history (self-initiated turn), empty MCP (no servers needed),
    // default shell policy (blocks HIGH risk), noop memory (embedding handles recall).
    let http = reqwest::Client::new();
    let mut mcp = McpClient::new();
    let shell_policy = ShellPolicy::default();
    let memory = NoopMemory;

    eprintln!(
        "[checkin] Running {} check-in with model {}",
        if is_weekly { "weekly sweep" } else { "daily" },
        effective_config.model
    );

    match run_agent_turn(
        effective_config,
        &prompt,
        &[],   // no prior conversation history — agent self-initiates
        &shell_policy,
        &memory,
        &mut mcp,
        &http,
        |event| {
            if let AgentEvent::ToolCall { name, preview, .. } = event {
                eprintln!("[checkin] tool: {} — {}", name, preview);
            }
        },
    )
    .await
    {
        Ok(finding) => {
            eprintln!("[checkin] Agent finding ({} chars)", finding.len());
            Some(finding)
        }
        Err(e) => {
            eprintln!("[checkin] Agent turn failed: {}", e);
            None
        }
    }
}

/// Write the agent's finding to the discourse table.
///
/// The pg_net trigger on `argus_agent_discourse` fires automatically and
/// relays the post to Discord #findings — no direct Discord API call needed.
async fn post_finding_to_discourse(
    supabase: &SupabaseClient,
    config: &AgentConfig,
    finding: &str,
    is_weekly: bool,
) {
    let label = if is_weekly { "Weekly Tech Sweep" } else { "Daily Observation" };
    let post = DiscoursePost {
        from_agent: format!("argus-checkin/{}", config.model),
        post_type: if is_weekly { "finding".to_string() } else { "reflection".to_string() },
        content: format!("**[ARGUS CHECKIN] {}**\n\n{}", label, finding),
        task_context: Some("scheduled_checkin".to_string()),
        requires_human_review: false,
    };
    if let Err(e) = supabase.write_discourse(&post).await {
        eprintln!("[checkin] Failed to post finding to discourse: {}", e);
    } else {
        eprintln!("[checkin] Finding posted to Discord #findings via discourse table");
    }
}

// ── System health ──────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
struct SystemHealth {
    timestamp: String,
    containers: String,
    disk: String,
    memory: String,
    /// Percentage of disk used (0–100).
    disk_pct: u8,
    /// Percentage of memory used (0–100).
    mem_pct: u8,
    /// True if any container is unhealthy or exited unexpectedly.
    has_unhealthy_container: bool,
}

async fn collect_system_health() -> SystemHealth {
    use tokio::process::Command;
    use tokio::time::{timeout, Duration};

    async fn run(cmd: &'static str) -> String {
        match timeout(Duration::from_secs(3), Command::new("sh").arg("-c").arg(cmd).output()).await
        {
            Ok(Ok(o)) if o.status.success() => {
                String::from_utf8_lossy(&o.stdout).trim().to_string()
            }
            _ => "unavailable".to_string(),
        }
    }

    // ── Memory ─────────────────────────────────────────────────────────────
    let (memory, mem_pct) = {
        let content = tokio::fs::read_to_string("/proc/meminfo")
            .await
            .unwrap_or_default();
        let mut total_kb: u64 = 0;
        let mut available_kb: u64 = 0;
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 { continue; }
            match parts[0] {
                "MemTotal:"     => { total_kb     = parts[1].parse().unwrap_or(0); }
                "MemAvailable:" => { available_kb = parts[1].parse().unwrap_or(0); }
                _ => {}
            }
        }
        if total_kb == 0 {
            ("unavailable".to_string(), 0u8)
        } else {
            let fmt = |kb: u64| -> String {
                if kb >= 1_048_576 {
                    format!("{:.1}G", kb as f64 / 1_048_576.0)
                } else {
                    format!("{}M", kb / 1024)
                }
            };
            let used = total_kb.saturating_sub(available_kb);
            let pct = ((used as f64 / total_kb as f64) * 100.0).round() as u8;
            (format!("{} used, {} free ({}%)", fmt(used), fmt(available_kb), pct), pct)
        }
    };

    // ── Disk ───────────────────────────────────────────────────────────────
    let (disk, disk_pct) = {
        let summary = run(
            "df -h / 2>/dev/null | tail -1 | awk '{print $3\"/\"$2\" used, \"$4\" free\"}'",
        )
        .await;
        let pct_str = run("df / 2>/dev/null | tail -1 | awk '{print $5}' | tr -d '%'").await;
        let pct = pct_str.parse::<u8>().unwrap_or(0);
        let display = if pct > 0 {
            format!("{} ({}%)", summary, pct)
        } else {
            summary
        };
        (display, pct)
    };

    // ── Containers ─────────────────────────────────────────────────────────
    let containers = run(
        "docker ps --format '{{.Names}} ({{.Status}})' 2>/dev/null | head -10",
    )
    .await;

    let has_unhealthy_container = containers
        .lines()
        .any(|line| {
            let l = line.to_lowercase();
            l.contains("unhealthy") || l.contains("exited") || l.contains("restarting")
        });

    SystemHealth {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S %Z").to_string(),
        containers,
        disk,
        memory,
        disk_pct,
        mem_pct,
        has_unhealthy_container,
    }
}

// ── Formatting ─────────────────────────────────────────────────────────────

/// Format the Telegram message sent to Bradlee.
///
/// ## Message layout (daily heartbeat)
///
/// ```text
/// ✅ Argus Daily — 2026-05-26 00:00:00
///
/// Finding: I noticed the semantic memory prefetch fired 38 times on
/// queries under 5 words despite the short-query guard. Possible guard
/// threshold drift. Full details posted to Discord #findings.
///
/// Disk: 42/100G used, 58G free (42%) | Memory: 3.1G used, 1.2G free (72%)
/// Containers: healthy
/// ```
///
/// When `is_alert` is true the finding block is suppressed — alert messages
/// are time-sensitive and must be read at a glance.
fn format_checkin_message(
    health: &SystemHealth,
    schedule: &str,
    is_alert: bool,
    is_daily: bool,
    finding: Option<&str>,
) -> String {
    let mut msg = if is_alert {
        format!("⚠️ Argus Alert — {}\n\n", health.timestamp)
    } else {
        format!("✅ Argus Daily — {}\n\n", health.timestamp)
    };

    // Agent finding — daily heartbeat only, never on pure alerts
    if is_daily && !is_alert {
        if let Some(text) = finding {
            // Truncate to ~300 chars so Telegram stays readable on mobile
            let snippet = if text.len() > 300 {
                format!("{}…", text.chars().take(300).collect::<String>())
            } else {
                text.to_string()
            };
            msg.push_str(&format!("Finding: {}\n\n", snippet));
            msg.push_str("[Full finding posted to Discord #findings]\n\n");
        }
    }

    // System metrics — alert: show only breached metrics; daily: show all
    if health.disk_pct > DISK_ALERT_PCT {
        msg.push_str(&format!("🔴 Disk: {} — ABOVE {}% THRESHOLD\n", health.disk, DISK_ALERT_PCT));
    } else if is_daily {
        msg.push_str(&format!("Disk: {}\n", health.disk));
    }

    if health.mem_pct > MEM_ALERT_PCT {
        msg.push_str(&format!("🔴 Memory: {} — ABOVE {}% THRESHOLD\n", health.memory, MEM_ALERT_PCT));
    } else if is_daily {
        msg.push_str(&format!("Memory: {}\n", health.memory));
    }

    if health.has_unhealthy_container {
        msg.push_str(&format!("\n🔴 Container issue:\n{}\n", health.containers));
    } else if is_daily {
        msg.push_str("Containers: healthy\n");
    }

    if !schedule.is_empty() && is_daily {
        msg.push_str(&format!("\nUpcoming:\n{}", schedule));
    }

    msg
}

// ── Schedule ───────────────────────────────────────────────────────────────

async fn read_schedule_summary(supabase: &SupabaseClient) -> String {
    match supabase.read_upcoming_schedule().await {
        Err(_) => String::new(),
        Ok(rows) => {
            let arr = match rows.as_array() {
                Some(a) if !a.is_empty() => a,
                _ => return String::new(),
            };
            let items: Vec<String> = arr.iter().take(3).filter_map(|r| {
                let title = r["title"].as_str().or(r["task"].as_str())?;
                let time = r["scheduled_time"].as_str().unwrap_or("");
                Some(format!("• {} at {}", title, time))
            }).collect();
            if items.is_empty() { String::new() } else { items.join("\n") }
        }
    }
}

// ── Telegram ───────────────────────────────────────────────────────────────

async fn send_telegram_message(
    client: &Client,
    bot_token: &str,
    chat_id: i64,
    text: &str,
) -> Result<(), String> {
    let url = format!("{}/bot{}/sendMessage", TELEGRAM_API, bot_token);
    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text":    text,
        }))
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Telegram API error: {}", body));
    }

    Ok(())
}
