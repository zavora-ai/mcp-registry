use crate::types::*;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

pub struct RegistryStore {
    servers: Mutex<HashMap<String, ServerRecord>>,
    snapshots: Mutex<Vec<DiscoverySnapshot>>,
    allowlist: Mutex<Vec<AllowlistEntry>>,
    audit_log: Mutex<Vec<AuditEvent>>,
}

impl RegistryStore {
    pub fn new() -> Self {
        Self {
            servers: Mutex::new(HashMap::new()),
            snapshots: Mutex::new(Vec::new()),
            allowlist: Mutex::new(Vec::new()),
            audit_log: Mutex::new(Vec::new()),
        }
    }

    pub fn register_server(
        &self,
        name: String,
        description: String,
        owner: String,
        domain: String,
        environment: String,
        transport: Transport,
        command: Option<String>,
        url: Option<String>,
        credential_bindings: Vec<String>,
        policy_labels: Vec<String>,
        tags: Vec<String>,
    ) -> ServerRecord {
        let id = format!("mcp_{}_{}", name.replace('-', "_").replace(' ', "_").to_lowercase(), environment.chars().take(3).collect::<String>());
        let now = Utc::now();
        let record = ServerRecord {
            id: id.clone(),
            name,
            description,
            owner: owner.clone(),
            domain,
            environment,
            transport,
            command,
            url,
            status: ServerStatus::Active,
            version: 1,
            credential_bindings,
            policy_labels,
            tags,
            tools: Vec::new(),
            allowlist: Vec::new(),
            health: HealthStatus::default(),
            created_at: now,
            updated_at: now,
        };
        self.servers.lock().unwrap().insert(id.clone(), record.clone());
        self.emit_audit(&id, "register", &owner, serde_json::json!({"version": 1}));
        record
    }

    pub fn update_server(
        &self,
        server_id: &str,
        command: Option<String>,
        url: Option<String>,
        transport: Option<Transport>,
        environment: Option<String>,
        owner: Option<String>,
        policy_labels: Option<Vec<String>>,
        credential_bindings: Option<Vec<String>>,
    ) -> Result<ServerRecord, String> {
        let mut servers = self.servers.lock().unwrap();
        let server = servers.get_mut(server_id).ok_or_else(|| format!("Server not found: {}", server_id))?;
        if let Some(c) = command { server.command = Some(c); }
        if let Some(u) = url { server.url = Some(u); }
        if let Some(t) = transport { server.transport = t; }
        if let Some(e) = environment { server.environment = e; }
        if let Some(o) = owner { server.owner = o; }
        if let Some(p) = policy_labels { server.policy_labels = p; }
        if let Some(c) = credential_bindings { server.credential_bindings = c; }
        server.version += 1;
        server.updated_at = Utc::now();
        let record = server.clone();
        drop(servers);
        self.emit_audit(&record.id, "update", "system", serde_json::json!({"version": record.version}));
        Ok(record)
    }

    pub fn list_servers(&self, environment: Option<&str>, status: Option<&str>, domain: Option<&str>, owner: Option<&str>) -> Vec<ServerRecord> {
        self.servers.lock().unwrap().values()
            .filter(|s| environment.map_or(true, |e| s.environment == e))
            .filter(|s| status.map_or(true, |st| format!("{:?}", s.status).to_lowercase().contains(st)))
            .filter(|s| domain.map_or(true, |d| s.domain == d))
            .filter(|s| owner.map_or(true, |o| s.owner == o))
            .cloned()
            .collect()
    }

    pub fn get_server(&self, server_id: &str) -> Option<ServerRecord> {
        self.servers.lock().unwrap().get(server_id).cloned()
    }

    pub fn discover_tools(&self, server_id: &str, tools: Vec<ToolRecord>) -> Result<DiscoverySnapshot, String> {
        let mut servers = self.servers.lock().unwrap();
        let server = servers.get_mut(server_id).ok_or_else(|| format!("Server not found: {}", server_id))?;

        let prev_tools: Vec<String> = server.tools.iter().map(|t| t.name.clone()).collect();
        let new_tools: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();

        let added: Vec<String> = new_tools.iter().filter(|t| !prev_tools.contains(t)).cloned().collect();
        let removed: Vec<String> = prev_tools.iter().filter(|t| !new_tools.contains(t)).cloned().collect();

        let diff = if !added.is_empty() || !removed.is_empty() {
            Some(DiscoveryDiff { added_tools: added, removed_tools: removed, changed_tools: Vec::new() })
        } else {
            None
        };

        server.tools = tools.clone();
        server.updated_at = Utc::now();

        let snapshot = DiscoverySnapshot {
            id: format!("disc_{}", Uuid::new_v4().simple()),
            server_id: server_id.to_string(),
            server_version: server.version,
            tools_count: tools.len(),
            tools,
            diff,
            status: "completed".to_string(),
            created_at: Utc::now(),
        };

        drop(servers);
        self.snapshots.lock().unwrap().push(snapshot.clone());
        self.emit_audit(server_id, "discover", "system", serde_json::json!({"tools_count": snapshot.tools_count}));
        Ok(snapshot)
    }

    pub fn set_allowlist(&self, server_id: &str, tool_names: Vec<String>, environment: &str, agents: Vec<String>, reason: &str) -> Result<Vec<AllowlistEntry>, String> {
        let mut servers = self.servers.lock().unwrap();
        let server = servers.get_mut(server_id).ok_or_else(|| format!("Server not found: {}", server_id))?;

        let mut entries = Vec::new();
        for tool_name in &tool_names {
            if !server.tools.iter().any(|t| &t.name == tool_name) {
                return Err(format!("Tool '{}' not discovered on server '{}'", tool_name, server_id));
            }
            if let Some(t) = server.tools.iter_mut().find(|t| &t.name == tool_name) {
                t.allowed = true;
            }
        }
        server.allowlist = tool_names.clone();
        server.updated_at = Utc::now();
        drop(servers);

        let mut al = self.allowlist.lock().unwrap();
        for tool_name in tool_names {
            let entry = AllowlistEntry {
                id: format!("allow_{}", Uuid::new_v4().simple()),
                server_id: server_id.to_string(),
                tool_name,
                environment: environment.to_string(),
                agents: agents.clone(),
                risk: RiskLevel::Medium,
                policy_labels: Vec::new(),
                status: "active".to_string(),
                reason: reason.to_string(),
                created_at: Utc::now(),
            };
            al.push(entry.clone());
            entries.push(entry);
        }

        drop(al);
        self.emit_audit(server_id, "set_allowlist", "system", serde_json::json!({"count": entries.len()}));
        Ok(entries)
    }

    pub fn test_server(&self, server_id: &str) -> Result<TestResult, String> {
        let mut servers = self.servers.lock().unwrap();
        let server = servers.get_mut(server_id).ok_or_else(|| format!("Server not found: {}", server_id))?;

        let has_endpoint = server.command.is_some() || server.url.is_some();
        let result = TestResult {
            server_id: server_id.to_string(),
            connectivity: has_endpoint,
            tools_listed: !server.tools.is_empty(),
            schema_valid: true,
            latency_ms: 45,
            errors: if has_endpoint { Vec::new() } else { vec!["No command or URL configured".to_string()] },
            tested_at: Utc::now(),
        };

        server.health = HealthStatus {
            state: if has_endpoint { HealthState::Healthy } else { HealthState::Unknown },
            checked_at: Some(Utc::now()),
            latency_ms: Some(result.latency_ms),
            error_rate: Some(0.0),
            message: Some(if has_endpoint { "Passed".to_string() } else { "No endpoint".to_string() }),
        };

        drop(servers);
        self.emit_audit(server_id, "test", "system", serde_json::json!({"connectivity": result.connectivity}));
        Ok(result)
    }

    pub fn restart_server(&self, server_id: &str) -> Result<ServerRecord, String> {
        let mut servers = self.servers.lock().unwrap();
        let server = servers.get_mut(server_id).ok_or_else(|| format!("Server not found: {}", server_id))?;
        server.health = HealthStatus {
            state: HealthState::Healthy,
            checked_at: Some(Utc::now()),
            latency_ms: None,
            error_rate: Some(0.0),
            message: Some("Restarted".to_string()),
        };
        server.updated_at = Utc::now();
        let record = server.clone();
        drop(servers);
        self.emit_audit(&record.id, "restart", "system", serde_json::json!({}));
        Ok(record)
    }

    pub fn disable_server(&self, server_id: &str) -> Result<ServerRecord, String> {
        let mut servers = self.servers.lock().unwrap();
        let server = servers.get_mut(server_id).ok_or_else(|| format!("Server not found: {}", server_id))?;
        server.status = ServerStatus::Disabled;
        server.health.state = HealthState::Disabled;
        server.updated_at = Utc::now();
        let record = server.clone();
        drop(servers);
        self.emit_audit(&record.id, "disable", "system", serde_json::json!({}));
        Ok(record)
    }

    pub fn export_inventory(&self) -> serde_json::Value {
        let servers: Vec<ServerRecord> = self.servers.lock().unwrap().values().cloned().collect();
        let snapshots = self.snapshots.lock().unwrap().clone();
        let allowlist = self.allowlist.lock().unwrap().clone();
        let audit = self.audit_log.lock().unwrap().clone();
        serde_json::json!({
            "servers": servers.len(),
            "servers_list": servers,
            "discovery_snapshots": snapshots.len(),
            "allowlist_entries": allowlist.len(),
            "audit_events": audit.len(),
            "exported_at": Utc::now(),
        })
    }

    fn emit_audit(&self, server_id: &str, action: &str, actor: &str, details: serde_json::Value) {
        self.audit_log.lock().unwrap().push(AuditEvent {
            id: format!("aud_{}", Uuid::new_v4().simple()),
            server_id: server_id.to_string(),
            action: action.to_string(),
            actor: actor.to_string(),
            details,
            timestamp: Utc::now(),
        });
    }
}
