# API Reference

## register_mcp_server

Add a new MCP server definition to the registry.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | Yes | Server display name |
| `description` | string | Yes | What this server does |
| `owner` | string | Yes | Team or user responsible |
| `domain` | string | Yes | Business domain (e.g. "developer", "platform") |
| `environment` | string | Yes | Target environment (development/staging/production) |
| `transport` | enum | Yes | `stdio`, `sse`, `streamable_http`, `managed`, `custom` |
| `command` | string | No | Command for stdio transport |
| `url` | string | No | URL for HTTP/SSE transport |
| `credential_bindings` | string[] | No | Vault credential references |
| `policy_labels` | string[] | No | Governance labels |
| `tags` | string[] | No | Categorization tags |

**Returns:**
```json
{ "server_id": "mcp_github_tools_sta", "version": 1, "status": "registered", "discovery_status": "not_started", "allowlist_status": "empty" }
```

---

## update_mcp_server

Edit server configuration. Creates a new immutable version.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `server_id` | string | Yes | Server to update |
| `command` | string | No | New command |
| `url` | string | No | New URL |
| `transport` | enum | No | New transport |
| `environment` | string | No | New environment |
| `owner` | string | No | New owner |
| `policy_labels` | string[] | No | New policy labels |
| `credential_bindings` | string[] | No | New credential bindings |

**Returns:**
```json
{ "server_id": "mcp_github_tools_sta", "version": 2, "status": "updated" }
```

---

## list_mcp_servers

Return registered servers with optional filters.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `environment` | string | No | Filter by environment |
| `status` | string | No | Filter by status |
| `domain` | string | No | Filter by domain |
| `owner` | string | No | Filter by owner |

**Returns:** Array of server summaries with id, name, environment, status, transport, tools_discovered, tools_allowed, health.

---

## get_mcp_server

Inspect one server including tools, health, credentials, and policy.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `server_id` | string | Yes | Server to inspect |

**Returns:** Full server record with tools list, health status, credential bindings, policy labels.

---

## discover_mcp_tools

Pull tool definitions from a server and create a discovery snapshot.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `server_id` | string | Yes | Server to discover |
| `tools` | array | Yes | Tool definitions (name, description, risk_class, scope, side_effect) |

**Returns:**
```json
{ "snapshot_id": "disc_abc123", "tools_count": 13, "diff": { "added_tools": [...], "removed_tools": [...] }, "status": "completed" }
```

---

## set_tool_allowlist

Approve which discovered tools agents can use.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `server_id` | string | Yes | Server containing the tools |
| `tool_names` | string[] | Yes | Tools to allow |
| `environment` | string | Yes | Environment scope |
| `agents` | string[] | No | Agent scope (empty = all) |
| `reason` | string | Yes | Justification for allow-listing |

**Returns:**
```json
{ "entries": 2, "status": "active", "allowlist": [{ "id": "allow_...", "tool": "search_repositories", "environment": "staging" }] }
```

**Error:** `"Tool 'x' not discovered on server 'y'"` — run `discover_mcp_tools` first.

---

## test_mcp_server

Run connectivity and schema checks.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `server_id` | string | Yes | Server to test |

**Returns:**
```json
{ "server_id": "...", "connectivity": true, "tools_listed": true, "schema_valid": true, "latency_ms": 45, "errors": [], "result": "passed" }
```

---

## restart_mcp_server

Restart a local or managed server runtime.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `server_id` | string | Yes | Server to restart |

**Returns:**
```json
{ "server_id": "...", "status": "restarted", "health": "healthy" }
```

---

## disable_mcp_server

Remove server from active agent routing without deleting history.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `server_id` | string | Yes | Server to disable |

**Returns:**
```json
{ "server_id": "...", "status": "disabled", "message": "Server removed from active routing. History preserved." }
```

---

## export_mcp_inventory

Export full inventory for audit and compliance.

**Parameters:** None.

**Returns:**
```json
{ "servers": 12, "servers_list": [...], "discovery_snapshots": 24, "allowlist_entries": 45, "audit_events": 180, "exported_at": "2026-05-23T..." }
```
