# MCP Registry

[![Crates.io](https://img.shields.io/crates/v/mcp-registry.svg)](https://crates.io/crates/mcp-registry)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Central control plane for MCP server registration, tool discovery, allow-lists, health monitoring, and audit — for [ADK-Rust Enterprise](https://enterprise.adk-rust.com).

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-registry/main/docs/architecture.svg" alt="MCP Registry Architecture" width="800"/>
</p>

## Why

Without a registry, MCP servers are ad hoc per-agent connectors with no visibility, governance, or control. The MCP Registry ensures every server, tool, credential binding, and exposure decision is centrally managed and auditable.

## Tools (10)

| Tool | Purpose | Risk |
|------|---------|------|
| `register_mcp_server` | Add a new MCP server definition | Internal write |
| `update_mcp_server` | Edit command, URL, transport, credentials, or policy labels | Internal write |
| `list_mcp_servers` | Return servers by environment, status, owner, or domain | Read-only |
| `get_mcp_server` | Inspect one server including tools and health | Read-only |
| `discover_mcp_tools` | Pull tool definitions from a server | Read-only |
| `set_tool_allowlist` | Approve which tools agents can use | High write |
| `test_mcp_server` | Run connectivity and schema checks | Read-only |
| `restart_mcp_server` | Restart local or managed server runtime | External write |
| `disable_mcp_server` | Remove server from active routing | Internal write |
| `export_mcp_inventory` | Export full inventory for audit | Read-only |

## Example Workflow

```
> register_mcp_server(name: "github-tools", transport: "stdio", command: "npx -y @modelcontextprotocol/server-github", ...)

{ "server_id": "mcp_github_tools_sta", "version": 1, "status": "registered" }

> test_mcp_server(server_id: "mcp_github_tools_sta")

{ "connectivity": true, "schema_valid": true, "latency_ms": 45, "result": "passed" }

> discover_mcp_tools(server_id: "mcp_github_tools_sta", tools: [...])

{ "snapshot_id": "disc_abc123", "tools_count": 13, "diff": { "added_tools": ["search_repositories", ...] } }

> set_tool_allowlist(server_id: "mcp_github_tools_sta", tool_names: ["search_repositories", "get_file_contents"], ...)

{ "entries": 2, "status": "active" }

> list_mcp_servers()

[{ "id": "mcp_github_tools_sta", "name": "github-tools", "status": "active", "tools_discovered": 13, "tools_allowed": 2 }]

> disable_mcp_server(server_id: "mcp_github_tools_sta")

{ "status": "disabled", "message": "Server removed from active routing. History preserved." }
```

## Key Concepts

- **Discovery ≠ Exposure** — discovering tools does not expose them to agents. Explicit allow-listing is required.
- **Versioned definitions** — every update creates a new immutable version for rollback.
- **Credential indirection** — servers reference vault credentials, never raw secrets.
- **Audit trail** — all mutations emit audit events with actor, action, and timestamp.
- **Environment isolation** — servers are scoped to dev/staging/production.

## Installation

```bash
cargo install mcp-registry
```

Or build from source:

```bash
git clone https://github.com/zavora-ai/mcp-registry
cd mcp-registry
cargo build --release
```

### MCP Client Config

```json
{
  "mcpServers": {
    "registry": {
      "command": "/path/to/mcp-registry"
    }
  }
}
```

## Documentation

| Document | Description |
|----------|-------------|
| [API Reference](docs/api-reference.md) | All 10 tools with parameters and returns |
| [Architecture](docs/architecture.svg) | System diagram |

## Governance

- Production allow-list changes require policy simulation
- Write-capable tool exposure requires Platform Admin approval
- All mutations are audit-logged
- Credential bindings reference vault entries only

## Implementation Phases

| Phase | Scope | Status |
|-------|-------|--------|
| 1 | Core registry, discovery, allow-lists, health, audit | ✅ Complete |
| 2 | Credential binding, policy labels, approval workflow | Planned |
| 3 | Resources, prompts, schema normalization, diffs | Planned |
| 4 | Versioning, rollback, inventory exports | Planned |
| 5 | Runtime routing proxy, per-call policy, dashboards | Planned |

## License

Apache-2.0

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **HealthCheck** — async health probe for registry monitoring
- **mcp-server.toml** — manifest declaring tools, risk classes, and credentials
- **Structured tracing** — `RUST_LOG` env-filter for observability

