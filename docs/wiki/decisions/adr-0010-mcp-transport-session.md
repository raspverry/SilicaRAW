---
title: "ADR 0010: MCP Transport and Session"
status: accepted
audience: all
updated: 2026-06-18
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md
---

# ADR 0010: MCP Transport and Session

## Context

Phase 26 starts MCP work after permission policy, action-log evidence, plugin manifest validation, data-only plugin presets, and plugin permission review exist.

SilicaRAW remains a local-first desktop editor. MCP must not become a shortcut around Core APIs, catalog transactions, or original-file safety.

Current MCP protocol references recognize local stdio transport and HTTP-based transport. The official specification documents stdio and Streamable HTTP, and the official transport guide recommends stdio for local integrations where the client can launch the server process.

Primary references:

- MCP specification: <https://modelcontextprotocol.io/specification>
- MCP transport guide: <https://modelcontextprotocol.io/docs/concepts/transports>

## Decision

SilicaRAW will use a **disabled-by-default, stdio-first MCP design** for the first MCP phase.

The first future MCP server shape is:

```txt
MCP client launches local SilicaRAW MCP process over stdio
-> MCP adapter validates manifest and session request
-> adapter calls silica-core read-only APIs
-> Core appends read evidence where scoped
-> adapter returns read-only tool result
```

Session lifetime:

```txt
One MCP session == one stdio server process lifetime.
No background listener.
No persisted session token.
No persisted grant.
No app-start server.
Session ends when the process exits or the client disconnects.
```

Permission posture:

```txt
Default mode -> Off
First enabled mode -> Read-only only
No mutating MCP tools in Phase 26
No permission self-escalation
No tool can change MCP mode or grants
No raw SQL, direct SQLite, unrestricted filesystem, original mutation, plugin install, or plugin enable tools
```

Data boundary:

```txt
MCP adapter -> Core APIs only
Core -> Storage APIs only
MCP adapter never opens catalog SQLite directly
MCP adapter never reads or writes original photo files directly
```

Streamable HTTP is deferred. A future HTTP MCP transport needs a separate ADR before implementation, including localhost binding rules, Origin validation, authentication/token policy, port lifetime, and cross-origin threat review.

## Consequences

- `crates/silica-mcp` remains runtime-free until a scoped implementation task starts the stdio adapter.
- Task 26.2 should define read-only MCP tool manifests only.
- Task 26.3 may implement read-only adapter calls through Core APIs only.
- Read-only MCP calls may log `mcp_read` evidence through Core where useful.
- Mutating tools listed in older broad specs remain out of scope for Phase 26.
- No user setting, Preferences toggle, or manifest may start MCP during Task 26.1.

## Alternatives Considered

- **Streamable HTTP first:** rejected for the first MCP phase because a listener introduces port, origin, authentication, and lifecycle risk that stdio avoids for local use.
- **Always-on app-hosted server:** rejected because MCP is off by default and should not start at app launch.
- **Direct catalog access from MCP:** rejected because all extension access must go through Core APIs and action trust boundaries.
- **Mutating MCP tools first:** rejected because Phase 26 is read-only first and mutating tools need future ADR approval.

## Links

- [Plugins and MCP](../topics/plugins-and-mcp.md)
- [Post-Alpha Product Roadmap](../roadmaps/post-alpha-product-roadmap.md#phase-26-mcp-read-only-first)
- [MCP Tool Manifest Schema](../../../schemas/mcp_tool_manifest.schema.json)
- [Action Trust](../topics/action-trust.md)
- [Data Safety](../topics/data-safety.md)
