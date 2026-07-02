# Roko's ACP and MCP Editor Integration Protocols

## Purpose of This Document

This document provides a complete technical reference to Roko's Agent Communication Protocol (ACP) implementation and its five MCP server crates. It is written for someone encountering Roko for the first time, with zero prior context. Every protocol type, state machine transition, and wire format is documented with verified code from the roko codebase. The final section analyzes how IronClaw could adopt these patterns and maps them against IronClaw's existing MCP client in `src/tools/mcp/`.

---

## Table of Contents

1. [What Are ACP and MCP, and Why Both Exist](#1-what-are-acp-and-mcp-and-why-both-exist)
2. [Architecture Overview](#2-architecture-overview)
3. [The ACP JSON-RPC 2.0 Protocol](#3-the-acp-json-rpc-20-protocol)
4. [ACP Wire Types](#4-acp-wire-types)
5. [Session Management](#5-session-management)
6. [The Workflow Pipeline State Machine](#6-the-workflow-pipeline-state-machine)
7. [Streaming Session Updates](#7-streaming-session-updates)
8. [Permission-Based Action Gates](#8-permission-based-action-gates)
9. [MCP Server Integration Per-Session](#9-mcp-server-integration-per-session)
10. [The Five MCP Crates](#10-the-five-mcp-crates)
11. [Builtin Tool System](#11-builtin-tool-system)
12. [Event Bridging and Forwarding](#12-event-bridging-and-forwarding)
13. [Configuration and Hot-Reload](#13-configuration-and-hot-reload)
14. [Knowledge Injection](#14-knowledge-injection)
15. [Comparison with IronClaw's MCP Implementation](#15-comparison-with-ironclaws-mcp-implementation)
16. [How IronClaw Could Adopt These Patterns](#16-how-ironclaw-could-adopt-these-patterns)

---

## 1. What Are ACP and MCP, and Why Both Exist

### ACP: Agent Client Protocol

The Agent Client Protocol is an open standard initiated by Zed and JetBrains that standardizes communication between code editors and AI coding agents. It was announced in August 2025, with JetBrains joining shortly after to avoid competing standards. The protocol uses JSON-RPC 2.0 over stdio (newline-delimited JSON on standard input/output), so agents run as simple editor subprocesses. ACP is built for the editor-to-agent integration point -- it defines how an editor boots an agent, sends user prompts, streams progress back, and gates dangerous actions via permission requests.

Key ACP methods include:
- `session/new` -- create a conversation session with configuration and MCP server endpoints
- `session/prompt` -- send user text to the agent for processing within a session
- `session/update` -- streaming notification from agent to editor with progress events
- Permission request/response -- bidirectional JSON-RPC allowing the agent to ask the editor for approval of file edits, terminal commands, etc.

**Specification**: The ACP specification is maintained at [agentclientprotocol.com](https://agentclientprotocol.com/) and on [GitHub](https://github.com/agentclientprotocol/agent-client-protocol). SDKs exist for Rust, TypeScript, Python, Java, and Kotlin. The schema is published as JSON Schema attached to each release (`schema/v1/schema.json` for the stable version).

### MCP: Model Context Protocol

The Model Context Protocol, created by Anthropic and now governed by an open specification at [modelcontextprotocol.io](https://modelcontextprotocol.io/), defines how LLM applications connect to external tool servers. Where ACP handles the editor-to-agent boundary, MCP handles the agent-to-tooling boundary. The current stable specification version is `2025-11-25`, with a release candidate for `2026-07-28` in progress.

MCP defines three roles:
- **Host**: the LLM application that initiates connections
- **Client**: a connector within the host application
- **Server**: a service that provides tools, resources, or prompts

The protocol is built on [JSON-RPC 2.0](https://www.jsonrpc.org/specification) with the following message types:

1. **Requests**: Must include a string or integer `id` (never `null`). The `id` must not have been previously used within the same session.
2. **Responses**: Must include the same `id` as the request. Either `result` (success) or `error` (failure) with `code` + `message`.
3. **Notifications**: One-way messages. Must NOT include an `id` field.

MCP defines two transports:
- **stdio**: newline-delimited JSON over stdin/stdout for local subprocess servers
- **Streamable HTTP**: HTTP-based transport for remote servers, with `Mcp-Session-Id` header for session management and `MCP-Protocol-Version` header for version pinning

### Why Both Are Needed

The two protocols are complementary layers in the same stack. ACP sits between the editor and the agent; MCP sits between the agent and its tool servers. In practice:

1. The editor launches the agent as a subprocess via ACP (JSON-RPC over stdio)
2. During `session/new`, the editor passes MCP server configurations to the agent
3. The agent connects to those MCP servers and discovers their tools
4. When the LLM calls a tool, the agent invokes it via MCP
5. Progress, streaming output, and permission requests flow back to the editor via ACP

Roko implements both protocols: `roko-acp` is the ACP server that editors connect to, while five `roko-mcp-*` crates are MCP servers that provide Roko's tools to other agents.

---

## 2. Architecture Overview

```
Editor (VS Code, Zed, JetBrains, ...)
  |
  |  ACP (JSON-RPC 2.0 over stdio)
  |  session/new, session/prompt, session/update, permissions
  v
roko-acp (Agent)
  |
  +-- bridge_events.rs   CognitiveEvent -> session/update streaming
  +-- session.rs          AcpSession, SessionManager, CancelToken
  +-- handler.rs          JSON-RPC method dispatch
  +-- pipeline.rs         Workflow state machine (PipelinePhase, PipelineAction)
  +-- workflow.rs         WorkflowRun tracking (timing, cost, tokens)
  +-- runner.rs           Pipeline execution engine
  +-- builtin_tools.rs    /fix, /review, /status, /clear, /compact, /cost
  +-- acp_adapter.rs      RuntimeEvent -> CognitiveEvent bridge
  +-- event_forward.rs    CognitiveEvent -> RuntimeEvent forwarding
  +-- knowledge.rs        Dispatch-time knowledge retrieval
  +-- config_watch.rs     Live config hot-reload via notify
  +-- transport.rs        AcpTransport trait + StdioTransport
  +-- types.rs            Wire types (SessionNewParams, ContentBlock, etc.)
  |
  |  MCP (JSON-RPC 2.0 over stdio)
  |  initialize, tools/list, tools/call
  v
roko-mcp-* (Tool Servers)
  +-- roko-mcp-code       Code intelligence (symbol lookup, call graph, search)
  +-- roko-mcp-github     GitHub API (PRs, issues, files, reviews)
  +-- roko-mcp-slack      Slack API (post, reply, DM, channels)
  +-- roko-mcp-scripts    Script execution from configured directories
  +-- roko-mcp-stdio      Shared JSON-RPC 2.0 transport library
```

**Source location**: `crates/roko-acp/src/` (15 files) and `crates/roko-mcp-*/src/`.

---

## 3. The ACP JSON-RPC 2.0 Protocol

### Transport Layer

Roko's ACP transport is defined in `crates/roko-acp/src/transport.rs`:

```rust
// crates/roko-acp/src/transport.rs

/// Transport layer for ACP communication.
///
/// All messages are newline-delimited JSON-RPC 2.0 objects over a
/// byte-oriented pipe (typically stdin/stdout when the editor starts
/// the agent as a subprocess).
#[async_trait]
pub trait AcpTransport: Send + Sync + 'static {
    /// Receive the next JSON-RPC request or notification from the editor.
    async fn receive(&mut self) -> Option<serde_json::Value>;
    /// Send a JSON-RPC response or notification back to the editor.
    async fn send(&self, message: &serde_json::Value) -> Result<(), AcpTransportError>;
}
```

The `StdioTransport` implementation reads newline-delimited JSON from stdin via `BufReader` and writes to stdout, each message terminated by `\n`. This matches the transport model used by the Language Server Protocol and is the canonical ACP transport.

### JSON-RPC 2.0 Compliance

Both ACP and MCP follow the [JSON-RPC 2.0 specification](https://www.jsonrpc.org/specification). The key rules are:

1. **Request**: `{"jsonrpc": "2.0", "id": <number|string>, "method": "...", "params": {...}}`
2. **Response**: `{"jsonrpc": "2.0", "id": <same-as-request>, "result": {...}}` or `{"jsonrpc": "2.0", "id": <same-as-request>, "error": {"code": ..., "message": "..."}}`
3. **Notification**: `{"jsonrpc": "2.0", "method": "...", "params": {...}}` -- no `id` field, no response expected

Standard error codes (from the JSON-RPC spec and MCP extensions):

| Code | Meaning |
|------|---------|
| `-32700` | Parse error -- invalid JSON |
| `-32600` | Invalid Request -- not a valid JSON-RPC object |
| `-32601` | Method not found |
| `-32602` | Invalid params |
| `-32603` | Internal error |

### Handler Dispatch

The ACP handler in `crates/roko-acp/src/handler.rs` dispatches methods to their implementations:

```rust
// crates/roko-acp/src/handler.rs (method dispatch)
match method {
    "initialize"           => handle_initialize(&config, &params),
    "session/new"          => handle_session_new(&mut session_manager, &params, &config).await,
    "session/prompt"       => handle_session_prompt(&mut session_manager, &params, &config).await,
    "session/list"         => handle_session_list(&session_manager),
    "session/cancel"       => handle_session_cancel(&mut session_manager, &params).await,
    "session/set_config"   => handle_session_set_config(&mut session_manager, &params, &config).await,
    "session/set_mode"     => handle_session_set_mode(&mut session_manager, &params, &config).await,
    "session/end"          => handle_session_end(&mut session_manager, &params).await,
    // Slash commands
    "session/slash_command" => handle_slash_command(&mut session_manager, &params, &config).await,
    _                      => Err(method_not_found(method)),
}
```

The `initialize` method follows the same pattern as MCP -- capability negotiation. The response includes the agent's `protocolVersion`, `capabilities`, and `serverInfo`. The key difference from MCP is that ACP's `session/new` creates a stateful session with configuration, MCP server configs, and client capabilities.

---

## 4. ACP Wire Types

All ACP wire types are defined in `crates/roko-acp/src/types.rs`. These are the structs that are serialized/deserialized on the JSON-RPC wire.

### Session Creation

```rust
// crates/roko-acp/src/types.rs

/// Parameters for the `session/new` JSON-RPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionNewParams {
    /// Optional human-readable name for the session.
    pub session_name: Option<String>,
    /// Client-declared capabilities (edit tracking, terminal, etc.).
    #[serde(default)]
    pub client_capabilities: ClientCapabilities,
    /// MCP servers the editor wants the agent to connect to.
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
}

/// Client capabilities declared by the editor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    /// Whether the editor supports edit tracking.
    #[serde(default)]
    pub edit_tracking: bool,
    /// Whether the editor provides a terminal for command execution.
    #[serde(default)]
    pub terminal: bool,
}
```

### Content Blocks

ACP content blocks mirror the MCP specification's content model:

```rust
// crates/roko-acp/src/types.rs

/// A single content block in a session update or tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content.
    Text { text: String },
    /// Rendered thinking/reasoning content (shown in a collapsed panel).
    Thinking { text: String },
    /// A resource reference (file, URL).
    Resource(ResourceRef),
}
```

### Tool Call Status

```rust
// crates/roko-acp/src/types.rs

/// Status of a tool call in the pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Classification of tool calls for editor UI rendering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallKind {
    FileEdit,
    FileCreate,
    TerminalCommand,
    McpToolCall,
    Other,
}

/// Stop reason for prompt completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    Cancelled,
}
```

### Permission Actions

```rust
// crates/roko-acp/src/types.rs

/// Action types that require user permission before proceeding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionAction {
    FileEdit,
    FileCreate,
    FileDelete,
    TerminalCommand,
    NetworkRequest,
    GitOperation,
}
```

---

## 5. Session Management

### AcpSession

Each session tracks its state, configuration, and conversation history. The core type in `crates/roko-acp/src/session.rs`:

```rust
// crates/roko-acp/src/session.rs

/// ACP server-side session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpSession {
    pub session_id: String,
    pub session_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub config_state: SessionConfigState,
    pub client_capabilities: ClientCapabilities,
    #[serde(default)]
    pub cancel_token: CancelToken,
    #[serde(skip, default = "new_atomic_flag")]
    pub busy: Arc<AtomicBool>,
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default)]
    pub config_options: Vec<ConfigOption>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub conversation_history: Vec<ConversationTurn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_run: Option<WorkflowRun>,
    #[serde(skip, default = "new_shared_run")]
    pub shared_run: SharedWorkflowRun,
    #[serde(skip)]
    pub cached_conventions: Option<String>,
    #[serde(skip, default)]
    pub always_allowed: HashSet<crate::types::PermissionAction>,
    #[serde(default = "default_true")]
    pub tools_enabled: bool,
    #[serde(default)]
    pub pinned_context: Vec<PinnedFile>,
}
```

Key design choices:
- `busy: Arc<AtomicBool>` -- prevents concurrent prompts on the same session. The handler checks this flag atomically before accepting a new `session/prompt`.
- `cancel_token: CancelToken` -- cooperative cancellation. The editor calls `session/cancel`, which sets the token's `AtomicBool`; the runner checks it periodically during long operations.
- `always_allowed: HashSet<PermissionAction>` -- session-scoped "always allow" decisions loaded from workspace trust and updated on user approval. Eliminates repeated permission prompts for safe actions.
- `shared_run: SharedWorkflowRun` -- an `Arc<Mutex<Option<WorkflowRun>>>` that the runner updates in real time so that slash commands (`/status`) and status queries can read the live pipeline state.

### CancelToken

```rust
// crates/roko-acp/src/session.rs

/// A lightweight cooperative cancellation token for ACP session work.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelToken {
    #[serde(skip, default = "new_atomic_flag")]
    cancelled: Arc<AtomicBool>,
    #[serde(skip, default = "new_notify")]
    notify: Arc<Notify>,
}
```

The token uses `AtomicBool` for the cancel signal and `tokio::sync::Notify` so async tasks can `await` cancellation rather than polling. This is the same pattern used by Go's `context.Context` -- propagate cancellation through a call tree without blocking threads.

### Session Config State

Each session carries a `SessionConfigState` that determines runtime behavior:

```rust
// crates/roko-acp/src/session.rs

/// Session-scoped configuration state.
pub struct SessionConfigState {
    pub agent_mode: String,       // "code", "review", "architect", etc.
    pub provider: String,         // LLM provider key
    pub model: String,            // model name
    pub effort: String,           // effort level
    pub clippy_enabled: bool,     // whether clippy gate runs
    pub tests_enabled: bool,      // whether test gate runs
    pub workflow: String,         // "none", "express", "standard", "full", "auto"
    pub review_strictness: String, // reviewer strictness
    pub max_iterations: u32,      // max auto-fix iterations
}
```

The config state is initialized from `roko.toml` defaults at session creation, with fallback logic that selects the first available model/provider.

---

## 6. The Workflow Pipeline State Machine

### Pipeline Phases

The pipeline is a pure state machine in `crates/roko-acp/src/pipeline.rs` -- it receives events and emits actions, with no I/O side effects:

```rust
// crates/roko-acp/src/pipeline.rs

/// Pipeline execution phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelinePhase {
    Pending,        // Created but not started
    Strategizing,   // Strategist agent analyzing prompt
    Implementing,   // Implementer agent writing code
    AutoFixing,     // Auto-fixer patching gate failures
    Gating,         // Gates (compile, test, clippy) running
    Reviewing,      // Reviewer agent(s) analyzing changes
    Committing,     // Creating a commit
    Complete,       // Pipeline completed successfully
    Halted { reason: String },  // Stopped (timeout, budget, cancel)
    Cancelled,      // User cancelled
}
```

### Pipeline Actions

The state machine emits actions that the runner executes:

```rust
// crates/roko-acp/src/pipeline.rs

/// Actions emitted by the pipeline state machine for the runner to execute.
#[derive(Debug, Clone)]
pub enum PipelineAction {
    /// Spawn the strategist agent with the raw prompt.
    SpawnStrategist { prompt: String },
    /// Spawn the implementer agent with the brief.
    SpawnImplementer { brief: String },
    /// Spawn the auto-fixer with gate failure output.
    SpawnAutoFixer { gate_output: String },
    /// Run the gate pipeline (compile, test, clippy as configured).
    RunGates,
    /// Spawn reviewer agent(s).
    SpawnReviewer { diff_context: String },
    /// Create a commit.
    Commit,
    /// Pipeline is done.
    Done,
    /// Pipeline halted -- persist state for resume.
    Halt { reason: String },
}
```

### Workflow Templates

Three templates determine which phases run:

```rust
// crates/roko-acp/src/pipeline.rs

/// Workflow template that determines which phases to run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowTemplate {
    /// Implement -> gate -> commit (fastest).
    Express,
    /// Implement -> gate -> review -> commit.
    Standard,
    /// Strategy -> implement -> gate -> multi-review -> commit.
    Full,
}
```

The `auto_select` function picks a template based on prompt analysis:

```rust
// crates/roko-acp/src/pipeline.rs

impl WorkflowTemplate {
    /// Select a template automatically based on prompt characteristics.
    pub fn auto_select(prompt: &str) -> Self {
        let word_count = prompt.split_whitespace().count();
        let has_multi_file_hints = prompt.contains("files")
            || prompt.contains("modules")
            || prompt.contains("system")
            || prompt.contains("architecture")
            || prompt.contains("refactor");
        let has_simple_hints = prompt.contains("fix")
            || prompt.contains("typo")
            || prompt.contains("rename")
            || prompt.contains("update")
            || prompt.contains("bump");

        if has_simple_hints && word_count < 15 {
            Self::Express
        } else if has_multi_file_hints || word_count > 50 {
            Self::Full
        } else {
            Self::Standard
        }
    }
}
```

### State Transitions

The pipeline state machine transitions:

```
Express:   Pending -> Implementing -> Gating -> Committing -> Complete
Standard:  Pending -> Implementing -> Gating -> Reviewing -> Committing -> Complete
Full:      Pending -> Strategizing -> Implementing -> Gating -> Reviewing -> Committing -> Complete

Any phase can transition to:
  -> Halted { reason }   (on timeout, budget, or error)
  -> Cancelled           (on user cancellation)
  -> AutoFixing          (on gate failure, up to max_iterations)
     -> Gating           (re-run gates after fix)
```

### WorkflowRun

`WorkflowRun` wraps the state machine with tracking metadata:

```rust
// crates/roko-acp/src/workflow.rs

/// A single workflow pipeline execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRun {
    pub run_id: String,
    pub pipeline: PipelineState,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub total_cost_usd: Option<f64>,
    pub total_tokens: Option<u64>,
    pub agents_spawned: u32,
}
```

---

## 7. Streaming Session Updates

### CognitiveEvent

The cognitive event system bridges the internal agent loop to ACP session updates. Defined in `crates/roko-acp/src/bridge_events.rs`:

```rust
// crates/roko-acp/src/bridge_events.rs

/// Events emitted by the cognitive loop and mapped to ACP session updates.
#[derive(Debug, Clone)]
pub enum CognitiveEvent {
    /// A streamed agent-visible text chunk.
    TokenChunk(String),
    /// A streamed internal reasoning chunk.
    ThinkingChunk(String),
    /// A tool call has started running.
    ToolCallStart {
        tool_call_id: String,
        title: String,
        kind: ToolCallKind,
        locations: Option<Vec<ToolCallLocation>>,
    },
    /// A tool call has finished with rendered content.
    ToolCallComplete {
        tool_call_id: String,
        status: ToolCallStatus,
        content: Vec<ContentBlock>,
    },
    /// A plan update with structured entries.
    PlanUpdate { entries: Vec<PlanEntry> },
    /// MCP server discovery results.
    McpStatus { statuses: Vec<McpServerStatus> },
    /// Prompt execution completed normally.
    Complete {
        stop_reason: StopReason,
        usage: Option<UsageInfo>,
    },
    /// Prompt execution failed before normal completion.
    Failure { message: String },
    /// Prompt execution stopped because the token budget was exhausted.
    MaxTokens,
}
```

These events are sent as `session/update` JSON-RPC notifications to the editor. The editor renders them in real time: `TokenChunk` as streaming text, `ToolCallStart`/`ToolCallComplete` as collapsible panels, `PlanUpdate` as a progress checklist.

### Session/Update Wire Format

Each `CognitiveEvent` is serialized into the `session/update` notification:

```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "sessionId": "sess_abc123",
    "event": {
      "type": "tool_call_start",
      "toolCallId": "tc_1",
      "title": "Running clippy gate",
      "kind": "other"
    }
  }
}
```

---

## 8. Permission-Based Action Gates

ACP uses bidirectional JSON-RPC to allow the agent to request permission from the editor before performing dangerous actions. This is the trust layer where the developer stays in control.

### Permission Flow

1. Agent encounters a destructive action (file edit, terminal command, git operation)
2. Agent sends a JSON-RPC **request** to the editor (not a notification) asking for permission
3. Editor shows the user a dialog with the action details
4. User approves, denies, or selects "always allow"
5. Editor sends a JSON-RPC **response** back to the agent
6. If "always allow", the action is added to `session.always_allowed` and subsequent calls skip the permission check

### Permission Actions

Six action types require permission (from `types.rs`):

| Action | Description |
|--------|-------------|
| `FileEdit` | Writing or editing a file |
| `FileCreate` | Creating a new file |
| `FileDelete` | Deleting a file |
| `TerminalCommand` | Running a terminal command |
| `NetworkRequest` | Making a network request |
| `GitOperation` | Running a git operation |

### User Decision Types

```rust
// crates/roko-acp/src/types.rs

/// User decision in response to a permission request.
pub enum PermissionDecision {
    Allow,
    AlwaysAllow,
    Deny,
}
```

This permission model differs from MCP's tool annotations. MCP provides *hints* (`destructiveHint`, `readOnlyHint`) that the host can use to decide policy. ACP makes the permission check a first-class protocol operation with a synchronous request-response cycle.

---

## 9. MCP Server Integration Per-Session

Each ACP session can declare MCP servers to connect to. The editor passes `McpServerConfig` entries during `session/new`, and the agent connects to each one.

```rust
// crates/roko-acp/src/types.rs

/// MCP server configuration passed by the editor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    /// Server name (used as tool prefix).
    pub name: String,
    /// Server URL or command.
    pub uri: String,
    /// Transport type.
    pub transport: McpTransportType,
    /// Optional environment variables to pass.
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}
```

The agent uses `roko-mcp-stdio` to connect to each server. During the session, the agent:

1. Sends `initialize` with its capabilities
2. Sends `notifications/initialized` after handshake
3. Calls `tools/list` to discover available tools
4. Calls `tools/call` when the LLM invokes a tool

The five `roko-mcp-*` crates are standalone MCP servers that can be used by any MCP client, not just Roko's ACP agent. They follow the MCP specification's `2024-11-05` protocol version.

---

## 10. The Five MCP Crates

### roko-mcp-stdio (Shared Transport Library)

`crates/roko-mcp-stdio/src/lib.rs` provides the shared JSON-RPC 2.0 stdio transport used by all other MCP crates:

```rust
// crates/roko-mcp-stdio/src/lib.rs

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    #[serde(default)]
    pub id: Value,
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INTERNAL_ERROR: i64 = -32603;
}
```

The `serve_stdio` function runs the event loop:

```rust
// crates/roko-mcp-stdio/src/lib.rs

/// Serve an MCP-style JSON-RPC loop over stdin/stdout.
pub fn serve_stdio<R, W, F>(reader: R, mut writer: W, mut handler: F) -> anyhow::Result<()>
where
    R: BufRead,
    W: Write,
    F: FnMut(JsonRpcRequest) -> Result<Value, JsonRpcError>,
{
    for line in reader.lines() {
        let line = line.context("read JSON-RPC line")?;
        if line.trim().is_empty() {
            continue;
        }
        // Parse JSON, validate jsonrpc field, dispatch to handler
        // Notifications (no id) are handled by calling handler and discarding the result
        // Requests (with id) get a JSON-RPC response written back
    }
    Ok(())
}
```

Each MCP server uses the same pattern: `serve_stdio(stdin, stdout, |request| handle_request(request))`. The handler dispatches on `request.method`:

- `"initialize"` -- returns `protocolVersion`, `capabilities`, `serverInfo`
- `"tools/list"` -- returns the list of available tools with JSON Schema for input parameters
- `"tools/call"` -- executes a tool and returns content blocks

### roko-mcp-code (Code Intelligence)

Backed by `roko-index`, this server provides code navigation and search tools:

| Tool | Description |
|------|-------------|
| `symbol_lookup` | Find symbol definitions by name |
| `call_graph` | Trace function call chains up to a depth |
| `imports` | List imports for a file |
| `semantic_search` | Search code by meaning |
| `search_code` | Multi-strategy code search (keyword, structural, HDC, embedding, hybrid) |
| `project_summary` | File tree with symbol counts |

The `search_code` tool supports five strategies:
```rust
// crates/roko-mcp-code/src/lib.rs
enum SearchStrategy { Keyword, Structural, Hdc, Embedding, Hybrid }
```

### roko-mcp-github (GitHub API)

Wraps the GitHub REST API with authentication, rate-limit handling, and retry logic:

| Tool | Description |
|------|-------------|
| `list_prs` | List pull requests with filtering |
| `get_pr` | Get PR details |
| `create_pr` | Create a pull request |
| `list_issues` | List issues with filtering |
| `create_issue` | Create an issue |
| `get_file` | Read a file from a repository |
| `search_code` | Search code across repositories |

Uses `GITHUB_TOKEN` environment variable for authentication. Handles GitHub's rate limiting via `X-RateLimit-*` and `Retry-After` headers with exponential backoff.

### roko-mcp-slack (Slack Web API)

| Tool | Description |
|------|-------------|
| `post_message` | Post to a channel (with optional thread) |
| `reply` | Reply in a thread |
| `get_thread` | Get thread messages |
| `react` | Add an emoji reaction |
| `list_channels` | List available channels |
| `lookup_user` | Find a user by email or name |
| `dm` | Send a direct message |
| `get_channel_history` | Get recent channel messages |
| `update_message` | Edit an existing message |

Uses `SLACK_BOT_TOKEN` for authentication.

### roko-mcp-scripts (Script Execution)

Runs scripts from configured directories with sandboxing:

| Tool | Description |
|------|-------------|
| `run_script` | Execute a script by name with arguments |
| `list_scripts` | List available scripts with descriptions |

Configuration via environment variables:
- `SCRIPTS_ROOT` / `SCRIPT_ROOTS` -- directories to scan for scripts
- `SCRIPT_TIMEOUT_SECS` -- execution timeout
- `SCRIPT_ENV_ALLOWLIST` -- environment variables passed to scripts

Scripts are discovered by scanning the configured roots. A `# description:` comment near the top of each script file provides the description used in `tools/list`.

---

## 11. Builtin Tool System

Roko's ACP layer has its own builtin tools -- slash commands that the user can invoke directly, separate from MCP tools. Defined in `crates/roko-acp/src/builtin_tools.rs`:

| Command | Description |
|---------|-------------|
| `/fix` | Spawn auto-fixer for compile/test/lint errors |
| `/review` | Trigger code review of current changes |
| `/status` | Show active workflow status |
| `/clear` | Clear conversation history |
| `/compact` | Compress conversation history to save tokens |
| `/cost` | Show accumulated cost and token usage |

These are dispatched via `session/slash_command` rather than `session/prompt`, giving the ACP handler a fast path that doesn't go through the full cognitive loop.

---

## 12. Event Bridging and Forwarding

Roko has two event bridge adapters that translate between the ACP cognitive event model and the runtime event system:

### AcpAdapter (RuntimeEvent -> CognitiveEvent)

`crates/roko-acp/src/acp_adapter.rs` implements `EventConsumer` from `roko-core` to receive `RuntimeEvent`s from the workflow engine and map them to `CognitiveEvent`s for ACP streaming:

```rust
// crates/roko-acp/src/acp_adapter.rs

pub struct AcpAdapter {
    session_id: String,
    run_id: String,
    sender: mpsc::Sender<CognitiveEvent>,
}

impl EventConsumer for AcpAdapter {
    fn consume(&self, event: &RuntimeEvent) {
        if let Some(cognitive_event) = self.map_event(event) {
            let _ = self.sender.try_send(cognitive_event);
        }
    }
}
```

Event mappings:
- `RuntimeEvent::AgentOutput` -> `CognitiveEvent::TokenChunk`
- `RuntimeEvent::AgentSpawned` -> `CognitiveEvent::ToolCallStart`
- `RuntimeEvent::AgentCompleted` -> `CognitiveEvent::ToolCallComplete { status: Completed }`
- `RuntimeEvent::AgentFailed` -> `CognitiveEvent::ToolCallComplete { status: Failed }`
- `RuntimeEvent::GateStarted` -> `CognitiveEvent::ToolCallStart`
- `RuntimeEvent::GatePassed` -> `CognitiveEvent::ToolCallComplete { status: Completed }`
- `RuntimeEvent::GateFailed` -> `CognitiveEvent::ToolCallComplete { status: Failed }`
- `RuntimeEvent::InferenceStarted` -> `CognitiveEvent::ToolCallStart`
- `RuntimeEvent::InferenceCompleted` -> `CognitiveEvent::ToolCallComplete { status: Completed }`
- `RuntimeEvent::PhaseTransition` -> `CognitiveEvent::TokenChunk("[Phase: X -> Y]")`
- `RuntimeEvent::WorkflowCompleted` -> `CognitiveEvent::Complete`

The adapter filters events by `run_id` so only events for the current session's active run are forwarded.

### AcpEventForwarder (CognitiveEvent -> RuntimeEvent)

`crates/roko-acp/src/event_forward.rs` does the reverse mapping, forwarding ACP cognitive events into the runtime event pipeline for centralized observability:

```rust
// crates/roko-acp/src/event_forward.rs

pub struct AcpEventForwarder {
    sink: HttpEventSink,
    run_id: String,
    agent_id: String,
}
```

This enables the `roko-serve` control plane to monitor ACP sessions through the same event infrastructure used for non-ACP workflows.

---

## 13. Configuration and Hot-Reload

### ConfigWatcher

`crates/roko-acp/src/config_watch.rs` uses the `notify` crate for filesystem-event-driven config reloading:

```rust
// crates/roko-acp/src/config_watch.rs

pub struct ConfigWatcher {
    _watcher: Option<RecommendedWatcher>,
    rx: mpsc::Receiver<()>,
    cache: Option<Arc<roko_core::config::ConfigCache>>,
}
```

The watcher monitors `roko.toml` and related config files. When a change is detected, it sends a signal via `mpsc::Receiver<()>`. The ACP handler loop polls `changed()` and reloads the config via `AcpConfig::load_roko_config()`. The `ConfigCache` provides zero-copy reads of the latest config via an `Arc`-swap pattern.

---

## 14. Knowledge Injection

### Dispatch-Time Knowledge

`crates/roko-acp/src/knowledge.rs` queries the durable knowledge store (`roko-neuro`) and playbook store (`roko-learn`) at dispatch time to inject relevant prior knowledge into each prompt:

```rust
// crates/roko-acp/src/knowledge.rs

pub(crate) struct DispatchKnowledge {
    /// Ranked knowledge hits from roko-neuro.
    pub hits: Vec<KnowledgeQueryHit>,
    /// Ranked playbooks from roko-learn.
    pub playbooks: Vec<Playbook>,
}
```

Two outputs:
1. **KnowledgeCard**: A visible card in the editor UI showing "Prior knowledge -- N results"
2. **Context text**: System prompt injection with the knowledge formatted for the LLM

The query is async and runs both knowledge and playbook lookups concurrently via `tokio::join!`. Empty results are gracefully handled -- dispatch continues without knowledge context.

---

## 15. Comparison with IronClaw's MCP Implementation

IronClaw already has a mature MCP client implementation in `src/tools/mcp/`. Here is a feature-by-feature comparison:

### Protocol Implementation

| Feature | IronClaw | Roko |
|---------|----------|------|
| Protocol version | `2024-11-05` | `2024-11-05` |
| JSON-RPC types | `McpRequest`, `McpResponse`, `McpError` | `JsonRpcRequest`, `JsonRpcError` (via roko-mcp-stdio) |
| MCP role | **Client** (connects to external servers) | **Both** -- client (connects to MCP servers per session) and **server** (5 MCP crates expose Roko's own tools) |
| ACP role | None (not implemented) | **Server** (editors connect to Roko via ACP) |

### Transport Support

| Transport | IronClaw | Roko |
|-----------|----------|------|
| stdio (subprocess) | Yes (`stdio_transport.rs`, `McpProcessManager`) | Yes (via `roko-mcp-stdio`) |
| HTTP (Streamable HTTP) | Yes (`http_transport.rs`, `HttpMcpTransport`) | No (stdio only for MCP servers) |
| Unix domain socket | Yes (`unix_transport.rs`, `#[cfg(unix)]`) | No |
| OAuth authentication | Yes (`auth.rs`, token refresh, DCR) | No (env-var tokens only) |

### Session Management

| Feature | IronClaw | Roko |
|---------|----------|------|
| Session ID tracking | `McpSessionManager` with `(user_id, server_name)` composite key | Per-ACP-session MCP connections |
| Cross-tenant isolation | `McpSessionKey` prevents session ID collision | Not applicable (single-user agent) |
| Stale session cleanup | `cleanup_stale()` with configurable idle timeout (30min default) | N/A |
| Mcp-Session-Id header | Captured from responses, sent on subsequent requests | N/A (stdio transport) |
| Server name validation | `McpServerName` newtype with allowlist validation | String-based |

### Tool Discovery and Annotations

| Feature | IronClaw | Roko |
|---------|----------|------|
| `tools/list` | Full implementation with caching (`tools_cache: RwLock<Option<Vec<McpTool>>>`) | Full implementation in each MCP server |
| Tool annotations | `McpToolAnnotations` with `destructiveHint`, `readOnlyHint`, `sideEffectsHint`, `executionTimeHint` | Not used (no annotation metadata) |
| Approval requirement | `requires_approval()` checks `destructiveHint` | ACP permission gate system instead |
| Tool ID namespacing | Server-name prefix: `{server}_{tool_name}` | N/A (tools listed by name directly) |

### Factory Pattern

IronClaw has a sophisticated factory in `src/tools/mcp/factory.rs`:

```rust
// src/tools/mcp/factory.rs -- transport dispatch
match server.effective_transport() {
    EffectiveTransport::Stdio { command, args, env } => { /* spawn subprocess */ },
    EffectiveTransport::Unix { socket_path } => { /* connect Unix socket */ },
    EffectiveTransport::Http => {
        if has_tokens || server.requires_auth() {
            McpClient::new_authenticated(server, session_manager, secrets, user_id)
        } else {
            // Wire session manager into transport for Mcp-Session-Id capture
            HttpMcpTransport::new(url, name).with_session_manager(session_manager, user_id)
        }
    },
}
```

Roko's MCP servers are simpler -- they are standalone binaries that read from stdin and write to stdout, connected by the editor or the ACP agent.

### Key Differences Summary

1. **IronClaw is an MCP client; Roko is both an MCP client and server.** IronClaw connects to external MCP servers to use their tools. Roko also exposes its own tools as MCP servers for other agents.

2. **IronClaw has no ACP layer.** Roko's `roko-acp` crate is a full ACP server that editors connect to. IronClaw's editor integration goes through its web gateway (`src/channels/web/`).

3. **IronClaw has richer transport support.** HTTP with OAuth, Unix sockets, and session management with cross-tenant isolation. Roko's MCP servers are stdio-only.

4. **Roko has richer pipeline orchestration.** The workflow state machine (Express/Standard/Full templates), auto-fix loops, and multi-agent strategies don't exist in IronClaw's MCP layer.

5. **Permission models differ.** IronClaw uses MCP tool annotations (`destructiveHint`) for approval gating. Roko uses ACP's bidirectional permission request/response protocol.

---

## 16. How IronClaw Could Adopt These Patterns

### Phase 1: ACP Server Implementation

IronClaw could expose itself as an ACP server, allowing editors like Zed, JetBrains, and VS Code to connect directly instead of only through the web gateway.

**Mapping to existing code:**

| ACP concept | IronClaw equivalent | Gap |
|-------------|---------------------|-----|
| `AcpTransport` trait | `src/channels/channel.rs` `Channel` trait | New impl needed for stdio JSON-RPC |
| `AcpSession` | `src/agent/` session management | Needs ACP wire format adapter |
| `CognitiveEvent` streaming | `src/channels/web/server.rs` WebSocket events | Already has streaming; needs ACP format |
| Permission gates | `src/tools/mcp/protocol.rs` `McpToolAnnotations` | Need bidirectional request/response |
| Config hot-reload | `src/settings.rs` | Add filesystem watcher |

**Implementation approach:**

1. Add a new channel: `src/channels/acp/` implementing the `Channel` trait
2. The `AcpChannel` would use `BufReader<Stdin>` / `Stdout` as transport
3. Map incoming `session/prompt` to `IncomingMessage`
4. Map outgoing events to `session/update` notifications
5. Bridge the permission system to IronClaw's existing tool approval flow (`ApprovalRequirement` in `src/tools/tool.rs`)

### Phase 2: Expose IronClaw Tools as MCP Servers

Following Roko's pattern, IronClaw could expose its builtin tools as standalone MCP servers. This would let other agents use IronClaw's tools.

**Candidates for MCP server exposure:**

| IronClaw tool | MCP server equivalent |
|---------------|----------------------|
| `memory_search`, `memory_write`, `memory_read`, `memory_tree` | `ironclaw-mcp-memory` |
| `shell` | `ironclaw-mcp-shell` |
| `file_read`, `file_write`, `apply_patch` | `ironclaw-mcp-files` |
| `web_fetch` | `ironclaw-mcp-web` |
| `job_create`, `job_status` | `ironclaw-mcp-jobs` |

**Reuse opportunity:** IronClaw already has `roko-mcp-stdio`-equivalent types in `src/tools/mcp/protocol.rs` (`McpRequest`, `McpResponse`, `McpTool`, `ContentBlock`). A thin `serve_stdio` wrapper could turn any `Tool` impl into a standalone MCP server binary.

### Phase 3: Workflow Pipeline

Roko's pure state machine pipeline is a clean pattern that could improve IronClaw's agent loop:

| Roko concept | IronClaw mapping |
|--------------|------------------|
| `PipelinePhase` | Job state machine in `src/context/` (Pending -> InProgress -> Completed) |
| `WorkflowTemplate` (Express/Standard/Full) | New concept -- IronClaw currently has one execution model |
| `auto_select` based on prompt analysis | Could extend `src/estimation/` cost/time estimation |
| Gate pipeline (compile, test, clippy) | New concept -- IronClaw has sandbox but no automatic quality gates |
| Auto-fix loop | New concept -- retry with gate output injected |

### Phase 4: Knowledge Injection

Roko's dispatch-time knowledge retrieval could enhance IronClaw's existing workspace memory:

| Roko concept | IronClaw mapping |
|--------------|------------------|
| `DispatchKnowledge` | `src/workspace/` memory search (`memory_search` tool) |
| `KnowledgeCard` for editor UI | Web gateway could show knowledge cards |
| Playbook retrieval | `src/skills/` SKILL.md system already does this |
| System prompt injection | IronClaw already injects identity files (AGENTS.md, SOUL.md) |

The gap is *automatic* knowledge injection at dispatch time. IronClaw currently requires the agent to explicitly call `memory_search`. Adding a pre-dispatch hook that queries the workspace and injects relevant memories into the system prompt would close this gap.

### Phase 5: Session Management Enhancements

IronClaw's `McpSessionManager` is already more robust than Roko's for the MCP client role (cross-tenant isolation, stale cleanup, typed server names). For the ACP server role, IronClaw would need:

1. **Session config state** -- per-session model/provider/effort selection (Roko's `SessionConfigState`)
2. **Cooperative cancellation** -- `CancelToken` with `AtomicBool` + `Notify` (Roko's pattern is clean and adoptable)
3. **Conversation history** -- per-session multi-turn context with bounded storage (Roko's `MAX_HISTORY_ASSISTANT_BYTES = 10_240`)
4. **Always-allow persistence** -- session-scoped permission decisions loaded from workspace trust

---

## References

### Specifications

1. **Model Context Protocol Specification (2025-11-25)**: [modelcontextprotocol.io/specification/2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25). The authoritative reference for MCP, defining JSON-RPC 2.0 message types, lifecycle management, transport options, capability negotiation, and tool/resource/prompt features.

2. **MCP Base Protocol**: [modelcontextprotocol.io/specification/2025-11-25/basic](https://modelcontextprotocol.io/specification/2025-11-25/basic). Details JSON-RPC message format, transports (stdio and Streamable HTTP), and lifecycle (initialize, initialized notification, shutdown).

3. **MCP Lifecycle Management**: [modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle). Initialization handshake, version negotiation, capability negotiation, and shutdown procedures.

4. **JSON-RPC 2.0 Specification**: [jsonrpc.org/specification](https://www.jsonrpc.org/specification). The underlying wire protocol used by both ACP and MCP.

5. **Agent Client Protocol (ACP)**: [agentclientprotocol.com](https://agentclientprotocol.com/). The open standard for editor-to-agent communication, co-developed by Zed and JetBrains.

6. **ACP GitHub Repository**: [github.com/agentclientprotocol/agent-client-protocol](https://github.com/agentclientprotocol/agent-client-protocol). Protocol schema (`schema/v1/schema.json`), SDKs (Rust, TypeScript, Python, Java, Kotlin), and specification.

7. **ACP Architecture**: [agentclientprotocol.com/get-started/architecture](https://agentclientprotocol.com/get-started/architecture). Editor/agent roles, communication patterns, MCP integration design.

### Roko Source Code

All code citations verified against the roko repository at `roko/crates/roko-acp/src/` and `roko/crates/roko-mcp-*/src/`:

| File | Contents |
|------|----------|
| `crates/roko-acp/src/types.rs` | ACP wire types (SessionNewParams, ContentBlock, ToolCallKind, PermissionAction, etc.) |
| `crates/roko-acp/src/transport.rs` | AcpTransport trait, StdioTransport |
| `crates/roko-acp/src/session.rs` | AcpSession, SessionManager, CancelToken, SessionConfigState |
| `crates/roko-acp/src/handler.rs` | JSON-RPC method dispatch |
| `crates/roko-acp/src/pipeline.rs` | PipelinePhase, PipelineAction, WorkflowTemplate, auto_select |
| `crates/roko-acp/src/workflow.rs` | WorkflowRun tracking |
| `crates/roko-acp/src/bridge_events.rs` | CognitiveEvent enum |
| `crates/roko-acp/src/acp_adapter.rs` | RuntimeEvent -> CognitiveEvent bridge |
| `crates/roko-acp/src/event_forward.rs` | CognitiveEvent -> RuntimeEvent forwarding |
| `crates/roko-acp/src/builtin_tools.rs` | Slash command dispatch (/fix, /review, /status, etc.) |
| `crates/roko-acp/src/knowledge.rs` | Dispatch-time knowledge retrieval |
| `crates/roko-acp/src/config_watch.rs` | Config hot-reload via notify |
| `crates/roko-mcp-stdio/src/lib.rs` | Shared JSON-RPC 2.0 stdio transport |
| `crates/roko-mcp-code/src/lib.rs` | Code intelligence MCP server |
| `crates/roko-mcp-github/src/main.rs` | GitHub API MCP server |
| `crates/roko-mcp-slack/src/main.rs` | Slack API MCP server |
| `crates/roko-mcp-scripts/src/main.rs` | Script execution MCP server |

### IronClaw Source Code

IronClaw MCP implementation files referenced in this document:

| File | Contents |
|------|----------|
| `src/tools/mcp/mod.rs` | Module root, re-exports, auth error detection |
| `src/tools/mcp/protocol.rs` | McpRequest, McpResponse, McpTool, McpToolAnnotations, ContentBlock, InitializeResult |
| `src/tools/mcp/session.rs` | McpSessionManager, McpSession, McpSessionKey (user+server composite key) |
| `src/tools/mcp/factory.rs` | create_client_from_config -- transport dispatch factory |
| `src/tools/mcp/client.rs` | McpClient with pluggable transports, tool caching, OAuth support |
| `src/tools/mcp/http_transport.rs` | HTTP transport with Mcp-Session-Id capture |
| `src/tools/mcp/stdio_transport.rs` | stdio subprocess transport |
| `src/tools/mcp/unix_transport.rs` | Unix domain socket transport |
| `src/tools/mcp/auth.rs` | OAuth token management and refresh |
| `src/tools/mcp/config.rs` | McpServerConfig, EffectiveTransport, OAuthConfig |
| `src/tools/mcp/client_store.rs` | McpClientStore, per-user client isolation |
| `src/tools/mcp/transport.rs` | McpTransport trait |

### Background

8. **JetBrains ACP Announcement**: [blog.jetbrains.com/ai/2025/12/bring-your-own-ai-agent-to-jetbrains-ides/](https://blog.jetbrains.com/ai/2025/12/bring-your-own-ai-agent-to-jetbrains-ides/). JetBrains' adoption of ACP for IDE integration.

9. **Zed ACP Announcement**: [zed.dev/blog/jetbrains-on-acp](https://zed.dev/blog/jetbrains-on-acp). Zed's perspective on the JetBrains collaboration.

10. **Language Server Protocol**: [microsoft.github.io/language-server-protocol/](https://microsoft.github.io/language-server-protocol/). The predecessor protocol that MCP and ACP draw inspiration from for standardizing editor integrations.
