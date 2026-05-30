//! Argus Web Server — axum WebSocket + REST API for the Next.js frontend

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::Method,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tower_http::cors::{Any, CorsLayer};

use argus_core::{AgentConfig, AgentEvent, ConversationMessage, EmbeddingClient, McpClient, MemoryBackend, ShellPolicy, MODEL_HAIKU, MODEL_SONNET, MODEL_OPUS, MODEL_GROK, MODEL_GROK_BUILD, MODEL_GROK_MULTI};
use argus_core::shell::PermissionPrompter;
use argus_memory::sqlite::{ConversationMeta, SqliteMemory};

// ─── WebSocket message types (mirrors TypeScript protocol) ─────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    UserMessage { content: String },
    SwitchModel { model: String },
    Cancel,
    NewConversation,
    LoadConversation { id: String },
    ListConversations,
}

/// Memory record serialized to match the frontend Memory type
#[derive(Debug, Serialize, Clone)]
struct MemoryPayload {
    id: String,
    content: String,
    #[serde(rename = "type")]
    memory_type: String,
    importance: f64,
    #[serde(rename = "createdAt")]
    created_at: String,
}

/// A single message as sent to the frontend for history replay.
#[derive(Debug, Serialize, Clone)]
struct HistoryMessagePayload {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}

/// Conversation metadata as sent to the frontend.
#[derive(Debug, Serialize, Clone)]
struct ConversationPayload {
    id: String,
    title: String,
    surface: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(rename = "messageCount")]
    message_count: i64,
    #[serde(rename = "startedAt")]
    started_at: String,
    #[serde(rename = "lastActiveAt")]
    last_active_at: String,
}

impl From<ConversationMeta> for ConversationPayload {
    fn from(m: ConversationMeta) -> Self {
        Self {
            id: m.id,
            title: m.title,
            surface: m.surface,
            model: m.model,
            message_count: m.message_count,
            started_at: m.started_at,
            last_active_at: m.last_active_at,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    Thinking,
    ToolCall {
        name: String,
        args: serde_json::Value,
        call_id: String,
    },
    ToolResult {
        name: String,
        result: String,
        success: bool,
        call_id: String,
    },
    ResponseComplete {
        content: String,
    },
    Error {
        message: String,
    },
    Status {
        eye_state: String,
        model: String,
    },
    Connected {
        version: String,
        model: String,
        /// Key names present in the vault (no values). Used by the frontend VaultStatus widget.
        vault_keys: Vec<String>,
        /// Names of MCP servers that connected successfully at startup.
        mcp_servers: Vec<String>,
    },
    MemoryUpdate {
        memories: Vec<MemoryPayload>,
    },
    /// Sent on connect (and after LoadConversation) to replay prior messages.
    ConversationHistory {
        id: String,
        messages: Vec<HistoryMessagePayload>,
    },
    /// Full list of past conversations for the sidebar.
    ConversationsList {
        conversations: Vec<ConversationPayload>,
    },
    /// Confirms a new or loaded conversation is active.
    ConversationStarted {
        id: String,
        title: String,
    },
}

// ─── Per-connection state ──────────────────────────────────────────────────

struct ConnectionState {
    config: AgentConfig,
    history: Vec<ConversationMessage>,
    client: reqwest::Client,
    memory: SqliteMemory,
    mcp: McpClient,
    shell_policy: ShellPolicy,
    conversation_id: String,
    conversation_title: String,
}

impl ConnectionState {
    fn new(
        api_key: String,
        brave_key: Option<String>,
        shell_prompter: Option<std::sync::Arc<dyn PermissionPrompter>>,
        exec_auth_token: Option<String>,
        embedding: Option<EmbeddingClient>,
        audit: Option<std::sync::Arc<argus_audit::AuditChain>>,
        discord_bot_token: Option<String>,
        discord_channel_id: Option<u64>,
    ) -> anyhow::Result<Self> {
        let mut config = AgentConfig::new(api_key);
        if let Some(k) = brave_key {
            config.brave_search_key = Some(k);
        }
        config.shell_prompter     = shell_prompter;
        config.exec_auth_token    = exec_auth_token;
        config.embedding          = embedding;
        config.audit              = audit;
        config.discord_bot_token  = discord_bot_token;
        config.discord_channel_id = discord_channel_id;

        let memory = SqliteMemory::open_default()
            .map_err(|e| anyhow::anyhow!("Memory init failed: {}", e))?;

        // Restore the most recent web conversation, or start fresh.
        let (conversation_id, conversation_title, history) =
            match memory.latest_conversation() {
                Ok(Some(meta)) if meta.surface == "web" => {
                    let hist = memory.load_history_str(&meta.id).unwrap_or_default();
                    (meta.id, meta.title, hist)
                }
                _ => {
                    let id = uuid::Uuid::new_v4().to_string();
                    let title = "New Conversation".to_string();
                    let _ = memory.upsert_conversation(&id, &title, "web", None, 0);
                    (id, title, Vec::new())
                }
            };

        let mut mcp = McpClient::new();
        let _ = mcp.connect_all();

        Ok(Self {
            config,
            history,
            client: reqwest::Client::new(),
            memory,
            mcp,
            shell_policy: ShellPolicy::default(),
            conversation_id,
            conversation_title,
        })
    }

    /// Map frontend model ID alias → OpenRouter model ID
    fn apply_model_switch(&mut self, frontend_id: &str) {
        let openrouter_id = match frontend_id {
            "claude-haiku"  => MODEL_HAIKU,
            "claude-sonnet" => MODEL_SONNET,
            "claude-opus"   => MODEL_OPUS,
            "grok"          => MODEL_GROK,
            "grok-build"    => MODEL_GROK_BUILD,
            "grok-multi"    => MODEL_GROK_MULTI,
            "gemini-flash"  => "google/gemini-3.1-flash-lite",
            other           => other, // pass through if already a full ID
        };
        self.config.model = openrouter_id.to_string();
    }

    /// Map OpenRouter model ID → frontend alias
    fn current_frontend_model(&self) -> String {
        match self.config.model.as_str() {
            MODEL_HAIKU  => "claude-haiku".to_string(),
            MODEL_SONNET => "claude-sonnet".to_string(),
            MODEL_OPUS   => "claude-opus".to_string(),
            MODEL_GROK       => "grok".to_string(),
            MODEL_GROK_BUILD  => "grok-build".to_string(),
            MODEL_GROK_MULTI => "grok-multi".to_string(),
            "google/gemini-3.1-flash-lite" => "gemini-flash".to_string(),
            other => other.to_string(),
        }
    }
}

// ─── Shared app state ──────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    api_key: String,
    brave_key: Option<String>,
    /// Vault key names (not values) — sent to the frontend on connect.
    vault_keys: Vec<String>,
    // Daemon-level capabilities forwarded to every WebSocket connection.
    // All are Arc-wrapped so cloning is cheap.
    shell_prompter:     Option<std::sync::Arc<dyn PermissionPrompter>>,
    exec_auth_token:    Option<String>,
    embedding:          Option<EmbeddingClient>,
    audit:              Option<std::sync::Arc<argus_audit::AuditChain>>,
    discord_bot_token:  Option<String>,
    discord_channel_id: Option<u64>,
}

// ─── Router ────────────────────────────────────────────────────────────────

pub async fn run_web_server(
    port: u16,
    config: AgentConfig,
    vault_keys: Vec<String>,
) -> anyhow::Result<()> {
    let state = Arc::new(AppState {
        api_key:            config.api_key,
        brave_key:          config.brave_search_key,
        vault_keys,
        shell_prompter:     config.shell_prompter,
        exec_auth_token:    config.exec_auth_token,
        embedding:          config.embedding,
        audit:              config.audit,
        discord_bot_token:  config.discord_bot_token,
        discord_channel_id: config.discord_channel_id,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(health))
        .route("/ws", get(ws_handler))
        .route("/sentry", get(sentry_handler))
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("[+] Argus web server listening on http://localhost:{}", port);
    println!("[+] WebSocket endpoint: ws://localhost:{}/ws", port);
    println!("[+] Frontend: set NEXT_PUBLIC_WS_URL=ws://localhost:{}/ws", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "Argus online. The hundred eyes are open."
}

// ─── Sentry endpoint ───────────────────────────────────────────────────────
//
// Returns system health data: RAM usage, Docker container states, daemon uptime.
// The Next.js frontend proxies to this endpoint rather than running system
// commands inside the frontend container (where Docker + /proc visibility differ).

/// Read memory stats from /proc/meminfo (Linux standard, always available in container).
/// Returns (used_str, free_str, total_kb, available_kb).
fn read_proc_meminfo() -> (String, String) {
    let content = match std::fs::read_to_string("/proc/meminfo") {
        Ok(s) => s,
        Err(_) => return ("?".into(), "?".into()),
    };

    let mut total_kb: u64 = 0;
    let mut available_kb: u64 = 0;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 { continue; }
        match parts[0] {
            "MemTotal:"     => { total_kb = parts[1].parse().unwrap_or(0); }
            "MemAvailable:" => { available_kb = parts[1].parse().unwrap_or(0); }
            _ => {}
        }
    }

    if total_kb == 0 {
        return ("?".into(), "?".into());
    }

    let used_kb = total_kb.saturating_sub(available_kb);
    let fmt = |kb: u64| -> String {
        if kb >= 1_048_576 {
            format!("{:.1}G", kb as f64 / 1_048_576.0)
        } else {
            format!("{}M", kb / 1024)
        }
    };

    (fmt(used_kb), fmt(available_kb))
}

/// Run `docker ps` with a hard 5-second timeout.
/// Returns empty vec if Docker socket isn't mounted or docker CLI is unavailable.
async fn docker_containers() -> Vec<serde_json::Value> {
    use tokio::process::Command;
    use tokio::time::{timeout, Duration};

    let result = timeout(
        Duration::from_secs(5),
        Command::new("docker")
            .args(["ps", "--format", "{{.Names}}|{{.Status}}|{{.Ports}}"])
            .output(),
    ).await;

    let output = match result {
        Ok(Ok(o)) if o.status.success() => o.stdout,
        _ => return vec![],
    };

    let text = String::from_utf8_lossy(&output);
    text.lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            let name   = parts.first().copied().unwrap_or("").trim().to_string();
            let status = parts.get(1).copied().unwrap_or("").trim().to_string();
            let ports  = parts.get(2).copied().unwrap_or("").trim().to_string();
            let healthy   = status.contains("healthy") && !status.contains("unhealthy");
            let unhealthy = status.contains("unhealthy");
            serde_json::json!({
                "name": name,
                "status": status,
                "ports": ports,
                "healthy": healthy,
                "unhealthy": unhealthy,
            })
        })
        .collect()
}

/// Return basic info about the running daemon process from /proc/self/status.
fn daemon_process_info() -> Vec<serde_json::Value> {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let mut vm_rss_kb: u64 = 0;
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            vm_rss_kb = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            break;
        }
    }
    let mem_str = if vm_rss_kb >= 1024 {
        format!("{:.0}MB", vm_rss_kb as f64 / 1024.0)
    } else {
        format!("{}KB", vm_rss_kb)
    };

    vec![serde_json::json!({
        "name": "argus-daemon",
        "pid": std::process::id().to_string(),
        "mem": mem_str,
        "uptime": ""
    })]
}

async fn sentry_handler() -> impl IntoResponse {
    let (used, free) = read_proc_meminfo();
    let containers = docker_containers().await;
    let processes = daemon_process_info();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    axum::Json(serde_json::json!({
        "memory": { "used": used, "free": free },
        "containers": containers,
        "processes": processes,
        "ts": ts,
    }))
}

/// Load all memories from SQLite and emit a MemoryUpdate message.
fn build_memory_update(memory: &SqliteMemory) -> ServerMessage {
    let records = memory.recall(None, None, 100).unwrap_or_default();
    let payloads = records
        .into_iter()
        .map(|r| MemoryPayload {
            id: r.id.to_string(),
            content: r.content,
            memory_type: r.memory_type,
            importance: r.importance,
            created_at: r.created_at.unwrap_or_else(|| "unknown".to_string()),
        })
        .collect();
    ServerMessage::MemoryUpdate { memories: payloads }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

// ─── WebSocket connection handler ──────────────────────────────────────────

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (ws_tx, ws_rx) = socket.split();
    let ws_tx = Arc::new(Mutex::new(ws_tx));

    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    let ws_tx_writer = Arc::clone(&ws_tx);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = match serde_json::to_string(&msg) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("[ws] serialization error: {}", e);
                    continue;
                }
            };
            let mut sink = ws_tx_writer.lock().await;
            if sink.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    let conn = match ConnectionState::new(
        state.api_key.clone(),
        state.brave_key.clone(),
        state.shell_prompter.clone(),
        state.exec_auth_token.clone(),
        state.embedding.clone(),
        state.audit.clone(),
        state.discord_bot_token.clone(),
        state.discord_channel_id,
    ) {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(ServerMessage::Error { message: e.to_string() });
            return;
        }
    };
    let conn = Arc::new(Mutex::new(conn));

    {
        let c = conn.lock().await;
        let mcp_servers: Vec<String> =
            c.mcp.servers.iter().map(|s| s.name.clone()).collect();
        let _ = tx.send(ServerMessage::Connected {
            version: "0.1.0".to_string(),
            model: c.current_frontend_model(),
            vault_keys: state.vault_keys.clone(),
            mcp_servers,
        });
        let _ = tx.send(build_memory_update(&c.memory));

        // Replay the restored conversation history so the UI is not blank on reconnect.
        if !c.history.is_empty() {
            let messages = c.history.iter().map(|m| HistoryMessagePayload {
                role: m.role.clone(),
                content: m.content.clone(),
                model: m.model.clone(),
            }).collect();
            let _ = tx.send(ServerMessage::ConversationHistory {
                id: c.conversation_id.clone(),
                messages,
            });
        }

        // Send the conversations list for the sidebar.
        let conversations = c.memory.list_conversations(30).unwrap_or_default()
            .into_iter().map(ConversationPayload::from).collect();
        let _ = tx.send(ServerMessage::ConversationsList { conversations });
    }

    let mut ws_rx = ws_rx;
    while let Some(Ok(msg)) = ws_rx.next().await {
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        let client_msg: ClientMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                let _ = tx.send(ServerMessage::Error {
                    message: format!("Invalid message: {}", e),
                });
                continue;
            }
        };

        match client_msg {
            ClientMessage::UserMessage { content } => {
                handle_user_message(content, Arc::clone(&conn), tx.clone()).await;
            }
            ClientMessage::SwitchModel { model } => {
                let mut c = conn.lock().await;
                c.apply_model_switch(&model);
                let new_model = c.current_frontend_model();
                let _ = tx.send(ServerMessage::Status {
                    eye_state: "watching".to_string(),
                    model: new_model,
                });
            }
            ClientMessage::Cancel => {
                let _ = tx.send(ServerMessage::Status {
                    eye_state: "watching".to_string(),
                    model: {
                        let c = conn.lock().await;
                        c.current_frontend_model()
                    },
                });
            }

            ClientMessage::NewConversation => {
                let mut c = conn.lock().await;
                let new_id = uuid::Uuid::new_v4().to_string();
                let title = "New Conversation".to_string();
                let _ = c.memory.upsert_conversation(&new_id, &title, "web", Some(&c.config.model), 0);
                c.conversation_id = new_id.clone();
                c.conversation_title = title.clone();
                c.history.clear();
                let conversations = c.memory.list_conversations(30).unwrap_or_default()
                    .into_iter().map(ConversationPayload::from).collect();
                let _ = tx.send(ServerMessage::ConversationStarted { id: new_id, title });
                let _ = tx.send(ServerMessage::ConversationsList { conversations });
            }

            ClientMessage::LoadConversation { id } => {
                let mut c = conn.lock().await;
                let history = c.memory.load_history_str(&id).unwrap_or_default();
                let meta = c.memory.list_conversations(30).unwrap_or_default()
                    .into_iter().find(|m| m.id == id);
                let title = meta.map(|m| m.title).unwrap_or_else(|| "Conversation".to_string());
                c.conversation_id = id.clone();
                c.conversation_title = title.clone();
                c.history = history.clone();
                let messages = history.iter().map(|m| HistoryMessagePayload {
                    role: m.role.clone(),
                    content: m.content.clone(),
                    model: m.model.clone(),
                }).collect();
                let _ = tx.send(ServerMessage::ConversationStarted { id: id.clone(), title });
                let _ = tx.send(ServerMessage::ConversationHistory { id, messages });
            }

            ClientMessage::ListConversations => {
                let c = conn.lock().await;
                let conversations = c.memory.list_conversations(30).unwrap_or_default()
                    .into_iter().map(ConversationPayload::from).collect();
                let _ = tx.send(ServerMessage::ConversationsList { conversations });
            }
        }
    }
}

// ─── Message handler ──────────────────────────────────────────────────────

async fn handle_user_message(
    user_msg: String,
    conn: Arc<Mutex<ConnectionState>>,
    tx: mpsc::UnboundedSender<ServerMessage>,
) {
    let _ = tx.send(ServerMessage::Thinking);

    let (agent_config, history_snapshot) = {
        let c = conn.lock().await;
        (c.config.clone(), c.history.clone())
    };

    let tx_clone = tx.clone();

    let result: Result<String, String> = {
        let mut c = conn.lock().await;
        let mut response_text = String::new();
        let mut tool_call_count: usize = 0;

        let ConnectionState {
            ref shell_policy,
            ref memory,
            ref mut mcp,
            ref client,
            ..
        } = *c;
        let mem: &dyn argus_core::MemoryBackend = memory;

        let r = argus_core::run_agent_turn(
            &agent_config,
            &user_msg,
            &history_snapshot,
            shell_policy,
            mem,
            mcp,
            client,
            |event| {
                match event {
                    AgentEvent::Thinking => {
                        let _ = tx_clone.send(ServerMessage::Thinking);
                    }
                    AgentEvent::ToolCall { id, name, args, .. } => {
                        tool_call_count += 1;
                        let _ = tx_clone.send(ServerMessage::ToolCall {
                            name,
                            args,
                            call_id: id,
                        });
                    }
                    AgentEvent::ToolResult { id, name, result, success, .. } => {
                        let _ = tx_clone.send(ServerMessage::ToolResult {
                            name,
                            result,
                            success,
                            call_id: id,
                        });
                    }
                    AgentEvent::Response(text) => {
                        response_text = text;
                    }
                    AgentEvent::Error(err) => {
                        let _ = tx_clone.send(ServerMessage::Error { message: err });
                    }
                }
            },
        ).await;

        // Auto-post to intranet after tool-heavy turns (mirrors telegram.rs behaviour).
        if tool_call_count > 2 {
            if let Some(ref emb) = agent_config.embedding {
                let emb = emb.clone();
                let author  = agent_config.model.clone();
                let summary = response_text.clone();
                let context = Some(format!("Web UI turn — {} tool calls", tool_call_count));
                tokio::spawn(async move {
                    let content = if summary.len() > 500 {
                        format!("{}...", summary.chars().take(497).collect::<String>())
                    } else {
                        summary
                    };
                    if let Err(e) = emb.post_finding(&author, &content, context).await {
                        eprintln!("[intranet] web auto-post failed: {}", e);
                    }
                });
            }
        }

        match r {
            Ok(text) => {
                c.history.push(ConversationMessage {
                    role: "user".to_string(),
                    content: user_msg.clone(),
                    model: None,
                });
                c.history.push(ConversationMessage {
                    role: "assistant".to_string(),
                    content: text.clone(),
                    model: Some(agent_config.model.clone()),
                });
                if c.history.len() > 40 {
                    let drain = c.history.len() - 40;
                    c.history.drain(0..drain);
                }
                // Auto-title from first user message.
                if c.conversation_title == "New Conversation" {
                    if let Some(first_msg) = c.history.first() {
                    let first = first_msg.content.chars().take(60).collect::<String>();
                    c.conversation_title = if first.len() == 60 {
                        format!("{}…", first)
                    } else {
                        first
                    };
                    }
                }
                Ok(text)
            }
            Err(e) => {
                if response_text.is_empty() {
                    Err(e)
                } else {
                    Ok(response_text)
                }
            }
        }
    };

    match result {
        Ok(content) => {
            let _ = tx.send(ServerMessage::ResponseComplete { content });
            let (frontend_model, memory_update) = {
                let c = conn.lock().await;
                // Persist history and metadata after every successful turn.
                let _ = c.memory.save_history_str(&c.conversation_id, &c.history);
                let _ = c.memory.upsert_conversation(
                    &c.conversation_id,
                    &c.conversation_title,
                    "web",
                    Some(&c.config.model),
                    c.history.len() / 2,
                );
                (c.current_frontend_model(), build_memory_update(&c.memory))
            };
            let _ = tx.send(memory_update);
            let _ = tx.send(ServerMessage::Status {
                eye_state: "watching".to_string(),
                model: frontend_model,
            });
        }
        Err(err) => {
            let _ = tx.send(ServerMessage::Error { message: err });
            let _ = tx.send(ServerMessage::Status {
                eye_state: "watching".to_string(),
                model: {
                    let c = conn.lock().await;
                    c.current_frontend_model()
                },
            });
        }
    }
}
