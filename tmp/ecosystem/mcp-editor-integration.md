# MCP and Editor Integration: Practical Technical Reference

This document is an IronClaw integration guide for MCP, ACP-style editor
facades, and tool exposure. It intentionally avoids repeating full protocol
specifications; verify the versioned upstream protocol text before changing wire
compatibility.

**Protocol boundary**: MCP is the JSON-RPC tool/resource/prompt protocol between
AI clients and tool servers. It is not the REST/SSE/WebSocket operator API; see
[control-plane.md](./control-plane.md) for that surface.

**Tool discovery boundary**: MCP `tools/list` discovers remote server tools over
configured transports. Local TOML/WASM discovery is covered in
[plugin-extension.md](./plugin-extension.md). Both may register into
`ToolRegistry`, but through different lifecycle and security paths.

---

## Table of Contents

1. [What Are MCP and ACP, and Why Both Exist](#1-what-are-mcp-and-acp-and-why-both-exist)
2. [Architecture Overview](#2-architecture-overview)
3. [The MCP JSON-RPC 2.0 Protocol](#3-the-mcp-json-rpc-20-protocol)
4. [MCP Wire Types: Complete Reference](#4-mcp-wire-types-complete-reference)
5. [Session Management](#5-session-management)
6. [Transport Layer: Three Implementations](#6-transport-layer-three-implementations)
7. [OAuth 2.1 Authentication](#7-oauth-21-authentication)
8. [The McpClientStore: Multi-Tenant Safety](#8-the-mcpclientstore-multi-tenant-safety)
9. [MCP 2025-11-25 Compatibility Notes](#9-mcp-2025-11-25-compatibility-notes)
10. [MCP 2026-Era Proposals: Stateless Core and Extensions](#10-mcp-2026-era-proposals-stateless-core-and-extensions)
11. [The ACP Protocol: Agent-to-Editor Communication (Zed ACP)](#11-the-acp-protocol-agent-to-editor-communication-zed-acp)
12. [Captured Workflow ACP: Multi-Agent Coordination Layer](#12-captured-workflow-acp-multi-agent-coordination-layer)
13. [ACP Workflow Pipeline State Machine](#13-acp-workflow-pipeline-state-machine)
14. [ACP Streaming Session Updates](#14-acp-streaming-session-updates)
15. [ACP Permission-Based Action Gates](#15-acp-permission-based-action-gates)
16. [Captured MCP Component Set](#16-captured-mcp-component-set)
17. [Builtin Tool System](#17-builtin-tool-system)
18. [Editor Integration: VS Code, Zed, and JetBrains](#18-editor-integration-vs-code-zed-and-jetbrains)
19. [Practical Editor Integration Workflows](#19-practical-editor-integration-workflows)
20. [Exposing IronClaw Tools as an MCP Server](#20-exposing-ironclaw-tools-as-an-mcp-server)
21. [Benchmarking: Protocol Overhead Measurement](#21-benchmarking-protocol-overhead-measurement)
22. [Candidate Implementation Plan for IronClaw](#22-candidate-implementation-plan-for-ironclaw)
23. [ACP vs MCP Comparison](#23-acp-vs-mcp-comparison)
24. [Specification References](#24-specification-references)
25. [Related Documents](#25-related-documents)

---

## 1. What Are MCP and ACP, and Why Both Exist

MCP solves tool discovery and invocation. ACP-style editor protocols solve
editor-agent sessions, streaming UI updates, and human gates. Keep those roles
separate even when they share JSON-RPC, stdio, HTTP, or SSE mechanics.

### The Model Context Protocol (MCP)

IronClaw's MCP client code lives under `src/tools/mcp/`. Treat
`src/tools/mcp/protocol.rs` as the local source of truth for implemented wire
types and protocol version constants. Do not copy version values into roadmap
docs; inspect the code and versioned spec when implementing.

MCP servers are external systems. For security review, assume they can return
malformed schemas, oversized outputs, prompt-injection text, and tool names that
collide with other integrations.

### The Agent Client Protocol (ACP)

Use ACP-style integration as an editor facade over IronClaw's existing
submission, session, approval, and streaming paths. Do not create an editor-only
agent loop. Editor sessions may present state differently, but planning,
execution, tools, approvals, checkpoints, retries, and completion must go
through the existing Reborn runner/driver/executor path.

---

## 2. Architecture Overview

### IronClaw's Current MCP Architecture

Key implementation points:

- `src/tools/mcp/protocol.rs`: local wire models.
- `src/tools/mcp/client.rs`: request flow and tool calls.
- `src/tools/mcp/transport.rs`: transport abstraction.
- `src/tools/mcp/http_transport.rs`, `stdio_transport.rs`,
  `unix_transport.rs`: concrete transports.
- `src/tools/mcp/session.rs`: session state.
- `src/tools/mcp/client_store.rs`: per-user client registry.
- `src/tools/mcp/process.rs`: stdio process lifecycle.
- `src/tools/registry.rs`: registration target for wrapped MCP tools.

### Multi-User Session Isolation

Hard invariant: key MCP clients and sessions by `(user_id, server_name)`, never
by server name alone. The transport, session ID, OAuth state, child process, and
cached tool surface must not leak across users.

---

## 3. The MCP JSON-RPC 2.0 Protocol

Do not duplicate JSON-RPC handling outside `src/tools/mcp/`. New compatibility
work should extend the protocol types and transport abstraction, then test
through `McpClient` or the factory that production code calls.

### JSON-RPC Framing Rules

IronClaw needs these invariants:

- Requests have an `id`; notifications do not.
- Responses must correlate to a pending request ID.
- Unknown notification types are ignored or logged without breaking the session.
- Error frames preserve protocol code and sanitized message.
- Body, output, and schema sizes are bounded before reaching the LLM.

### MCP Handshake Sequence

Required flow:

1. Initialize with client capabilities.
2. Validate server protocol/capabilities against the configured compatibility
   policy.
3. Send initialized notification without an ID.
4. Discover tools.
5. Register wrappers with approval and safety metadata.

### Standard Error Codes

Keep protocol error mapping close to `McpClient`. Do not let transport-level
errors, auth failures, and server JSON-RPC errors collapse into one opaque
failure; operators need to know which layer broke.

---

## 4. MCP Wire Types: Complete Reference

This section is an implementation checklist, not a full schema dump.

### McpRequest

Add request constructors for new methods instead of building JSON ad hoc in
callers. Constructors should enforce notification/request ID rules.

### McpResponse

Responses should parse into a typed success or typed error. Preserve raw data
only behind debug logging and redaction controls.

### McpError

Expose sanitized code/message to callers. Keep raw server payloads out of chat
history unless explicitly redacted.

### McpTool

Normalize tool names before registration and keep the original server/tool name
for audit. Tool descriptions are untrusted model-facing text.

### McpToolAnnotations

Annotations are hints, not policy. Host policy decides approval, network access,
and whether a tool is exposed to a specific user/session.

### McpTool::requires_approval()

Approval should be conservative. Treat mutating, external, unknown, or
ambiguous tools as requiring approval unless host policy says otherwise.

### ExecutionTimeHint

Use hints for timeouts and UI expectations only. Do not let server-declared
latency override host maximums.

### InitializeResult

Validate server identity, protocol version, capabilities, and optional
instructions. Server instructions are untrusted content.

### ServerCapabilities

Add capability handling incrementally. Unsupported capabilities should not fail
tool-only integrations unless the configured compatibility policy requires it.

### ListToolsResult and CallToolResult

`tools/list` affects registry state; `tools/call` affects execution output.
Test both through the production wrapper so registry collision, approval, and
sanitization behavior are covered.

### ContentBlock

Sanitize all text blocks and bound binary/resource blocks. Treat returned
content as untrusted tool output, not assistant-authored text.

---

## 5. Session Management

### McpSession

Session IDs are bearer-like material. Store them per user/server, never log
them, and validate header values before reuse.

### McpSessionManager

Session manager responsibilities:

- Create, look up, and invalidate sessions by `(user_id, server_name)`.
- Drop stale sessions on auth failure or server reset.
- Avoid holding locks across network calls.

### Session Lifecycle

Lifecycle events:

```text
configured -> initializing -> ready -> degraded -> expired/removed
```

Gateway status may display lifecycle state, but it must not expose session IDs
or tokens.

### Session ID Security Validation

Reject control characters, oversized values, and values that cannot safely be
used as HTTP header content.

### McpProcessManager: Stdio Process Lifecycle

Stdio MCP servers are child processes with host access. The process manager
must control command allowlisting, environment injection, shutdown, restart, and
per-user isolation. Avoid shell invocation; use argv arrays.

---

## 6. Transport Layer: Three Implementations

Keep transport-specific details behind `McpTransport`. The client should not
branch on HTTP vs stdio vs Unix socket except during factory construction.

### HTTP Transport (Streamable HTTP)

Security requirements:

- Operator-configured URL only.
- HTTPS or explicit local-dev exception.
- Auth headers redacted in logs.
- Body and stream limits.
- Redirect policy reviewed before enabling.

### Stdio Transport

Security requirements:

- No shell expansion.
- Explicit argv and working directory.
- Per-user env isolation.
- Kill process on deactivation or shutdown.
- Bound stdout/stderr and parse NDJSON defensively.

### Unix Socket Transport

Security requirements:

- Validate socket path ownership and location.
- Apply the same framing limits as stdio.
- Treat local socket servers as external processes, not trusted in-process code.

### Transport Comparison

| Transport | Best use | Main risk |
|-----------|----------|-----------|
| HTTP | Hosted or managed server | auth/session leakage, untrusted network output |
| stdio | Local MCP package | process escape, env leakage |
| Unix socket | Local daemon | file permissions, stale sockets |

---

## 7. OAuth 2.1 Authentication

OAuth belongs in MCP auth/config modules and the secrets store, not in feature
handlers or tool wrappers.

### OAuth Flow

Keep browser callbacks, token exchange, refresh, and storage behind auth
helpers. Expose only lifecycle state and setup prompts to the web gateway.

### OAuthConfig

Config must identify authorization endpoint, token endpoint, client metadata,
scopes, redirect policy, and PKCE behavior. Do not hardcode provider-specific
URLs in generic runtime code.

### Secret Key Naming Convention

Secret keys must be namespaced by user, server, and credential purpose. Display
names are not secret identifiers.

### Concurrent Refresh Safety

Refresh must be single-flight per user/server credential. Multiple tool calls
should await the same refresh rather than racing and overwriting tokens.

### Factory Transport Decision Tree

Transport selection should be deterministic from validated config:

```text
stdio command -> Stdio transport
unix socket   -> Unix transport
http url      -> HTTP transport
```

Ambiguous or mixed config should fail validation.

---

## 8. The McpClientStore: Multi-Tenant Safety

`McpClientStore` should remain the shared lookup point for lifecycle code and
tool wrappers. Wrappers should carry `(server_name, store)` rather than a
captured user-specific client that can go stale or leak across users.

### Surface Conflict Detection

When a server's tool surface changes, compare the normalized surface signature.
Conflicting changes should mark the server degraded or require re-registration
rather than silently changing tool behavior under an existing name.

---

## 9. MCP 2025-11-25 Compatibility Notes

Treat this section as a compatibility checklist for a named spec revision. Check
the spec and IronClaw's protocol constants before implementation.

### Client Features (Server-Initiated)

Server-initiated requests are compatibility-sensitive. Add only when IronClaw
has a caller and a clear policy for approval, auth, and UI display.

### Server Features IronClaw Implements

Keep feature status in code/tests or a dedicated parity file, not prose counts.
This document should describe integration constraints, not claim coverage.

### Caching Headers (2025-11-25)

If supported, cache metadata must be keyed by user/server/session and invalidated
on auth or surface changes.

### W3C Trace Context (2025-11-25)

Trace propagation must not leak user IDs, secrets, prompts, or session IDs. Use
opaque request IDs.

### JSON Schema 2020-12 (2025-11-25)

Schema support affects tool validation and LLM tool presentation. Add tests for
unknown keywords, refs, defaults, and overly broad schemas before enabling.

---

## 10. MCP 2026-Era Proposals: Stateless Core and Extensions

This heading preserves the existing anchor. Treat all items here as future or
compatibility-only until a concrete versioned spec and implementation plan are
chosen.

### Stateless Protocol Core

Stateless mode changes session storage and auth assumptions. Do not mix stateful
and stateless behavior behind one cache key.

### Extensions Framework

Namespace extension capabilities and reject unknown required extensions. Optional
extensions may be ignored with diagnostics.

### Tasks Extension (`io.modelcontextprotocol.tasks`)

Tasks overlap with IronClaw jobs/routines. If adopted, bridge to existing job
and routine state rather than adding a separate task runner.

### MCP Apps Extension (`io.modelcontextprotocol.apps`)

Apps are UI surfaces. Expose them only through gateway-reviewed routes with CSP,
auth, origin checks, and output scrubbing.

### Authorization Hardening

Prefer tightening existing token/session handling before adding new protocol
surface. Compatibility should not weaken bearer auth, CORS, origin checks, body
limits, or secret handling.

### Features to Treat as Compatibility-Only

Compatibility-only means parse or ignore safely. It does not mean advertise
support to users, tools, or editors.

---

## 11. The ACP Protocol: Agent-to-Editor Communication (Zed ACP)

Use editor protocols to present IronClaw, not to fork it.

### Architecture

```text
editor client
  -> ACP/editor bridge
  -> IronClaw submission/session/gate APIs
  -> existing agentic loop
```

### ACP Session Lifecycle

Map editor sessions to IronClaw sessions/threads. The bridge may store editor
correlation IDs, but thread state remains owned by `SessionManager` and the DB.

### ACP Capabilities

Advertise only capabilities backed by IronClaw's actual route/tool/gate support.
Avoid hardcoded editor support claims; verify each client during implementation.

### ACP Wire Types

Define local DTOs near the bridge. Convert into IronClaw submissions and events
at the boundary.

### Ecosystem Adoption

Keep ecosystem notes out of implementation docs unless tied to tested adapters.
Editor support changes independently of IronClaw.

---

## 12. Captured Workflow ACP: Multi-Agent Coordination Layer

Captured workflow ACP concepts are useful vocabulary for sessions, phases, and
gates. They should not introduce a second orchestration stack.

### Captured Workflow ACP Core Concepts

Map concepts as:

| Captured concept | IronClaw home |
|------------------|---------------|
| session | `Session` / `Thread` |
| step | job/routine/tool execution state |
| gate | approval or Reborn gate |
| update stream | SSE/WebSocket events |

### Captured Workflow ACP Session Lifecycle

Lifecycle state should derive from existing thread/job/gate state rather than a
parallel workflow table unless persistence requirements demand it.

### Captured Workflow ACP Wire Types

If a facade is needed, make DTOs projection-only. Do not let facade DTOs become
the internal source of truth.

### Captured Workflow ACP JSON-RPC Request Types

Route requests into existing handlers or manager methods. Do not bypass
submission parsing, approval gates, cost guardrails, or tool safety.

---

## 13. ACP Workflow Pipeline State Machine

### Step Types

Use existing typed states where possible:

```text
message submitted | planning | tool pending approval | executing | completed | failed
```

### Workflow State Machine

Keep state transitions monotonic and auditable. The editor bridge should render
state; it should not own recovery or retry policy.

### Pipeline Execution Example: Code Review Session

Flow:

1. Editor sends review request.
2. Bridge creates/submits an IronClaw turn.
3. Tool approval gates surface through existing gate APIs.
4. Streaming events return to the editor.
5. Completion writes through normal session/job persistence.

---

## 14. ACP Streaming Session Updates

### SSE Event Types

Reuse `src/channels/web/types.rs` event contracts where practical. Add adapter
events only when editor clients need a shape that the browser gateway cannot
share.

### SSE Wire Format

SSE is one delivery option, not a protocol identity. Preserve event IDs and
reconnect semantics when proxying gateway events.

### IronClaw's Existing SSE Implementation

Use `SseManager` and existing gateway streams for browser/operator paths.
Editor-specific streams should not duplicate connection counters, auth logic, or
event filtering without a reason.

---

## 15. ACP Permission-Based Action Gates

### Gate Classification

Use the existing approval/gate taxonomy and only add classifications that affect
policy or UX.

### Relationship to IronClaw's ApprovalRequirement

Bridge gates into `ApprovalRequirement` or Reborn gate resolution. Do not invent
editor-only approval outcomes.

---

## 16. Captured MCP Component Set

Use captured crate names as decomposition hints only.

### Crate 1: roko-mcp-types

IronClaw home: `src/tools/mcp/protocol.rs` and adjacent DTO modules.

### Crate 2: roko-mcp-client

IronClaw home: `src/tools/mcp/client.rs`, transports, auth, and client store.

### Crate 3: roko-mcp-server

IronClaw server mode, if implemented, should live behind a dedicated module that
wraps `ToolRegistry` with auth, audit, and exposure policy.

### Crate 4: roko-acp

IronClaw editor bridge DTOs should stay separate from core session and tool
models.

### Crate 5: roko-editor-bridge

The bridge should translate editor requests into existing gateway/submission
operations.

---

## 17. Builtin Tool System

Core internal capabilities should remain built-in Rust tools only when they are
tightly coupled to runtime state, security, storage, or approvals. Otherwise use
WASM for maintained sandboxed capabilities and MCP for external servers.

All tool paths must converge on shared validation, timeout, execution,
sanitization, approval, and result processing.

---

## 18. Editor Integration: VS Code, Zed, and JetBrains

Avoid hardcoded support assumptions. Pick an editor only after checking its
tested extension APIs and protocol support for the implementation target.

### VS Code Integration via Language Server Protocol

Use LSP only for editor-native code intelligence or commands. Do not tunnel MCP
tool execution through LSP if a clearer bridge is available.

### Zed Integration via Extension API and ACP

Treat ACP support as versioned and client-specific. Keep IronClaw state changes
behind the same submission and approval APIs used by other channels.

### Other Editor Integrations

For additional editors, implement the thinnest bridge that maps editor messages
to IronClaw submissions and gateway-style events.

---

## 19. Practical Editor Integration Workflows

### Workflow 1: Code Review

Editor request -> IronClaw thread turn -> tools/search/read -> approval if
needed -> streamed findings -> persisted turn.

### Workflow 2: Autonomous Refactor with Human Gate

Editor request -> plan/job -> file edits gated by approval policy -> diff shown
by editor -> gate resolution through existing APIs.

### Workflow 3: Memory-Augmented Debugging

Editor request -> workspace memory/search tools -> debug summary. Persistent
memory remains the workspace system, not editor-local transcript storage.

### Workflow 4: MCP Server Onboarding

Editor/UI collects server config -> IronClaw validates transport/auth -> tools
listed -> registry wrappers activated per user/server.

### Workflow 5: Background Heartbeat with Editor Notification

Heartbeat/routine emits existing event -> bridge projects it to editor
notification. The editor does not run its own heartbeat engine.

---

## 20. Exposing IronClaw Tools as an MCP Server

Server mode is higher risk than client mode because it exposes IronClaw's tool
surface to external clients.

### MCP Server Endpoint Design

Minimum requirements:

- Explicit enablement and bind address.
- Authentication before initialize/tools/list/tools/call.
- Per-client exposure policy; do not publish every tool by default.
- Tool-call audit trail with user/client/server correlation.
- Approval propagation for mutating tools.
- Output sanitization identical to internal tool execution.
- Body, schema, and stream limits.

### CLI Flag for MCP Server Mode

Use a conservative flag/config gate, for example:

```text
ironclaw --mcp-server --mcp-server-bind 127.0.0.1:PORT
```

The exact CLI shape should follow existing config precedence and bootstrap
rules.

---

## 21. Benchmarking: Protocol Overhead Measurement

### What to Measure

- Initialize and tools/list latency.
- Tool-call request/response latency.
- Registry update time.
- OAuth refresh impact.
- Transport failure recovery.
- Output sanitization cost.

### Benchmark Harness

Use hermetic mock MCP servers for PR-gating benchmarks. Live servers are drift
checks only and must not require secrets in committed fixtures.

### Reporting Guidance

Report benchmark output in generated artifacts or CI logs, not prose docs. Avoid
fixed performance counts here.

---

## 22. Candidate Implementation Plan for IronClaw

Prioritize safety and compatibility where there is a caller.

### Phase 1: Enhanced MCP Client (Immediate)

Harden and extend the client path before server mode.

#### 1.1 Resources and Prompts Support

Add only if an IronClaw feature needs them. Treat resources and prompts as
untrusted server content with size and redaction limits.

#### 1.2 Tool List Change Notifications

Handle as a registry-surface update with conflict detection and audit.

#### 1.3 Sampling / LLM Proxy Compatibility

Do not let external MCP servers call IronClaw's LLM without explicit policy,
cost guardrails, auth, and audit.

#### 1.4 Server-Name Newtype Hardening

Continue validating server names at config boundaries and use the newtype in
maps, logs, and registry identifiers.

#### 1.5 Per-Server Health Checking

Health checks should update lifecycle state and gateway status without invoking
tools unless configured.

#### 1.6 MCP 2026 Stateless Protocol Compatibility

Implement only behind version negotiation and separate cache/session semantics.

### Phase 2: IronClaw as MCP Server (Medium Term)

Start with read-only, low-risk tools and explicit exposure policy. Add mutating
tools only after approval propagation and audit are proven.

### Phase 2b: Zed/JetBrains ACP Integration (Medium Term)

Build a thin editor bridge over existing session/gateway APIs. Confirm client
support and protocol version during implementation.

### Phase 3: Captured Workflow ACP Facade (Long Term)

Consider only if external workflow clients need a stable projection. Keep the
facade projection-only.

### Phase 4: Server-Side MCP Exposure with Tools/Call Audit Trail

Expose audit by client, user, server, tool, approval result, latency, and
sanitized outcome. Do not log secrets or raw prompts beyond configured trace
capture.

---

## 23. ACP vs MCP Comparison

| Concern | MCP | ACP/editor facade |
|---------|-----|-------------------|
| Primary role | Tool/resource/prompt protocol | Editor-agent session UX |
| IronClaw direction | Existing client, possible server | Thin bridge only |
| State owner | MCP session manager/client store | IronClaw session/thread/job state |
| Main risk | Untrusted tools and external processes | Duplicated agent loop or gates |
| Tests | Client/factory/tool wrapper | Bridge through submission/gate APIs |

---

## 24. Specification References

### Versioned Specifications and References

Use versioned MCP/ACP specifications at implementation time. Do not rely on this
document for complete wire compatibility.

### IronClaw Source Files

- `src/tools/mcp/protocol.rs`
- `src/tools/mcp/client.rs`
- `src/tools/mcp/transport.rs`
- `src/tools/mcp/client_store.rs`
- `src/tools/mcp/session.rs`
- `src/tools/mcp/process.rs`
- `src/tools/registry.rs`
- `src/channels/web/types.rs`
- `src/channels/web/platform/sse.rs`
- `src/channels/web/platform/ws.rs`

### Roko Reference Implementation

Captured Roko names in this document are provenance labels and decomposition
hints only. Prefer IronClaw owning modules when designing changes.

---

## 25. Related Documents

### In `/tmp/ecosystem/`

| Document | Use |
|----------|-----|
| [control-plane.md](control-plane.md) | Operator REST/SSE/WebSocket surface, gateway security, and projections. |
| [plugin-extension.md](plugin-extension.md) | Local extension discovery, WASM/TOML lifecycle, and event-source guidance. |

### In `/tmp/agent-intelligence/`

| Document | Use |
|----------|-----|
| [agent-patterns.md](../agent-intelligence/agent-patterns.md) | Adapter and task-runner patterns relevant to protocol bridges. |

### In `src/`

- `src/tools/README.md`
- `src/channels/web/CLAUDE.md`
- `src/agent/CLAUDE.md`
- `src/NETWORK_SECURITY.md`
