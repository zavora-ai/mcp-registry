use crate::store::RegistryStore;
use crate::types::*;
use chrono::Utc;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;

// --- Input types ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegisterMcpServerInput {
    pub name: String,
    pub description: String,
    pub owner: String,
    pub domain: String,
    pub environment: String,
    pub transport: Transport,
    pub command: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub credential_bindings: Vec<String>,
    #[serde(default)]
    pub policy_labels: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateMcpServerInput {
    pub server_id: String,
    pub command: Option<String>,
    pub url: Option<String>,
    pub transport: Option<Transport>,
    pub environment: Option<String>,
    pub owner: Option<String>,
    pub policy_labels: Option<Vec<String>>,
    pub credential_bindings: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListMcpServersInput {
    pub environment: Option<String>,
    pub status: Option<String>,
    pub domain: Option<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetMcpServerInput {
    pub server_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DiscoverMcpToolsInput {
    pub server_id: String,
    /// Simulated tool definitions for discovery (in production, pulled from live server)
    #[serde(default)]
    pub tools: Vec<ToolInput>,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct ToolInput {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub risk_class: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub side_effect: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SetToolAllowlistInput {
    pub server_id: String,
    pub tool_names: Vec<String>,
    pub environment: String,
    #[serde(default)]
    pub agents: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TestMcpServerInput {
    pub server_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RestartMcpServerInput {
    pub server_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DisableMcpServerInput {
    pub server_id: String,
}

// --- Server ---

#[derive(Clone)]
pub struct RegistryServer {
    pub store: std::sync::Arc<RegistryStore>,
}

#[tool_router(server_handler)]
impl RegistryServer {
    #[tool(description = "Add a new MCP server definition")]
    fn register_mcp_server(&self, Parameters(i): Parameters<RegisterMcpServerInput>) -> String {
        let record = self.store.register_server(
            i.name, i.description, i.owner, i.domain, i.environment,
            i.transport, i.command, i.url, i.credential_bindings, i.policy_labels, i.tags,
        );
        serde_json::to_string_pretty(&serde_json::json!({
            "server_id": record.id,
            "version": record.version,
            "status": "registered",
            "discovery_status": "not_started",
            "allowlist_status": "empty",
        })).unwrap()
    }

    #[tool(description = "Edit command, URL, transport, credentials, or policy labels")]
    fn update_mcp_server(&self, Parameters(i): Parameters<UpdateMcpServerInput>) -> String {
        match self.store.update_server(&i.server_id, i.command, i.url, i.transport, i.environment, i.owner, i.policy_labels, i.credential_bindings) {
            Ok(r) => serde_json::to_string_pretty(&serde_json::json!({
                "server_id": r.id, "version": r.version, "status": "updated"
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Return registered servers by environment, status, owner, or domain")]
    fn list_mcp_servers(&self, Parameters(i): Parameters<ListMcpServersInput>) -> String {
        let servers = self.store.list_servers(
            i.environment.as_deref(), i.status.as_deref(), i.domain.as_deref(), i.owner.as_deref(),
        );
        let summary: Vec<serde_json::Value> = servers.iter().map(|s| serde_json::json!({
            "id": s.id, "name": s.name, "environment": s.environment,
            "status": s.status, "transport": s.transport,
            "tools_discovered": s.tools.len(),
            "tools_allowed": s.allowlist.len(),
            "health": s.health.state,
        })).collect();
        serde_json::to_string_pretty(&summary).unwrap()
    }

    #[tool(description = "Inspect one server, including exposed tools and health")]
    fn get_mcp_server(&self, Parameters(i): Parameters<GetMcpServerInput>) -> String {
        match self.store.get_server(&i.server_id) {
            Some(s) => serde_json::to_string_pretty(&serde_json::json!({
                "id": s.id, "name": s.name, "description": s.description,
                "owner": s.owner, "domain": s.domain, "environment": s.environment,
                "transport": s.transport, "command": s.command, "url": s.url,
                "status": s.status, "version": s.version,
                "credential_bindings": s.credential_bindings,
                "policy_labels": s.policy_labels,
                "tools_discovered": s.tools.len(),
                "tools_allowed": s.allowlist.len(),
                "tools": s.tools.iter().map(|t| serde_json::json!({
                    "name": t.name, "description": t.description,
                    "risk": t.risk_class, "allowed": t.allowed,
                })).collect::<Vec<_>>(),
                "health": { "state": s.health.state, "checked_at": s.health.checked_at, "latency_ms": s.health.latency_ms },
                "created_at": s.created_at, "updated_at": s.updated_at,
            })).unwrap(),
            None => format!("Server not found: {}", i.server_id),
        }
    }

    #[tool(description = "Pull tool definitions from a server")]
    fn discover_mcp_tools(&self, Parameters(i): Parameters<DiscoverMcpToolsInput>) -> String {
        let tools: Vec<ToolRecord> = i.tools.into_iter().map(|t| ToolRecord {
            name: t.name,
            description: t.description,
            parameters_schema: serde_json::json!({}),
            risk_class: match t.risk_class.as_deref() {
                Some("high") => RiskLevel::High,
                Some("critical") => RiskLevel::Critical,
                Some("low") => RiskLevel::Low,
                _ => RiskLevel::Medium,
            },
            scope: t.scope.unwrap_or_else(|| "read".to_string()),
            side_effect: t.side_effect,
            allowed: false,
            discovered_at: Utc::now(),
        }).collect();

        match self.store.discover_tools(&i.server_id, tools) {
            Ok(snap) => serde_json::to_string_pretty(&serde_json::json!({
                "snapshot_id": snap.id,
                "tools_count": snap.tools_count,
                "diff": snap.diff,
                "status": snap.status,
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Approve which discovered tools agents can use")]
    fn set_tool_allowlist(&self, Parameters(i): Parameters<SetToolAllowlistInput>) -> String {
        match self.store.set_allowlist(&i.server_id, i.tool_names, &i.environment, i.agents, &i.reason) {
            Ok(entries) => serde_json::to_string_pretty(&serde_json::json!({
                "entries": entries.len(),
                "status": "active",
                "allowlist": entries.iter().map(|e| serde_json::json!({
                    "id": e.id, "tool": e.tool_name, "environment": e.environment,
                })).collect::<Vec<_>>(),
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Run connectivity and schema checks")]
    fn test_mcp_server(&self, Parameters(i): Parameters<TestMcpServerInput>) -> String {
        match self.store.test_server(&i.server_id) {
            Ok(r) => serde_json::to_string_pretty(&serde_json::json!({
                "server_id": r.server_id,
                "connectivity": r.connectivity,
                "tools_listed": r.tools_listed,
                "schema_valid": r.schema_valid,
                "latency_ms": r.latency_ms,
                "errors": r.errors,
                "result": if r.connectivity && r.schema_valid { "passed" } else { "failed" },
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Restart local or managed server runtime")]
    fn restart_mcp_server(&self, Parameters(i): Parameters<RestartMcpServerInput>) -> String {
        match self.store.restart_server(&i.server_id) {
            Ok(r) => serde_json::to_string_pretty(&serde_json::json!({
                "server_id": r.id, "status": "restarted", "health": r.health.state,
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Remove server from active agent routing")]
    fn disable_mcp_server(&self, Parameters(i): Parameters<DisableMcpServerInput>) -> String {
        match self.store.disable_server(&i.server_id) {
            Ok(r) => serde_json::to_string_pretty(&serde_json::json!({
                "server_id": r.id, "status": "disabled",
                "message": "Server removed from active routing. History preserved.",
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Export server, tool, credential, and policy inventory")]
    fn export_mcp_inventory(&self) -> String {
        serde_json::to_string_pretty(&self.store.export_inventory()).unwrap()
    }
}
