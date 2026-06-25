//! Bridge between argus-core's MissionExecutor trait and argus-missions' MissionRegistry.
//! Avoids circular crate dependency while keeping the dispatch clean.

use argus_core::MissionExecutor;
use crate::tools::{MissionRegistry, execute_mission_tool, mission_tool_schemas};
use argus_core::{AgentConfig, SentryBus};
use argus_core::supabase::SupabaseClient;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;

pub struct MissionBridge {
    pub registry: Arc<MissionRegistry>,
    pub config: AgentConfig,
    pub sentry_bus: Option<Arc<SentryBus>>,
    pub supabase: Option<SupabaseClient>,
    pub http: Client,
    pub discord_token: Option<String>,
    pub ops_channel_id: Option<String>,
}

impl MissionExecutor for MissionBridge {
    fn execute<'a>(
        &'a self,
        name: &'a str,
        args: &'a Value,
        model: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>> {
        Box::pin(async move {
            execute_mission_tool(
                name,
                args,
                &self.config,
                &self.registry,
                self.sentry_bus.as_deref(),
                self.supabase.as_ref(),
                &self.http,
                model,
                self.discord_token.as_deref(),
                self.ops_channel_id.as_deref(),
            ).await
        })
    }

    fn list_missions_json(&self) -> Vec<serde_json::Value> {
        self.registry.list_all().into_iter()
            .map(|m| serde_json::to_value(&m).unwrap_or_default())
            .collect()
    }
}

/// Tool schemas to append to builtin_tool_schemas() in argus-core.
/// Called from main.rs to inject schemas without a circular dep.
pub fn schemas() -> Vec<Value> {
    mission_tool_schemas()
}
