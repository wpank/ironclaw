# Agent Design Patterns in Roko

**Source crates**: `roko-agent`, `roko-std`, `roko-core`
**Priority**: MEDIUM -- improvements to agent loop design
**Relevant docs**: `docs/v2/05-AGENT.md`, `docs/v2-depth/07-agent-runtime/`

---

## Table of Contents

1. [What This Document Covers](#1-what-this-document-covers)
2. [Why Agent Design Patterns Matter](#2-why-agent-design-patterns-matter)
3. [The Agent Trait -- Core Abstraction](#3-the-agent-trait----core-abstraction)
4. [Pattern 1: Translator (Wire Format Abstraction)](#4-pattern-1-translator-wire-format-abstraction)
5. [Pattern 2: Streaming Event Reassembly](#5-pattern-2-streaming-event-reassembly)
6. [Pattern 3: Resumable Checkpoints](#6-pattern-3-resumable-checkpoints)
7. [Pattern 4: Metacognitive Monitor](#7-pattern-4-metacognitive-monitor)
8. [Pattern 5: Harness Adapter](#8-pattern-5-harness-adapter)
9. [Pattern 6: Composable Scorers](#9-pattern-6-composable-scorers)
10. [Pattern 7: Task Runner and Budget Guardrails](#10-pattern-7-task-runner-and-budget-guardrails)
11. [Pattern 8: Retry Policy with Classified Errors](#11-pattern-8-retry-policy-with-classified-errors)
12. [Pattern 9: Agent Composition Operators](#12-pattern-9-agent-composition-operators)
13. [Pattern 10: Warm Session Reuse and Resume Validation](#13-pattern-10-warm-session-reuse-and-resume-validation)
14. [IronClaw Integration Plan](#14-ironclaw-integration-plan)
15. [Academic References](#15-academic-references)
16. [Complexity Assessment](#16-complexity-assessment)

---

## 1. What This Document Covers

This document catalogs ten reusable design patterns extracted from the roko AI agent runtime. Each pattern addresses a specific challenge in building production-grade LLM-powered agent systems: how to abstract over incompatible provider wire formats, how to reassemble fragmented streaming responses into complete turn records, how to persist and resume long-running multi-step tasks, how to detect when an agent is stuck and intervene automatically, and how to compose smaller agent units into larger pipelines.

Every pattern is documented with its motivation (the concrete problem it solves), the actual Rust types and function signatures verified against the roko source, a worked example showing the pattern in action, and a mapping to IronClaw's existing agent architecture in `src/agent/`.

### Scope and non-scope

**In scope**: Patterns from `roko-agent` and `roko-std` that govern the agent execution lifecycle -- the path from receiving a prompt to producing a final output.

**Not in scope**: Memory and indexing patterns (covered in the roko-index document), orchestration and swarm coordination (covered in the orchestrator-swarm document), provider transport mechanics (covered in the LLM integration document), and tool definition/execution (covered in the tool system document).

### Crate topology

The patterns live across three crates:

| Crate | Role | Key modules |
|-------|------|-------------|
| `roko-core` | Shared primitives: `Signal`, `Context`, `Task`, `ContentHash`, `ToolCall`, `ToolDef` | `signal.rs`, `context.rs`, `task.rs`, `tool/` |
| `roko-agent` | Agent runtime: trait, translators, streaming, checkpoints, introspection, harness, composition, retry, session, task runner | `agent.rs`, `translate/`, `streaming.rs`, `tool_loop/checkpoint.rs`, `introspection.rs`, `harness/`, `composition.rs`, `retry.rs`, `session.rs`, `task_runner.rs` |
| `roko-std` | Standard library of reusable components: scorers, mock agents | `scorer.rs`, `mock.rs` |

---

## 2. Why Agent Design Patterns Matter

Modern LLM agent systems face a combinatorial explosion of provider APIs, transport protocols, streaming formats, error modes, and deployment constraints. Without principled abstractions, each new backend or capability multiplies the codebase linearly. Roko's agent patterns reduce this to a constant-time extension cost by factoring cross-cutting concerns into composable traits:

| Challenge | Without patterns | With patterns |
|-----------|-----------------|---------------|
| New LLM backend | Rewrite tool calling, streaming, retry for each provider | Implement one `Translator`, inherit streaming/retry/checkpoint for free |
| Streaming display | Bespoke SSE parsing per provider | Unified `StreamAccumulator` state machine handles all variants |
| Long task recovery | Lost progress on crash or timeout | `Checkpoint` serializes tool-loop state; resume from last snapshot |
| Stuck detection | Manual monitoring, user complaints | `MetacognitiveMonitor` detects loops and contradictions automatically |
| Multi-agent workflows | Hardcoded orchestration per workflow | `AgentComposition` operators (pipeline, parallel, conditional, mixture) |

The key insight is that all these patterns compose through the `Agent` trait: any `Agent` can be wrapped in any composition operator, retried with any policy, checkpointed at any granularity, and monitored for stuck behavior -- because they all speak the same `Signal -> AgentResult` interface.

---

## 3. The Agent Trait -- Core Abstraction

### Problem

An AI agent system must support many backend implementations (direct LLM calls, subprocess wrappers, HTTP proxies, composed pipelines) behind a single interface. Each implementation is async, non-deterministic, and may produce side effects, so it does not fit into the traditional functional transform trait pattern.

### Design rationale

The `Agent` trait is deliberately minimal: one required method (`run`) plus two optional capabilities (`supports_streaming`, `run_streaming`). This mirrors the "ports and adapters" architectural pattern [Cockburn 2005] where the trait is the port and each implementation is an adapter. The trait's simplicity enables composition -- any wrapper that implements `Agent` can be substituted wherever an agent is expected.

### Source: `roko-agent/src/agent.rs`

```rust
/// An agent: an async executor that takes an input signal (typically a prompt)
/// and produces output signals.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Run the agent against the input signal.
    async fn run(&self, input: &Signal, ctx: &Context) -> AgentResult;

    /// Human-readable name for logs/metrics.
    fn name(&self) -> &str;

    /// Stable backend identifier for audit and episode logging.
    fn backend_id(&self) -> &'static str {
        "unknown"
    }

    /// Does this agent emit a streaming trace?
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Run the agent with streaming output.
    ///
    /// The default implementation falls back to `run()` and emits a single
    /// `ContentDelta` with the full output text, plus a `Usage` chunk
    /// if token counts are non-zero.
    async fn run_streaming(
        &self,
        input: &Signal,
        ctx: &Context,
        event_tx: mpsc::Sender<StreamChunk>,
    ) -> AgentResult {
        let result = self.run(input, ctx).await;
        if let Ok(text) = result.output.body.as_text() {
            if !text.is_empty() {
                let _ = event_tx
                    .send(StreamChunk::ContentDelta(text.to_string()))
                    .await;
            }
        }
        if result.usage.total_tokens() > 0 {
            let _ = event_tx.send(StreamChunk::Usage(result.usage)).await;
        }
        result
    }
}
```

### The AgentResult value type

Every agent run produces an `AgentResult` carrying four fields:

```rust
pub struct AgentResult {
    /// The primary output signal (Kind::AgentOutput).
    pub output: Signal,
    /// Intermediate signals: stream chunks, tool calls, diffs, errors.
    pub trace: Vec<Signal>,
    /// Legacy token usage + cost counters.
    pub usage: Usage,
    /// Canonical usage observation with optional provenance.
    pub usage_obs: Option<UsageObservation>,
    /// Whether the agent ran successfully.
    pub success: bool,
}
```

The `trace` field is ordered chronologically and provides full observability into the agent's internal execution path. The `usage` and `usage_obs` fields support both legacy per-token accounting and the newer structured observation format.

### Worked example: testing an agent

```rust
// Any Agent implementation can be tested against this interface:
let agent = ExecAgent::new("echo", vec![], SafetyLayer::with_defaults());
let prompt = Signal::builder(Kind::Prompt)
    .body(Body::text("hello"))
    .build();
let result = agent.run(&prompt, &Context::now()).await;
assert!(result.success);
assert_eq!(result.output.body.as_text().unwrap(), "hello");
```

---

## 4. Pattern 1: Translator (Wire Format Abstraction)

### Problem

LLM providers use incompatible wire formats for tool calling. OpenAI sends tools as a JSON array in the request body and returns `tool_calls` objects. Anthropic's Claude CLI accepts a `--tools=Read,Edit,Bash` flag and returns stream-json events with `tool_use` blocks. Google Gemini uses `functionDeclarations` / `functionCall` / `functionResponse`. Models without native function calling need the ReAct prompt-level approach with `Action:` / `Observation:` text markers.

Without an abstraction layer, the agent's tool-call logic must branch on provider type at every step: serializing tool definitions, parsing model responses, formatting results for the next turn. Each new provider multiplies this branching.

### Design rationale

The Translator pattern is a structural adapter [Gamma et al. 1995] that converts between a canonical internal representation (roko's `ToolDef`, `ToolCall`, `ToolResult`) and each provider's wire format. Translators are **sync, pure functions** with no I/O and no side effects -- they reshape data without performing network calls. This separation means the async transport layer (HTTP, subprocess, stdio) remains independent of the serialization logic.

Research supports this approach: tool-call format preference is model-specific, with documented accuracy differences of 5-30 percentage points when using the wrong format [Meta-Harness evaluation, WildToolBench, Qwen3-coder format switch measurements].

### Source: `roko-agent/src/translate/mod.rs`

```rust
/// Bidirectional bridge between canonical tools and a backend's wire format.
///
/// Implementors are sync and pure: given identical inputs they must
/// produce identical outputs, and they perform no I/O.
pub trait Translator: Send + Sync {
    /// Which wire format this translator emits/parses.
    fn format(&self) -> ToolFormat;

    /// Serialize the tool catalog into the backend's expected shape.
    fn render_tools(&self, tools: &[ToolDef]) -> RenderedTools;

    /// Parse the backend's response into canonical tool calls.
    fn parse_calls(&self, response: &BackendResponse) -> Result<Vec<ToolCall>, TranslatorError>;

    /// Serialize tool results for the next turn.
    fn render_results(&self, results: &[(ToolCall, ToolResult)]) -> RenderedResults;

    /// Extract the assistant message for conversation history injection.
    fn render_assistant_message(&self, _response: &BackendResponse) -> Option<serde_json::Value> {
        None
    }
}
```

### Payload enums

The trait uses three enums to represent the different serialization shapes:

```rust
/// Backend-specific tool-spec payload.
pub enum RenderedTools {
    /// JSON array for the HTTP body (OpenAI, Ollama, compatible gateways).
    JsonArray(serde_json::Value),
    /// CLI flag payload (e.g. "Read,Edit,Bash") for Claude CLI.
    CliFlag(String),
    /// Text block inlined into the system prompt (ReAct fallback).
    SystemPromptBlock(String),
}

/// Backend-specific tool-result payload for the next turn.
pub enum RenderedResults {
    /// Array of tool-result messages (OpenAI, Ollama).
    JsonMessages(serde_json::Value),
    /// Text to splice into the prompt (ReAct).
    TextBlock(String),
    /// No-op -- backend owns its own tool-call loop (Claude CLI).
    HandledByBackend,
}

/// Raw backend response variants.
pub enum BackendResponse {
    /// Single JSON object (Ollama, OpenAI, Anthropic API non-streaming).
    Json(serde_json::Value),
    /// Sequence of stream-json events (Claude CLI).
    StreamJson(Vec<serde_json::Value>),
    /// Plain-text completion (ReAct models).
    Text(String),
}
```

### Concrete implementations

Roko ships six translator implementations:

| Translator | File | Wire format | Tool delivery | Result format |
|-----------|------|-------------|---------------|---------------|
| `OpenAiTranslator` | `translate/openai.rs` | OpenAI JSON | `JsonArray` | `JsonMessages` |
| `StrictOpenAiTranslator` | `translate/openai.rs` | Strict-mode OpenAI JSON | `JsonArray` | `JsonMessages` |
| `ClaudeTranslator` | `translate/claude.rs` | Claude CLI stream-json | `CliFlag` | `HandledByBackend` |
| `OllamaTranslator` | `translate/ollama.rs` | Ollama `/api/chat` | `JsonArray` | `JsonMessages` |
| `GeminiTranslator` | `translate/gemini.rs` | Gemini `functionDeclarations` | `JsonArray` | `JsonMessages` |
| `ReActTranslator` | `translate/react.rs` | Text-level ReAct markers | `SystemPromptBlock` | `TextBlock` |

### Worked example: ReAct translator round-trip

The ReAct translator is the most interesting because it works entirely at the text level, enabling tool use with models that have no native function-calling support:

```rust
// Source: roko-agent/src/translate/react.rs

// 1. Render tools into a system prompt block
let tools = vec![ToolDef::new("read_file", "Read a file.", ...)];
let rendered = ReActTranslator.render_tools(&tools);
// -> SystemPromptBlock("You have access to the following tools:\n\n### read_file\n...")

// 2. Model generates a plain-text completion with Action/Input markers
let completion = "Thought: I need to look at the file.\n\
                  Action: read_file\n\
                  Action Input: {\"path\": \"src/lib.rs\"}\n\n";

// 3. Parse the completion into a canonical ToolCall
let calls = ReActTranslator.parse_calls(&BackendResponse::Text(completion.into()))?;
assert_eq!(calls[0].name, "read_file");
assert_eq!(calls[0].arguments["path"], "src/lib.rs");
assert_eq!(calls[0].id, "react-0");  // fixed ID for single-call ReAct

// 4. Render the tool result as an "Observation:" text block
let result = ToolResult::text("pub fn main() {}");
let rendered = ReActTranslator.render_results(&[(calls[0].clone(), result)]);
// -> TextBlock("Observation: pub fn main() {}\n\n")
```

**Parsing detail**: the ReAct parser uses `rfind("Action:")` (last occurrence, not first) because earlier `Action:` strings may appear quoted inside the model's reasoning text. Only the trailing occurrence corresponds to the actual action the model is emitting.

### Translator selection

The `translate/capability.rs` module provides automatic translator selection based on model profiles:

```rust
pub fn translator_for(model_name: &str) -> Box<dyn Translator>;
pub fn translator_for_profile(profile: &ModelProfile) -> Box<dyn Translator>;
pub fn capabilities_for(model_name: &str) -> ModelCapabilities;
```

This means new models only need a profile entry -- the system automatically selects the correct translator.

---

## 5. Pattern 2: Streaming Event Reassembly

### Problem

LLM providers deliver responses as fragmented server-sent event (SSE) streams. A single assistant turn may arrive as dozens or hundreds of small delta chunks: incremental text fragments, partial tool-call JSON, reasoning/thinking tokens, usage accounting, and a terminal done marker. The agent runtime needs to reconstruct a complete, well-formed turn record from these fragments for tool dispatch, conversation history injection, and checkpoint serialization.

The fragmentation varies by provider: OpenAI sends `chat.completion.chunk` events with incremental `content` and `tool_calls[index].function.arguments` deltas. Anthropic sends `content_block_start`, `content_block_delta`, and `message_stop` events. The reassembly logic must handle all variants without provider-specific branching in the agent loop.

### Design rationale

The `StreamAccumulator` implements a state machine that reduces any `StreamChunk` sequence into a complete `BackendResponse`. This is the "accumulator" or "reducer" pattern common in event-stream processing [Ousterhout 2018, Chapter 7]. The accumulator is append-only -- each `push` is idempotent with respect to ordering -- which makes it safe to use in concurrent streaming scenarios.

### Source: `roko-agent/src/streaming.rs`

#### The stream chunk vocabulary

```rust
/// One incremental piece of a streaming LLM response.
pub enum StreamChunk {
    /// Incremental reasoning or thinking text.
    ReasoningDelta(String),
    /// Incremental assistant-visible content text.
    ContentDelta(String),
    /// Incremental function-call data for one tool call slot.
    ToolCallDelta {
        /// Zero-based tool call index within the current assistant turn.
        index: usize,
        /// Incremental tool call identifier fragment.
        id_delta: Option<String>,
        /// Incremental function name fragment.
        name_delta: Option<String>,
        /// Incremental JSON argument text.
        arguments_delta: String,
    },
    /// Token accounting emitted during or after the stream.
    Usage(Usage),
    /// Terminal stream marker with the canonical finish reason.
    Done(FinishReason),
    /// Terminal provider or transport error surfaced as a stream event.
    Error(String),
    /// Progress update from a running tool (harness adapters).
    ToolProgress {
        /// Tool name or identifier.
        tool: String,
        /// Human-readable progress status.
        status: String,
    },
}
```

#### The accumulator state machine

```rust
pub struct StreamAccumulator {
    reasoning: String,
    content: String,
    tool_calls: Vec<PartialToolCall>,
    usage: Usage,
    finish_reason: FinishReason,
}

#[derive(Debug, Clone, Default)]
struct PartialToolCall {
    id: String,
    name: String,
    arguments: String,
}

impl StreamAccumulator {
    pub fn new() -> Self { Self::default() }

    /// Incorporate one streamed chunk into the in-progress response.
    pub fn push(&mut self, chunk: StreamChunk) {
        match chunk {
            StreamChunk::ReasoningDelta(delta) => self.reasoning.push_str(&delta),
            StreamChunk::ContentDelta(delta) => self.content.push_str(&delta),
            StreamChunk::ToolCallDelta { index, id_delta, name_delta, arguments_delta } => {
                // Auto-extend the tool_calls vec to accommodate the index
                while self.tool_calls.len() <= index {
                    self.tool_calls.push(PartialToolCall::default());
                }
                let tool_call = &mut self.tool_calls[index];
                if let Some(id) = id_delta {
                    tool_call.id = id;
                }
                if let Some(name) = name_delta {
                    tool_call.name = name;
                }
                tool_call.arguments.push_str(&arguments_delta);
            }
            StreamChunk::Usage(usage) => self.usage = usage,
            StreamChunk::Done(finish_reason) => {
                // Preserve a more specific finish reason if we already have one
                let should_preserve_existing = matches!(finish_reason, FinishReason::Stop)
                    && !matches!(self.finish_reason, FinishReason::Stop);
                if !should_preserve_existing {
                    self.finish_reason = finish_reason;
                }
            }
            StreamChunk::Error(_) => {}
            StreamChunk::ToolProgress { .. } => {
                // Informational; does not affect the accumulated response.
            }
        }
    }
}
```

### State machine transitions

The accumulator's state evolves through these transitions:

```
                                     +-- push(ReasoningDelta) --> reasoning.push_str()
                                     |
                                     +-- push(ContentDelta) ----> content.push_str()
                                     |
[Empty] -- push(chunk) -----------> [Accumulating] ----------+
                                     |                       |
                                     +-- push(ToolCallDelta)  |
                                     |   auto-extend vec,     |
                                     |   append id/name/args  |
                                     |                        |
                                     +-- push(Usage) --------> usage = new_usage
                                     |
                                     +-- push(Done(reason)) -> finish_reason = reason
                                     |                         (unless more-specific already set)
                                     |
                                     +-- push(Error) --------> no-op (logged externally)
                                     |
                                     +-- push(ToolProgress) -> no-op (informational)
```

### Tool call reassembly detail

Tool call deltas arrive indexed. A single assistant turn may emit multiple parallel tool calls (e.g., reading two files simultaneously). The accumulator tracks them independently:

```
Stream:
  ToolCallDelta { index: 0, id_delta: Some("call_1"), name_delta: Some("read_file"), arguments_delta: "{\"" }
  ToolCallDelta { index: 1, id_delta: Some("call_2"), name_delta: Some("list_dir"), arguments_delta: "{\"" }
  ToolCallDelta { index: 0, arguments_delta: "path\":\"src/main.rs\"}" }
  ToolCallDelta { index: 1, arguments_delta: "path\":\".\"}" }
  Done(ToolCalls)

Accumulator state after all chunks:
  tool_calls[0] = { id: "call_1", name: "read_file", arguments: "{\"path\":\"src/main.rs\"}" }
  tool_calls[1] = { id: "call_2", name: "list_dir", arguments: "{\"path\":\".\"}" }
```

### Finish reason precedence

The `Done` handler has a subtle priority rule: if the accumulator already holds a more specific finish reason (e.g., `FinishReason::ToolCalls` from a prior chunk), a subsequent `Done(Stop)` does not overwrite it. This handles the case where some providers send a generic stop event after already signaling tool use.

### Worked example: streaming an entire conversation turn

```rust
let (tx, mut rx) = mpsc::channel(64);
let mut accumulator = StreamAccumulator::new();

// Agent streams chunks into the channel
let handle = tokio::spawn(async move {
    agent.run_streaming(&prompt, &ctx, tx).await
});

// Consumer reassembles the complete response
while let Some(chunk) = rx.recv().await {
    // Forward to UI for real-time display
    display.push_chunk(&chunk);
    // Accumulate for the final turn record
    accumulator.push(chunk);
}

let result = handle.await?;
// accumulator now holds the complete response for history/checkpoint
```

---

## 6. Pattern 3: Resumable Checkpoints

### Problem

LLM-powered tool loops can run for minutes or hours -- reading files, editing code, running tests, iterating on errors. A crash, timeout, or context-window rotation during a long tool loop loses all accumulated progress: the conversation history, the tool calls dispatched so far, and the provider session state needed to continue the conversation.

This is the classic checkpoint-restart problem from high-performance computing [Elnozahy et al. 2002], adapted for the LLM agent setting where "process state" includes not just computational variables but the entire conversation transcript and provider session metadata.

### Design rationale

The `Checkpoint` type captures the minimal state needed to resume a tool loop: iteration count (so the budget is correctly decremented), accumulated tool calls (so the agent knows what has been done), conversation messages (so the provider can continue the conversation), and provider session identifiers (for providers that support session continuity).

The checkpoint is serialized as JSON and persisted to disk. On resume, the tool loop loads the checkpoint, validates it against the current configuration (model, backend, prompt fingerprint), and continues from the saved iteration.

### Source: `roko-agent/src/tool_loop/checkpoint.rs`

```rust
/// Serializable snapshot of a ToolLoop mid-execution.
///
/// Created by the loop when it stops for any reason other than
/// StopReason::Stop (the normal "final answer" path).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Number of tool-call iterations completed before this snapshot.
    pub iterations: usize,
    /// All tool calls dispatched so far (across all iterations).
    pub tool_calls: Vec<ToolCall>,
    /// The full conversation message history at snapshot time.
    pub messages: Vec<serde_json::Value>,
    /// Provider-issued session identifiers required to resume the conversation.
    #[serde(default)]
    pub session: SessionState,
}

impl Checkpoint {
    pub fn new(
        iterations: usize,
        tool_calls: Vec<ToolCall>,
        messages: Vec<serde_json::Value>,
    ) -> Self { /* ... */ }

    /// Attach provider session continuity state.
    pub fn with_session(mut self, session: SessionState) -> Self { /* ... */ }

    /// Serialize to JSON bytes for persistence.
    pub fn to_bytes(&self) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON bytes.
    pub fn from_bytes(bytes: &[u8]) -> serde_json::Result<Self> {
        serde_json::from_slice(bytes)
    }

    /// Persist to disk as formatted JSON (creates parent directories).
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let cp: Self = serde_json::from_str(&json)?;
        Ok(cp)
    }
}
```

### Checkpoint lifecycle

```
Tool Loop Running
  |
  v
[Iteration N completes]
  |
  +-- StopReason::Stop (final answer) --> No checkpoint, return AgentResult
  |
  +-- StopReason::MaxIterations --------> Checkpoint::new(N, calls, messages).save()
  |                                       Return with checkpoint path
  +-- StopReason::Timeout --------------> Checkpoint::new(N, calls, messages).save()
  |
  +-- StopReason::Error ----------------> Checkpoint::new(N, calls, messages).save()
  |
  v
[Later: Resume requested]
  |
  v
Checkpoint::load(path)?
  |
  v
validate_resume_request(persisted, requested)?
  |                      |
  | Ok                   | Err(mismatch)
  v                      v
Resume from iteration N   Reject: start fresh
```

### Worked example: checkpoint round-trip

```rust
// Create checkpoint during tool loop
let call = ToolCall::new("c1", "echo", json!({"x": 1}));
let cp = Checkpoint::new(
    3,  // 3 iterations completed
    vec![call],
    vec![
        json!({"role": "system", "content": "sys"}),
        json!({"role": "user", "content": "usr"}),
    ],
);

// Persist to disk
let dir = tempfile::tempdir()?;
let path = dir.path().join("state").join("checkpoint.json");
cp.save(&path)?;

// Resume later
let loaded = Checkpoint::load(&path)?;
assert_eq!(loaded.iterations, 3);
assert_eq!(loaded.tool_calls[0].name, "echo");
assert_eq!(loaded.messages.len(), 2);
```

---

## 7. Pattern 4: Metacognitive Monitor

### Problem

LLM agents can get stuck in loops -- calling the same tool with the same arguments repeatedly, contradicting themselves across turns ("this works" followed by "this won't work"), or producing outputs with declining confidence. Without automatic detection, these failure modes consume budget and time until the iteration limit is reached.

This pattern draws on metacognition research in AI -- the idea that an intelligent system can monitor its own reasoning process and intervene when it detects dysfunction [Flavell 1979; Cox 2005]. Recent work on LLM self-monitoring and calibration [MIRROR benchmark, Xie et al. 2024] confirms that explicit metacognitive scaffolding improves agent reliability.

### Design rationale

The `MetacognitiveMonitor` inspects the recent turn history and fires one of four interventions when it detects a problem. It operates on three detection channels:

1. **Repetition detection**: Fingerprints tool calls as `"name:arguments"` strings and checks if the last N fingerprints are identical (indicating a stuck loop).
2. **Contradiction detection**: Scans recent turn text for co-occurring positive commitments ("possible", "confirmed", "works") and negative commitments ("impossible", "won't work", "fails") within a sliding window.
3. **Confidence thresholding**: Extracts model-reported confidence scores from responses and triggers escalation or human handoff when they drop below configurable thresholds.

### Source: `roko-agent/src/introspection.rs`

```rust
/// A single tool / reasoning turn observed by the metacognitive monitor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Turn {
    /// Turn index in the current run.
    pub index: usize,
    /// Assistant text returned for this turn.
    pub assistant_text: String,
    /// Reasoning / thinking content, if any.
    pub reasoning: Option<String>,
    /// Tool calls emitted in the turn.
    pub tool_calls: Vec<ToolCall>,
    /// Optional model confidence in the turn.
    pub confidence: Option<f32>,
}

/// Intervention suggested by the monitor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intervention {
    /// Escalate to a larger / more capable model.
    EscalateModel,
    /// Route the current task to a human.
    HumanHandoff,
    /// Abort the current run.
    Abort,
    /// Inject a reflection prompt and continue.
    InjectReflection(String),
}

/// Simple metacognitive monitor with configurable thresholds.
#[derive(Debug, Clone)]
pub struct MetacognitiveMonitor {
    /// Number of repeated tool-call fingerprints that indicate a stuck loop.
    pub repeat_threshold: usize,            // default: 3
    /// Sliding-window size for contradiction detection.
    pub contradiction_window: usize,        // default: 4
    /// Confidence below this triggers escalation.
    pub confidence_threshold: f32,          // default: 0.35
    /// Confidence below this triggers human handoff.
    pub human_handoff_threshold: f32,       // default: 0.15
}
```

### Detection priority chain

The `check()` method evaluates detection channels in priority order, returning the first intervention that fires:

```rust
impl MetacognitiveMonitor {
    pub fn check(&self, turns: &[Turn]) -> Option<Intervention> {
        if turns.is_empty() { return None; }

        // Priority 1: stuck loop (most actionable -- inject reflection)
        if self.repeated_tool_calls(turns) {
            return Some(Intervention::InjectReflection(
                "the same tool call is repeating; pause, inspect the result, \
                 and reconcile the plan".into(),
            ));
        }

        // Priority 2: self-contradiction (inject reflection to restate)
        if self.contradiction_detected(turns) {
            return Some(Intervention::InjectReflection(
                "the recent turns contradict each other; restate the current \
                 state before continuing".into(),
            ));
        }

        // Priority 3: low confidence (escalate or hand off)
        if let Some(confidence) = turns.last().and_then(|turn| turn.confidence) {
            if confidence < self.human_handoff_threshold {
                return Some(Intervention::HumanHandoff);
            }
            if confidence < self.confidence_threshold {
                return Some(Intervention::EscalateModel);
            }
        }

        None
    }
}
```

### Repetition detection algorithm

Tool calls are fingerprinted as `"name:arguments"` strings. The detector collects the last `repeat_threshold` fingerprints from the most recent turns and checks if they are all identical:

```rust
fn repeated_tool_calls(&self, turns: &[Turn]) -> bool {
    let fingerprints: Vec<String> = turns
        .iter()
        .rev()
        .flat_map(|turn| turn.tool_fingerprints())
        .take(self.repeat_threshold)
        .collect();
    if fingerprints.len() < self.repeat_threshold || fingerprints.is_empty() {
        return false;
    }
    fingerprints.windows(2).all(|pair| pair[0] == pair[1])
}
```

### Contradiction detection algorithm

The contradiction detector scans text for co-occurring positive and negative commitment phrases within the `contradiction_window`:

```rust
fn contradiction_detected(&self, turns: &[Turn]) -> bool {
    let recent = turns.iter().rev().take(self.contradiction_window);
    let mut saw_positive = false;
    let mut saw_negative = false;

    for turn in recent {
        let text = format!("{} {}", turn.assistant_text,
                           turn.reasoning.as_deref().unwrap_or(""))
            .to_lowercase();
        if contains_positive_commitment(&text) { saw_positive = true; }
        if contains_negative_commitment(&text) { saw_negative = true; }
        if saw_positive && saw_negative { return true; }
    }
    false
}
```

The commitment vocabularies are deliberately small and high-precision:
- **Positive**: "i can", "we can", "possible", "works", "feasible", "confirmed"
- **Negative**: "i can't", "cannot", "not possible", "won't work", "fails", "impossible"

### Confidence extraction

The monitor extracts confidence from backend responses by probing a sequence of JSON paths:

```rust
fn extract_confidence(response: &BackendResponse) -> Option<f32> {
    // Probes: "confidence", "confidence_score", "score",
    //         "message.confidence", "choices.0.message.confidence"
    // ...
}
```

This supports OpenAI-compatible, Anthropic, and custom providers that report confidence in different locations.

### Worked example: detecting a stuck loop

```rust
let monitor = MetacognitiveMonitor::default(); // repeat_threshold = 3

let stuck_turn = Turn {
    index: 1,
    assistant_text: String::new(),
    reasoning: None,
    tool_calls: vec![ToolCall::new("1", "read_file", json!({"path": "x"}))],
    confidence: Some(0.9),
};

// Same tool call repeated 3 times
let turns = vec![stuck_turn.clone(), stuck_turn.clone(), stuck_turn];
match monitor.check(&turns) {
    Some(Intervention::InjectReflection(msg)) => {
        assert!(msg.contains("same tool call is repeating"));
    }
    other => panic!("expected InjectReflection, got {:?}", other),
}
```

### Worked example: confidence-triggered escalation

```rust
let monitor = MetacognitiveMonitor::default(); // confidence_threshold = 0.35

let low_confidence_turn = Turn {
    index: 1,
    assistant_text: "answer".into(),
    reasoning: None,
    tool_calls: Vec::new(),
    confidence: Some(0.2),  // below 0.35 but above 0.15
};

match monitor.check(&[low_confidence_turn]) {
    Some(Intervention::EscalateModel) => { /* correct */ }
    other => panic!("expected EscalateModel, got {:?}", other),
}
```

### Agent identity (companion type)

The `introspection.rs` module also provides `AgentIdentity`, a lightweight snapshot of the current agent's role, model tier, temperament, and capability mask:

```rust
pub struct AgentIdentity {
    pub role: AgentRole,
    pub model_tier: ModelTier,
    pub temperament: Temperament,
    pub capabilities: ToolPermissions,
}
```

This enables role-aware metacognition: an `Implementer` with `Temperament::Balanced` might have different confidence thresholds than a `Reviewer` with `Temperament::Cautious`.

---

## 8. Pattern 5: Harness Adapter

### Problem

The roko system needs to delegate work to external agent runtimes -- Claude CLI, Cursor, Hermes, OpenClaw -- each with its own transport protocol (HTTP, CLI subprocess, ACP over stdio, MCP server mode), capability set (streaming, tool injection, session resume, cancellation), and lifecycle management. A naive implementation would require a separate integration path for each harness-transport combination, creating an M x N explosion.

### Design rationale

The harness adapter pattern factors this into three orthogonal concerns:

1. **`HarnessAdapter` trait**: extends `Agent` with harness-specific metadata (transport, capabilities, probe). This is the classic GoF Adapter pattern [Gamma et al. 1995] applied to agent runtimes.
2. **`HarnessCapabilities` struct**: static description of what a harness can do at a given transport. Enables dispatch-time capability negotiation.
3. **`validate_for_task()`**: pre-dispatch validation that checks capability-task compatibility and produces actionable error messages.

### Source: `roko-agent/src/harness/mod.rs` and `roko-agent/src/harness/capability.rs`

#### The HarnessAdapter trait

```rust
/// A harness adapter is an Agent that wraps an external long-lived process.
#[async_trait]
pub trait HarnessAdapter: Agent {
    /// Stable identifier (appears in roko.toml and episode logs).
    fn harness_id(&self) -> &str;

    /// Which transport this adapter speaks.
    fn transport(&self) -> TransportFlavor;

    /// Static description of capabilities at this transport.
    fn capabilities(&self) -> &HarnessCapabilities;

    /// Cheap install / auth / version probe (~250ms on healthy installs).
    async fn probe(&self) -> Result<(), ProbeError>;

    /// Where the harness stores its state on disk.
    fn state_dir(&self) -> Option<&Path> { None }

    /// Lifecycle manager for the underlying daemon, if any.
    fn service(&self) -> Option<&dyn HarnessService> { None }
}
```

#### Transport flavors (7-tier hierarchy)

```rust
pub enum TransportFlavor {
    HttpOpenAi,      // Tier 1: OpenAI-compatible HTTP API
    HttpResponses,   // Tier 1: OpenAI Responses API over HTTP
    OneShotJson,     // Tier 2: CLI with JSON envelope on stdout
    OneShotPlain,    // Tier 2: CLI with plain-text output
    AcpStdio,        // Tier 3: ACP over stdio
    TuiJsonRpc,      // Tier 4: TUI JSON-RPC over stdio (deferred to v2)
    McpServer,       // Tier 5: MCP server mode
}
```

#### Capability negotiation

The `HarnessCapabilities` struct declares what a harness can do at its transport level:

```rust
pub struct HarnessCapabilities {
    pub one_shot: OneShotMode,           // How prompts are delivered
    pub streaming: StreamingMode,         // SSE, NDJSON, raw tokens, or none
    pub session_resume: SessionResumeMode, // PreviousResponseId, CliFlag, Acp, or none
    pub mcp_passthrough: McpMode,         // PerCall, ConfigFile, ServerOnly, or none
    pub tool_injection: ToolInjection,    // PerCallTools, ConfigFile, McpOnly, or Opaque
    pub model_override: bool,             // Can the caller override the model per request?
    pub multiplex_safe: bool,             // Can multiple agents share this adapter?
    pub cancel: CancelMode,              // HttpEndpoint, KillChild, AcpCancel, or none
    pub overhead_p50_ms: u32,            // Approximate p50 latency overhead
}
```

#### Pre-dispatch validation

```rust
pub fn validate_for_task(
    adapter: &dyn HarnessAdapter,
    task: &HarnessTaskRequirements,
) -> Result<(), CapabilityMismatch> {
    let c = adapter.capabilities();

    // Check: tools needed but no injection path available
    if task.needs_tools
        && matches!(c.tool_injection, ToolInjection::Opaque)
        && matches!(c.mcp_passthrough, McpMode::None | McpMode::ServerOnly)
    {
        return Err(CapabilityMismatch {
            adapter: adapter.harness_id().to_string(),
            transport: adapter.transport(),
            need: "tools",
            hint: "use a transport with PerCallTools, ConfigFile, or McpOnly tool injection",
        });
    }

    // Check: streaming needed but not available
    if task.needs_streaming && matches!(c.streaming, StreamingMode::None) {
        return Err(CapabilityMismatch { /* ... */ });
    }

    // ... MCP, session resume, cancel, PTY checks follow the same pattern
    Ok(())
}
```

### Validation constraint matrix

| Task requirement | Capability check | Mismatch hint |
|-----------------|-----------------|---------------|
| `needs_tools` | `tool_injection != Opaque` OR `mcp_passthrough` is `PerCall`/`ConfigFile` | Use a transport with PerCallTools, ConfigFile, or McpOnly |
| `needs_streaming` | `streaming != None` | Use a transport with SseChatCompletions or NdJson |
| `needs_mcp` | `mcp_passthrough` is `PerCall` or `ConfigFile` | Use a transport that supports per-call or config-file MCP injection |
| `needs_session_resume` | `session_resume != None` | Use a transport with PreviousResponseId, CliFlag, or Acp |
| `needs_cancel` | `cancel != None` | Use a transport with HttpEndpoint, KillChild, or AcpCancel |
| PTY overhead | `one_shot != PtyAutomation` OR `allows_pty_overhead` | Set allows_pty_overhead = true or use a different transport |

### Worked example: capability-driven dispatch rejection

```rust
// Conservative adapter with default (unsupported) capabilities
let adapter = HermesOneShotAdapter::default(); // TransportFlavor::OneShotPlain

// Task that requires streaming
let task = HarnessTaskRequirements {
    needs_streaming: true,
    ..Default::default()
};

match validate_for_task(&adapter, &task) {
    Err(mismatch) => {
        assert_eq!(mismatch.need, "streaming");
        assert_eq!(mismatch.adapter, "hermes");
        assert!(mismatch.hint.contains("SseChatCompletions"));
        // Orchestrator routes to a different adapter
    }
    Ok(()) => panic!("should have rejected"),
}
```

### Supporting types

The module also provides:

- **`HarnessService`**: Lifecycle manager for daemon processes (start/stop/status/healthcheck).
- **`HarnessRegistry`**: Central lookup + probe cache for all registered harnesses.
- **`ChildProcessRunner`**: Shared subprocess lifecycle manager with `ScrubbedEnv` for safe credential handling.
- **`EventParser` / `HarnessEvent`**: Protocol-specific line parsing for subprocess output.
- **`AcpStdioClient`**: ACP (Agent Communication Protocol) client for stdio-based harnesses.

---

## 9. Pattern 6: Composable Scorers

### Problem

AI agent systems need to rank and filter items (search results, memory entries, candidate responses) by multiple independent criteria: relevance, recency, reputation, confidence. Building monolithic scoring functions creates rigid, hard-to-test code. Adding a new scoring dimension requires modifying the existing scorer.

### Design rationale

The scorer composition pattern uses two algebraic operators -- addition and multiplication -- to combine independent scoring functions. This follows the principle of separating concerns into independently testable units that compose through well-defined algebraic properties [Wadler 1992].

- **Sum** (`+`): aggregates evidence from multiple sources. If scorer A gives 0.3 and scorer B gives 0.4, the sum is 0.7. Use when each scorer contributes independent evidence toward a single verdict.
- **Multiply** (`*`): scales each dimension independently. If relevance is 0.9 and recency is 0.5, the product is 0.45. Use when each scorer acts as a gate -- a zero in any dimension zeros the whole score.

### Source: `roko-std/src/scorer.rs`

```rust
use roko_core::traits::Score as ScoreFn;
use roko_core::{Context, Engram, Score};

/// Sum several scorers element-wise (aggregates evidence).
pub struct SumScorer {
    scorers: Vec<Box<dyn ScoreFn>>,
    name: String,
}

impl ScoreFn for SumScorer {
    fn score(&self, signal: &Engram, ctx: &Context) -> Score {
        self.scorers
            .iter()
            .fold(Score::ZERO, |acc, s| acc + s.score(signal, ctx))
    }
    fn name(&self) -> &'static str { "sum_scorer" }
}

/// Multiply several scorers element-wise (scales each axis).
pub struct MulScorer {
    scorers: Vec<Box<dyn ScoreFn>>,
    name: String,
}

impl ScoreFn for MulScorer {
    fn score(&self, signal: &Engram, ctx: &Context) -> Score {
        let one = Score::new(1.0, 1.0, 1.0, 1.0);
        self.scorers
            .iter()
            .fold(one, |acc, s| acc * s.score(signal, ctx))
    }
    fn name(&self) -> &'static str { "mul_scorer" }
}

/// Returns a fixed score for every signal. Useful for static weighting.
pub struct ConstScorer {
    value: Score,
}
```

### Score type

The `Score` type is a 4-dimensional vector with named fields: `confidence`, plus three application-defined axes. Operations (`+`, `*`) are element-wise.

### Composition algebra

```
SumScorer([A, B])  =>  A(signal) + B(signal)     (element-wise addition)
MulScorer([A, B])  =>  A(signal) * B(signal)     (element-wise multiplication)

// Nesting is natural:
MulScorer([
    SumScorer([relevance, recency]),    // evidence aggregation
    ConstScorer(0.8, 1.0, 1.0, 1.0),   // static weight
])
```

### Worked example: ranking search results

```rust
// Overall score = relevance x recency x reputation
let scorer = MulScorer::new(vec![
    Box::new(RelevanceScorer::new(query)),
    Box::new(RecencyScorer),
    Box::new(ReputationScorer),
]);

let results: Vec<(Engram, Score)> = candidates
    .iter()
    .map(|c| (c.clone(), scorer.score(c, &ctx)))
    .collect();

// Sort by confidence axis descending
results.sort_by(|a, b| b.1.confidence.partial_cmp(&a.1.confidence).unwrap());
```

### Identity elements

- `SumScorer` with zero scorers returns `Score::ZERO` (additive identity).
- `MulScorer` with zero scorers returns `Score::new(1.0, 1.0, 1.0, 1.0)` (multiplicative identity).

This means empty compositions are safe to use and behave correctly.

---

## 10. Pattern 7: Task Runner and Budget Guardrails

### Problem

Agent tasks need a coordination layer that wraps the raw agent execution with event broadcasting, anomaly detection, budget enforcement, cost accounting, and conductor-driven model escalation. Without this, each caller must independently implement these cross-cutting concerns.

### Design rationale

The `TaskRunner` is the composition point for the task execution pipeline. It owns the agent and its support infrastructure as a single unit, ensuring that every task iteration passes through the same pipeline of checks and accounting.

### Source: `roko-agent/src/task_runner.rs`

```rust
pub struct TaskRunner {
    /// The task-facing agent implementation.
    pub agent: Box<dyn Agent>,
    /// Event stream publisher for runtime feedback.
    pub event_bus: EventBus,
    /// Session-local anomaly detector.
    pub anomaly: AnomalyDetector,
    /// Budget guardrail applied across task iterations.
    pub budget: BudgetGuardrail,
    /// Learned conductor policy for intervention decisions.
    pub conductor: ConductorBandit,
    /// Pricing table for cost computation.
    pub cost_table: CostTable,
    /// Requested model slug.
    pub model_slug: String,
    /// Provider identifier.
    pub provider_id: String,
    /// Maximum task-loop iterations before aborting.
    pub max_iterations: u32,
}
```

### Task result

```rust
pub struct TaskResult {
    pub output: Signal,
    pub total_usage: Usage,
    pub total_cost_usd: f64,
    pub iterations: u32,
    pub gate_passed: bool,
    pub conductor_actions: Vec<ConductorAction>,
}
```

### Anomaly detection

The `AnomalyDetector` uses a sliding-window prompt-hash approach to detect prompt loops:

```rust
const PROMPT_LOOP_WINDOW: usize = 20;
const PROMPT_LOOP_THRESHOLD: usize = 5;

pub struct AnomalyDetector {
    prompt_hash_window: VecDeque<u64>,
    session_start_ms: i64,
}

impl AnomalyDetector {
    /// Check a prompt hash for repeated-loop behavior.
    pub fn check_prompt(&mut self, prompt_hash: u64) -> Option<Anomaly> {
        self.prompt_hash_window.push_back(prompt_hash);
        if self.prompt_hash_window.len() > PROMPT_LOOP_WINDOW {
            self.prompt_hash_window.pop_front();
        }
        let repeated_count = self.prompt_hash_window
            .iter()
            .filter(|&&hash| hash == prompt_hash)
            .count();
        (repeated_count >= PROMPT_LOOP_THRESHOLD)
            .then_some(Anomaly::PromptLoop { repeated_count })
    }
}
```

This detects when the same prompt hash appears 5 or more times in the last 20 entries, indicating the agent is looping.

### Event bus

The `EventBus` uses a `tokio::broadcast` channel to publish runtime events:

```rust
pub enum AgentEvent {
    TurnStarted { task_id, model, provider, timestamp_ms },
    TurnCompleted { turn, usage, tool_call_count, gate_passed, finish_reason },
    CostRecorded { model, provider, cost_usd, tokens },
    AnomalyDetected { anomaly },
}
```

### Task runner errors

```rust
pub enum TaskRunnerError {
    BudgetExhausted,
    Anomaly(Anomaly),
    ModelEscalation,
}
```

### Worked example: task execution with anomaly detection

```rust
let mut runner = TaskRunner {
    agent: Box::new(my_agent),
    event_bus: EventBus::new(128),
    anomaly: AnomalyDetector::new(now_ms),
    budget: BudgetGuardrail::new(10.0),  // $10 per-task limit
    conductor: ConductorBandit::default(),
    cost_table: CostTable::default(),
    model_slug: "claude-sonnet-4-20250514".into(),
    provider_id: "anthropic".into(),
    max_iterations: 50,
};

// Subscribe to events for monitoring
let mut rx = runner.event_bus.subscribe();
tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        match event {
            AgentEvent::AnomalyDetected { anomaly } => {
                warn!("anomaly: {:?}", anomaly);
            }
            AgentEvent::CostRecorded { cost_usd, .. } => {
                metrics::gauge!("task_cost_usd", cost_usd);
            }
            _ => {}
        }
    }
});
```

---

## 11. Pattern 8: Retry Policy with Classified Errors

### Problem

LLM provider calls fail in diverse ways: rate limiting (429), authentication failures (401), timeouts, server errors (5xx), content policy violations, context overflow, and model-not-found errors. Each error class requires a different retry strategy: rate limits should back off exponentially, authentication failures should not retry at all, and timeouts should retry with moderate delay.

### Design rationale

The retry policy separates error classification from backoff computation, following the AWS full-jitter exponential backoff pattern [Brooker 2015]. The key insight from Amazon's analysis is that naive exponential backoff without jitter causes "thundering herd" problems -- all clients that were rate-limited simultaneously retry at the same time, causing another rate-limit spike. Full jitter randomizes each client's retry schedule to decorrelate the herd.

### Source: `roko-agent/src/retry.rs`

#### Error classification

```rust
/// Canonical error classes used by retry policy configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    RateLimit,
    AuthFailure,
    Timeout,
    ServerError,
    ContentPolicy,
    ContextOverflow,
    ModelNotFound,
    Unknown,
}

impl From<&ProviderError> for ErrorClass {
    fn from(error: &ProviderError) -> Self {
        match error {
            ProviderError::RateLimit { .. } => Self::RateLimit,
            ProviderError::AuthFailure => Self::AuthFailure,
            ProviderError::Timeout => Self::Timeout,
            ProviderError::ServerError(_) => Self::ServerError,
            ProviderError::ContentPolicy => Self::ContentPolicy,
            ProviderError::ContextOverflow => Self::ContextOverflow,
            ProviderError::ModelNotFound => Self::ModelNotFound,
            ProviderError::Other(_) => Self::Unknown,
        }
    }
}
```

#### Retry policy with full-jitter backoff

```rust
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub retryable_errors: Vec<ErrorClass>,
}
```

#### Backoff formula

The delay for attempt `n` is computed as:

```
exp_delay = base_delay_ms * 2^min(n, 10)     // exponential growth, capped at 2^10
capped    = min(exp_delay, max_delay_ms)      // ceiling
floor     = base_delay_ms / 2                 // guaranteed minimum
delay     = random(floor ..= capped)          // full jitter
```

In code:

```rust
fn delay_for_attempt_with_rng<R: Rng + ?Sized>(&self, attempt: u32, rng: &mut R) -> u64 {
    let exp_delay = self.base_delay_ms.saturating_mul(1u64 << attempt.min(10));
    let capped = exp_delay.min(self.max_delay_ms);
    // Guarantee a minimum floor of half the base delay so jitter never
    // produces a near-zero wait (critical for rate-limit backoff).
    let floor = self.base_delay_ms / 2;
    if capped <= floor {
        return capped;
    }
    rng.gen_range(floor..=capped)
}
```

The floor of `base_delay_ms / 2` is an important safety measure: standard full-jitter (`random(0..capped)`) can produce near-zero delays, which defeats the purpose of backoff for rate-limit errors. The floor ensures a meaningful minimum wait.

#### Retryability classification

```rust
pub fn should_retry(&self, error: &ProviderError, attempt: u32) -> bool {
    if attempt >= self.max_attempts { return false; }

    match error {
        ProviderError::RateLimit { .. } => true,     // always retry
        ProviderError::AuthFailure => false,          // never retry
        ProviderError::ContentPolicy => false,        // never retry
        ProviderError::Timeout => true,               // transient
        ProviderError::ServerError(_) => true,        // transient
        ProviderError::ContextOverflow => false,      // structural
        _ => attempt < 2,                             // unknown: retry once
    }
}
```

#### Rate-limit-specific policy

```rust
impl RetryPolicy {
    pub fn for_rate_limit() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 2_000,   // DEFAULT_RATE_LIMIT_RETRY_BASE_DELAY_MS
            max_delay_ms: 60_000,   // DEFAULT_RATE_LIMIT_RETRY_MAX_BACKOFF_MS
            retryable_errors: vec![ErrorClass::RateLimit],
        }
    }
}
```

This produces the following delay distribution:

| Attempt | Exponential delay | Capped | Floor | Jitter range |
|---------|-------------------|--------|-------|-------------|
| 0 | 2,000 ms | 2,000 ms | 1,000 ms | [1,000 - 2,000] ms |
| 1 | 4,000 ms | 4,000 ms | 1,000 ms | [1,000 - 4,000] ms |
| 2 | 8,000 ms | 8,000 ms | 1,000 ms | [1,000 - 8,000] ms |
| 3 | (max_attempts reached) | -- | -- | no retry |

#### Provider Retry-After hint

When the provider includes a `Retry-After` header (converted to milliseconds), the policy respects it but enforces the floor:

```rust
pub fn rate_limit_delay(&self, attempt: u32, retry_after_ms: Option<u64>) -> u64 {
    if let Some(provider_hint) = retry_after_ms {
        return provider_hint.max(self.base_delay_ms / 2);
    }
    self.delay_for_attempt(attempt)
}
```

### Worked example: retry with provider hint

```rust
let policy = RetryPolicy::for_rate_limit();

// Provider says wait 5 seconds
assert_eq!(policy.rate_limit_delay(0, Some(5_000)), 5_000);

// Provider says wait 500ms -- but floor is 1000ms, so floor wins
assert_eq!(policy.rate_limit_delay(0, Some(500)), 1_000);

// No provider hint -- fall back to jittered exponential
let delay = policy.rate_limit_delay(0, None);
assert!(delay >= 1_000 && delay <= 2_000);
```

---

## 12. Pattern 9: Agent Composition Operators

### Problem

Complex AI workflows require combining multiple agents: running them in sequence (pipeline), in parallel (fan-out/merge), conditionally (routing by task type), or as a mixture-of-agents where candidates propose and an aggregator synthesizes. Without structured composition, each workflow is a bespoke implementation with its own usage tracking, error handling, and trace collection.

### Design rationale

The composition pattern provides four operators that close over the `Agent` trait -- any `Agent` (including composed agents) can be used as a component in any operator. This follows the composite design pattern [Gamma et al. 1995] and draws on recent mixture-of-agents research [Wang et al. 2024] showing that LLMs improve when presented with outputs from other models.

### Source: `roko-agent/src/composition.rs`

#### Merge strategies

```rust
pub enum MergeStrategy {
    /// Concatenate textual outputs in input order.
    Concatenate,
    /// Aggregate outputs into a structured JSON array.
    Aggregate,
    /// Majority vote over normalized text outputs.
    Vote,
    /// Pick the single best result using a heuristic.
    BestOfN,
}
```

#### Task-based conditional routing

```rust
pub struct SkillSelector {
    default_branch: usize,
    by_category: HashMap<TaskCategory, usize>,
    by_complexity: HashMap<TaskComplexityBand, usize>,
    by_reasoning: HashMap<TaskReasoningLevel, usize>,
    by_speed: HashMap<TaskSpeedPriority, usize>,
    by_quality: HashMap<TaskQualityProfile, usize>,
}

impl SkillSelector {
    pub fn new(default_branch: usize) -> Self { /* ... */ }
    pub fn with_category(mut self, cat: TaskCategory, branch: usize) -> Self { /* ... */ }
    pub fn with_complexity(mut self, band: TaskComplexityBand, branch: usize) -> Self { /* ... */ }

    /// Score a task into a branch index, checking dimensions in priority order.
    pub fn select(&self, task: &Task) -> usize {
        // Priority: category > complexity > reasoning > speed > quality > default
        // ...
    }
}
```

#### The four composition operators

```rust
pub enum AgentComposition {
    /// Run agents in sequence, feeding each output into the next input.
    Pipeline(Vec<Box<dyn Agent>>),
    /// Run agents concurrently and merge their results.
    Parallel(Vec<Box<dyn Agent>>, MergeStrategy),
    /// Pick one branch from the task selector.
    Conditional {
        condition: Box<dyn Fn(&Task) -> usize + Send + Sync>,
        branches: Vec<Box<dyn Agent>>,
    },
    /// Run a candidate set, then let an aggregator compress the fan-out.
    MixtureOfAgents {
        agents: Vec<Box<dyn Agent>>,
        aggregator: Box<dyn Agent>,
    },
}
```

#### CompositeAgent wrapper

```rust
pub struct CompositeAgent {
    name: String,
    composition: AgentComposition,
}

#[async_trait]
impl Agent for CompositeAgent {
    async fn run(&self, input: &Signal, ctx: &Context) -> AgentResult {
        match &self.composition {
            AgentComposition::Pipeline(agents) =>
                self.run_pipeline(agents, input, ctx).await,
            AgentComposition::Parallel(agents, strategy) =>
                self.run_parallel(agents, input, ctx, *strategy).await,
            AgentComposition::Conditional { condition, branches } =>
                self.run_conditional(condition.as_ref(), branches, input, ctx).await,
            AgentComposition::MixtureOfAgents { agents, aggregator } =>
                self.run_mixture(agents, aggregator, input, ctx).await,
        }
    }

    fn name(&self) -> &str { &self.name }

    fn supports_streaming(&self) -> bool {
        // All components must support streaming for the composition to support it
        match &self.composition {
            AgentComposition::Pipeline(agents) =>
                agents.iter().all(|a| a.supports_streaming()),
            // ... same for other variants
        }
    }
}
```

### Pipeline execution semantics

```
Input --> [Agent A] --> output_A --> [Agent B] --> output_B --> [Agent C] --> Final Output
              |                          |                          |
              v                          v                          v
           trace_A                    trace_B                    trace_C
                    \                    |                    /
                     +------- merged trace --------+--------+

Usage = sum(usage_A, usage_B, usage_C)
Success = success_A AND success_B AND success_C  (short-circuits on first failure)
```

If any stage fails (`result.success == false`), the pipeline stops immediately and returns the failing result with all accumulated usage and trace.

### Parallel execution semantics

```
         +---> [Agent A] ---> result_A ---+
Input ---+---> [Agent B] ---> result_B ---+--> merge(strategy) --> Final Output
         +---> [Agent C] ---> result_C ---+

Usage = sum(usage_A, usage_B, usage_C)
Success = success_A AND success_B AND success_C
```

All branches run concurrently via `futures::future::join_all`. The merge strategy determines how outputs are combined.

### Merge strategy details

| Strategy | Behavior | Use case |
|----------|----------|----------|
| `Concatenate` | Join text outputs with `\n\n` | Multi-perspective analysis |
| `Aggregate` | Collect as JSON array with success/kind/content per result | Structured fan-out results |
| `Vote` | Majority vote over lowercased, trimmed text | Classification consensus |
| `BestOfN` | Pick result with (success, content length, earliest index) priority | Quality selection |

### Mixture-of-agents execution

The MoA operator implements the two-phase pattern from Wang et al. [2024]:

```
Phase 1 (fan-out): Run all candidate agents in parallel with Aggregate merge
Phase 2 (synthesis): Feed the aggregated JSON to the aggregator agent

         +---> [Candidate A] ---+
Input ---+---> [Candidate B] ---+--> JSON aggregate --> [Aggregator] --> Final Output
         +---> [Candidate C] ---+
```

### Worked example: conditional routing by task category

```rust
let selector = SkillSelector::new(0)
    .with_category(TaskCategory::Docs, 1);

let agent = CompositeAgent::new(
    "conditional",
    AgentComposition::conditional(
        selector,
        vec![
            Box::new(MockAgent::reply("default")),
            Box::new(MockAgent::reply("docs")),
        ],
    ),
);

let task = Task {
    category: Some(TaskCategory::Docs),
    complexity_band: Some(TaskComplexityBand::Standard),
    ..Task::new("t1", "docs")
};
let input = Signal::builder(Kind::Prompt)
    .body(Body::from_json(&task).unwrap())
    .build();
let result = agent.run(&input, &Context::at(0)).await;
assert_eq!(result.output.body.as_text().unwrap(), "docs");
```

### Worked example: pipeline feeds output forward

```rust
let agent = CompositeAgent::new(
    "pipe",
    AgentComposition::pipeline(vec![
        Box::new(MockAgent::reply("stage-one")),
        Box::new(MockAgent::reply("stage-two")),
    ]),
);

let result = agent.run(&prompt("start"), &Context::at(0)).await;
assert!(result.success);
assert_eq!(result.output.body.as_text().unwrap(), "stage-two");
```

---

## 13. Pattern 10: Warm Session Reuse and Resume Validation

### Problem

Spawning a new agent session for every task is expensive -- it requires initializing subprocess state, negotiating authentication, loading context, and paying the cold-start latency. When multiple tasks share the same configuration (same model, same prompt template, same context), reusing a warm session can save significant time and cost.

However, session reuse is dangerous if not validated. A session warmed with one prompt template or context cannot safely serve a task with a different prompt -- the model would hallucinate based on stale context. The system needs explicit opt-in policies that declare when reuse is safe and validation gates that reject mismatched resume requests.

### Design rationale

The warm session reuse pattern uses BLAKE3 content-addressable fingerprints to detect prompt and context drift. The `WarmReusePolicy` declares the conditions under which a warmed session may be selected, and `validate_resume_request()` enforces those conditions at resume time. This follows a "fail-closed" philosophy -- any mismatch rejects the resume, forcing a fresh session.

BLAKE3 was chosen for its speed (over 3x faster than SHA-256 on modern hardware), its security properties (based on the BLAKE2/BLAKE family [Aumasson et al. 2020]), and its availability as a Rust crate with zero-copy hashing.

### Source: `roko-agent/src/session.rs`

#### Reuse scope hierarchy

```rust
pub enum ReuseScope {
    /// Reuse is disabled.
    Disabled,
    /// Reuse is valid only for the exact task id.
    Task,
    /// Reuse is valid within the same plan id.
    Plan,
    /// Reuse is valid for a caller-defined session id.
    Session,
}
```

#### Warm reuse policy

```rust
pub struct WarmReusePolicy {
    pub policy_id: String,
    pub scope: ReuseScope,
    pub max_idle_ms: Option<u64>,
    pub plan_id: Option<String>,
    pub task_id: Option<String>,
    pub session_id: Option<String>,
    pub prompt_policy_fingerprint: Option<String>,
    pub context_fingerprint: Option<String>,
    pub allow_context_carryover: bool,
}
```

#### The `allows()` validation gate

```rust
impl WarmReusePolicy {
    pub fn allows(&self, request: &WarmReuseRequest, warmed_at: Instant, now: Instant) -> bool {
        // Gate 1: scope disabled -> reject
        if self.scope == ReuseScope::Disabled { return false; }

        // Gate 2: idle timeout -> reject
        if let Some(max_idle_ms) = self.max_idle_ms {
            let age_ms = now.duration_since(warmed_at).as_millis() as u64;
            if age_ms > max_idle_ms { return false; }
        }

        // Gate 3: prompt fingerprint mismatch -> reject
        if self.prompt_policy_fingerprint != request.prompt_policy_fingerprint {
            return false;
        }

        // Gate 4: context fingerprint mismatch -> reject
        if self.context_fingerprint != request.context_fingerprint {
            return false;
        }

        // Gate 5: context carryover not explicitly allowed -> reject
        if request.context_fingerprint.is_some() && !self.allow_context_carryover {
            return false;
        }

        // Gate 6: scope-specific identity match
        match self.scope {
            ReuseScope::Disabled => false,
            ReuseScope::Task => {
                self.plan_id == request.plan_id && self.task_id == request.task_id
            }
            ReuseScope::Plan => self.plan_id == request.plan_id,
            ReuseScope::Session => self.session_id == request.session_id,
        }
    }
}
```

#### Resume validation (separate from reuse)

Resume validation is stricter -- it checks the full invocation record including backend, model, and role:

```rust
pub fn validate_resume_request(
    persisted: &AgentInvocationSession,
    requested: &AgentInvocationSession,
) -> Result<(), ResumeValidationError> {
    if !persisted.state.is_resumable() {
        return Err(ResumeValidationError::TerminalState(persisted.state));
    }
    if persisted.backend_id != requested.backend_id {
        return Err(ResumeValidationError::BackendMismatch);
    }
    if persisted.model != requested.model {
        return Err(ResumeValidationError::ModelMismatch);
    }
    if persisted.role != requested.role {
        return Err(ResumeValidationError::RoleMismatch);
    }
    if persisted.plan_id != requested.plan_id || persisted.task_id != requested.task_id {
        return Err(ResumeValidationError::ScopeMismatch);
    }
    if persisted.prompt_fingerprint != requested.prompt_fingerprint {
        return Err(ResumeValidationError::PromptMismatch);
    }
    if persisted.context_fingerprint != requested.context_fingerprint {
        return Err(ResumeValidationError::ContextMismatch);
    }
    Ok(())
}
```

#### Invocation state machine

```rust
pub enum InvocationState {
    InProgress,  // Running or interrupted before terminal update
    Succeeded,   // Completed successfully
    Failed,      // Failed
    TimedOut,     // Timed out
    Cancelled,   // Cancelled
}

impl InvocationState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::TimedOut | Self::Cancelled)
    }
    pub const fn is_resumable(self) -> bool {
        matches!(self, Self::InProgress | Self::TimedOut | Self::Cancelled)
    }
}
```

#### BLAKE3 fingerprinting

```rust
/// Stable BLAKE3 fingerprint for prompt/context policy material.
pub fn fingerprint_text(text: &str) -> String {
    ContentHash::of(text.as_bytes()).to_hex()
}
```

`ContentHash` wraps `blake3::hash()` and produces a hex-encoded digest. This fingerprint is used for both prompt policy fingerprinting (detecting template changes) and context fingerprinting (detecting stale context).

### Worked example: session reuse with fingerprints

```rust
let policy = WarmReusePolicy::stateless("test-policy")
    .for_session("sess-1")
    .with_fingerprints(
        Some(fingerprint_text("You are a coding assistant")),
        None,
    )
    .with_max_idle(Duration::from_secs(300));

// Same session, same prompt -> allowed
let request = WarmReuseRequest::session("sess-1")
    .with_fingerprints(
        Some(fingerprint_text("You are a coding assistant")),
        None,
    );
assert!(policy.allows(&request, warmed_at, now));

// Different prompt fingerprint -> rejected
let bad_request = WarmReuseRequest::session("sess-1")
    .with_fingerprints(
        Some(fingerprint_text("You are a testing assistant")),
        None,
    );
assert!(!policy.allows(&bad_request, warmed_at, now));

// Idle too long -> rejected
let stale_now = warmed_at + Duration::from_secs(600);
assert!(!policy.allows(&request, warmed_at, stale_now));
```

---

## 14. IronClaw Integration Plan

### Current IronClaw architecture

IronClaw's agent system in `src/agent/` uses a different architecture from roko's trait-based approach. The key components:

| IronClaw module | Role | Roko equivalent |
|----------------|------|-----------------|
| `agent_loop.rs` | `Agent` struct, `AgentDeps`, main event loop | `TaskRunner` |
| `dispatcher.rs` | Agentic loop: LLM call -> tool execution -> repeat | `ToolLoop` |
| `agentic_loop.rs` | Shared `run_agentic_loop()` engine, `LoopDelegate` trait | `Agent` trait |
| `session.rs` | `Session` -> `Thread` -> `Turn` data model | `session.rs` |
| `session_manager.rs` | Lifecycle, lookup, pruning | `HarnessRegistry` |
| `compaction.rs` | Context window management | (no direct equivalent) |
| `context_monitor.rs` | Memory pressure detection | `MetacognitiveMonitor` |
| `self_repair.rs` | Stuck job/broken tool detection | `MetacognitiveMonitor` |
| `undo.rs` | Turn-based undo/redo with checkpoints | `Checkpoint` |
| `scheduler.rs` | Parallel job scheduling | `AgentComposition::Parallel` |
| `cost_guard.rs` | LLM spend and rate enforcement | `BudgetGuardrail` |

### Pattern-by-pattern integration mapping

#### 1. Agent trait -> LoopDelegate trait

IronClaw already has a trait-based agent abstraction: `LoopDelegate` in `agentic_loop.rs`. Three implementations exist: `ChatDelegate`, `JobDelegate`, `ContainerDelegate`. The roko `Agent` trait is narrower (just `run`), while `LoopDelegate` has more lifecycle hooks (`check_signals`, `before_llm_call`, `after_iteration`).

**Integration approach**: No change needed. IronClaw's `LoopDelegate` is already more capable than roko's `Agent`. The roko patterns are inspiration for adding composition operators on top of `LoopDelegate`.

#### 2. Translator -> ironclaw_llm crate

IronClaw's `ironclaw_llm` crate already handles multi-provider LLM integration. The translator pattern's key insight -- separating wire format translation from transport -- could improve the LLM crate's provider abstraction.

**Integration approach**: If IronClaw adds non-OpenAI-compatible backends (e.g., a ReAct-only model), extract a `Translator` trait in `ironclaw_llm` following roko's design. Currently not needed since IronClaw uses rig-core for provider abstraction.

#### 3. StreamAccumulator -> web gateway SSE

IronClaw's web gateway in `src/channels/web/` streams SSE events to the browser. The `StreamAccumulator` pattern could improve the gateway's handling of fragmented LLM responses.

**Integration approach**: Add a `StreamAccumulator` struct to `src/channels/web/` that normalizes provider-specific SSE formats into a canonical event vocabulary. This is useful if IronClaw adds streaming from multiple LLM providers with different SSE formats.

#### 4. Checkpoint -> undo.rs enhancement

IronClaw's `undo.rs` already implements turn-based checkpointing with max 20 checkpoints. The roko `Checkpoint` pattern adds two capabilities IronClaw lacks: provider session state preservation and disk persistence for crash recovery.

**Integration approach**: Extend `undo.rs` checkpoints with optional `SessionState` (provider session IDs for resume) and add `save()`/`load()` methods for crash recovery. This requires adding a checkpoint directory under `~/.ironclaw/checkpoints/`.

#### 5. MetacognitiveMonitor -> self_repair.rs + context_monitor.rs

IronClaw already has two metacognition-adjacent modules: `self_repair.rs` (stuck job detection) and `context_monitor.rs` (memory pressure). The roko pattern consolidates these into a single monitor with configurable thresholds.

**Integration approach**: Add roko-style repetition detection (tool-call fingerprinting) and contradiction detection to `self_repair.rs`. The confidence thresholding is less relevant since IronClaw doesn't currently extract model confidence, but could be added when confidence-reporting models are used.

#### 6. HarnessAdapter -> not needed

IronClaw does not delegate to external agent runtimes. It is itself the runtime. This pattern is not applicable.

#### 7. Composable Scorers -> workspace search

IronClaw's workspace uses hybrid search (FTS + vector via RRF). The composable scorer pattern could replace the hardcoded RRF scoring with a configurable scorer pipeline.

**Integration approach**: Add `SumScorer`/`MulScorer` to `src/workspace/` for composable search ranking. Low priority since the current RRF approach works well.

#### 8. TaskRunner -> agent_loop.rs

IronClaw's `agent_loop.rs` already serves as the task coordination point with `AgentDeps`. The roko `TaskRunner` pattern adds structured event broadcasting and conductor-driven model escalation.

**Integration approach**: Add a `tokio::broadcast` event bus to `AgentDeps` for structured runtime events (turn started, turn completed, cost recorded, anomaly detected). This enables monitoring and observability without polluting the main code path.

#### 9. RetryPolicy -> ironclaw_llm

IronClaw's `ironclaw_llm` crate handles provider retries. The roko pattern's explicit error classification and full-jitter backoff with floor would improve resilience.

**Integration approach**: Add `ErrorClass` and `RetryPolicy` to `ironclaw_llm`. The floored full-jitter formula (floor = base/2) is a concrete improvement over naive exponential backoff.

#### 10. Composition operators -> scheduler.rs

IronClaw's `scheduler.rs` supports parallel job scheduling. The roko composition pattern adds structured pipeline, conditional, and mixture operators.

**Integration approach**: Add an `AgentComposition` enum to `src/agent/` for composing `LoopDelegate` implementations. Most useful for implementing multi-step workflows (e.g., plan -> implement -> review pipeline).

#### 11. Warm session reuse -> session_manager.rs

IronClaw's `session_manager.rs` already tracks sessions and prunes idle ones. The roko pattern adds fingerprint-based reuse validation.

**Integration approach**: Add BLAKE3 fingerprinting of system prompts and context to `session_manager.rs`. On session reuse, validate that the prompt fingerprint matches. This prevents stale context leakage across different conversation contexts.

### Priority ranking

| Priority | Pattern | Effort | Value |
|----------|---------|--------|-------|
| P1 | Retry with full-jitter backoff | Low | High (reduces rate-limit failures) |
| P2 | Event bus for TaskRunner | Medium | High (enables observability) |
| P3 | Metacognitive repetition detection | Medium | Medium (catches stuck loops earlier) |
| P4 | Checkpoint crash recovery | Medium | Medium (prevents lost work) |
| P5 | Stream accumulator | Low | Low (current approach works) |
| P6 | Composition operators | High | Medium (enables new workflows) |
| P7 | Composable scorers | Low | Low (current RRF works) |
| P8 | Session fingerprinting | Low | Low (single-user currently) |

---

## 15. Academic References

### Core patterns and design

1. **Gamma, E., Helm, R., Johnson, R., & Vlissides, J.** (1995). *Design Patterns: Elements of Reusable Object-Oriented Software*. Addison-Wesley. -- The Adapter, Composite, and Strategy patterns underpin the Translator, CompositeAgent, and SkillSelector designs respectively.

2. **Cockburn, A.** (2005). "Hexagonal Architecture (Ports and Adapters)." -- The Agent trait as a "port" with multiple adapter implementations follows this architectural pattern.

### Agent reasoning frameworks

3. **Yao, S., Zhao, J., Yu, D., Du, N., Shafran, I., Narasimhan, K., & Cao, Y.** (2023). "ReAct: Synergizing Reasoning and Acting in Language Models." *ICLR 2023*. -- The ReAct translator implements this Thought-Action-Observation loop for models without native function calling.

4. **Wang, J., Wang, X., Shang, J., et al.** (2024). "Mixture-of-Agents Enhances Large Language Model Capabilities." *arXiv:2406.04692*, ICLR 2025. -- The `MixtureOfAgents` composition operator implements this paper's fan-out-then-aggregate architecture.

### Metacognition and self-monitoring

5. **Flavell, J. H.** (1979). "Metacognition and cognitive monitoring: A new area of cognitive-developmental inquiry." *American Psychologist*, 34(10), 906-911. -- Foundational work on metacognition that inspired the MetacognitiveMonitor pattern.

6. **Cox, M. T.** (2005). "Metacognition in computation: A selected research review." *Artificial Intelligence*, 169(2), 104-141. -- Survey of computational metacognition approaches.

7. **Xie, Z., et al.** (2024). "MIRROR: A Hierarchical Benchmark for Metacognitive Calibration in Large Language Models." *arXiv:2604.19809*. -- Quantitative framework for measuring LLM metacognition, relevant to the confidence thresholding in the monitor.

### Fault tolerance and retry

8. **Elnozahy, E. N., Alvisi, L., Wang, Y.-M., & Johnson, D. B.** (2002). "A survey of rollback-recovery protocols in message-passing systems." *ACM Computing Surveys*, 34(3), 375-408. -- Comprehensive survey of checkpoint-restart techniques that the Checkpoint pattern draws from.

9. **Egwutuoha, I. P., Levy, D., Selic, B., & Chen, S.** (2013). "A survey of fault tolerance mechanisms and checkpoint/restart implementations for high performance computing systems." *The Journal of Supercomputing*, 65(3), 1108-1152.

10. **Brooker, M.** (2015). "Exponential Backoff and Jitter." *AWS Architecture Blog*. https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/ -- The RetryPolicy's full-jitter backoff formula derives directly from this analysis, which showed full jitter significantly reduces total work and server load compared to naive exponential backoff or equal jitter.

### Streaming and hashing

11. **W3C** (2015). "Server-Sent Events." *W3C Recommendation*, February 3, 2015. -- The streaming reassembly pattern handles the SSE event-stream format standardized in this specification.

12. **O'Connor, J., Aumasson, J.-P., Neves, S., & Wilcox-O'Hearn, Z.** (2020). "BLAKE3: One function, fast everywhere." *IETF Internet-Draft: draft-aumasson-blake3-00*. https://github.com/BLAKE3-team/BLAKE3 -- The content-addressable fingerprinting in the session module uses BLAKE3 for its speed (3x+ SHA-256), parallelizability, and security properties.

---

## 16. Complexity Assessment

### Pattern complexity matrix

| Pattern | Lines of code | External deps | Test count | Integration difficulty |
|---------|-------------|---------------|-----------|----------------------|
| Agent trait | ~250 | `async-trait`, `tokio`, `roko-core` | 5 | Low (trait only) |
| Translator | ~550 (mod.rs) + ~430 per impl | `serde_json`, `roko-core` | 20+ per impl | Medium (per-provider) |
| StreamAccumulator | ~200 | `roko-core` | 10+ | Low (state machine) |
| Checkpoint | ~155 | `serde`, `tempfile` (tests) | 4 | Low (serde round-trip) |
| MetacognitiveMonitor | ~310 | `serde`, `roko-core` | 5 | Medium (threshold tuning) |
| HarnessAdapter | ~240 (mod.rs) + ~950 (capability.rs) | `async-trait`, `serde` | 30+ | High (per-harness) |
| Composable scorers | ~170 | `roko-core` | 5 | Low (algebra) |
| TaskRunner | ~200+ | `tokio`, `chrono`, `indexmap` | 10+ | Medium (integration) |
| RetryPolicy | ~260 | `rand` | 8 | Low (pure computation) |
| Composition | ~580 | `async-trait`, `futures` | 5 | Medium (concurrency) |
| Session/Resume | ~560 | `serde`, `blake3` (via `roko-core`) | 20+ | Medium (policy logic) |

### Risk assessment

| Risk | Mitigation |
|------|-----------|
| Translator format mismatch produces silent tool-call failures | Round-trip tests for every translator; `TranslatorError::Malformed` catches parse failures |
| StreamAccumulator drops chunks under backpressure | `mpsc::channel` with bounded capacity; consumer must keep up |
| Checkpoint deserialization fails after schema change | `#[serde(default)]` on new fields; version testing in CI |
| MetacognitiveMonitor false positives on contradiction | Small, high-precision vocabulary; configurable thresholds |
| RetryPolicy infinite loop if `max_attempts` misconfigured | Hard cap: `attempt.min(10)` in exponential growth |
| Composition operators create unbounded parallelism | Caller controls the number of branches; no implicit fan-out |
| Session reuse serves stale context | Fail-closed validation; fingerprint mismatch rejects reuse |
