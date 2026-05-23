use chrono::{DateTime, Utc};
use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    Stdio,
    Sse,
    StreamableHttp,
    Managed,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    Active,
    Disabled,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthState {
    Healthy,
    Degraded,
    Failing,
    Disabled,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub domain: String,
    pub environment: String,
    pub transport: Transport,
    pub command: Option<String>,
    pub url: Option<String>,
    pub status: ServerStatus,
    pub version: u32,
    pub credential_bindings: Vec<String>,
    pub policy_labels: Vec<String>,
    pub tags: Vec<String>,
    pub tools: Vec<ToolRecord>,
    pub allowlist: Vec<String>,
    pub health: HealthStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRecord {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,
    pub risk_class: RiskLevel,
    pub scope: String,
    pub side_effect: Option<String>,
    pub allowed: bool,
    pub discovered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AllowlistEntry {
    pub id: String,
    pub server_id: String,
    pub tool_name: String,
    pub environment: String,
    pub agents: Vec<String>,
    pub risk: RiskLevel,
    pub policy_labels: Vec<String>,
    pub status: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySnapshot {
    pub id: String,
    pub server_id: String,
    pub server_version: u32,
    pub tools_count: usize,
    pub tools: Vec<ToolRecord>,
    pub diff: Option<DiscoveryDiff>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryDiff {
    pub added_tools: Vec<String>,
    pub removed_tools: Vec<String>,
    pub changed_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub state: HealthState,
    pub checked_at: Option<DateTime<Utc>>,
    pub latency_ms: Option<u64>,
    pub error_rate: Option<f64>,
    pub message: Option<String>,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            state: HealthState::Unknown,
            checked_at: None,
            latency_ms: None,
            error_rate: None,
            message: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub server_id: String,
    pub action: String,
    pub actor: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub server_id: String,
    pub connectivity: bool,
    pub tools_listed: bool,
    pub schema_valid: bool,
    pub latency_ms: u64,
    pub errors: Vec<String>,
    pub tested_at: DateTime<Utc>,
}
