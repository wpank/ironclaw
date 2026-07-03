# Agent Patterns

These are reusable engineering patterns for IronClaw's existing agent and Reborn execution paths. They are not instructions to port a foreign agent trait or to add another loop.

## Core Boundary

IronClaw already has the primary abstractions:

- v1 chat/job/container execution: `src/agent/agentic_loop.rs` with `LoopDelegate`, plus `ChatDelegate`, `JobDelegate`, and `ContainerDelegate`.
- Reborn execution: `crates/ironclaw_engine/` and bridge adapters.
- Tools: `src/tools/registry.rs`, `src/tools/execute.rs`, and `src/tools/dispatch.rs`.
- LLMs: `crates/ironclaw_llm::LlmProvider`.
- Sessions and turns: `src/agent/session.rs` and `src/agent/session_manager.rs`.

Any new pattern should wrap or extend those surfaces.

## Pattern Map

| Pattern | IronClaw Home | Use |
| --- | --- | --- |
| Wire-format adapter | `crates/ironclaw_llm/` provider implementations | Normalize provider request/response details behind `LlmProvider`. |
| Streaming reassembly | Provider-specific streaming clients and gateway SSE/WebSocket handlers | Rebuild fragmented content/tool events before persistence or UI emission. |
| Resumable checkpoint | `src/agent/session.rs`, `src/agent/undo.rs`, Reborn run state | Persist enough state to resume or rollback without replaying unsafe side effects. |
| Metacognitive monitor | `src/agent/self_repair.rs`, loop post-iteration hooks, Reborn runner status | Detect stuck work, repeated failures, context pressure, and contradictions. |
| Harness adapter | test support and provider/tool harnesses | Keep external CLIs and fake services behind probeable adapters. |
| Composable scorers | routing, search, prioritization, risk classification | Combine small pure scorers; keep side effects at callers. |
| Budget guardrail runner | `CostGuard`, `BudgetGate`, scheduler, Reborn budgets | Warn, throttle, pause, or fail at the boundary that spends resources. |
| Classified retry | `crates/ironclaw_llm/retry.rs`, tool execution wrappers, gateway clients | Retry only transient classes; preserve auth, schema, and safety failures. |
| Composition operators | scheduler, routines, Reborn product workflow | Sequence, parallelize, race, or fallback without bypassing existing execution engines. |
| Warm session reuse | session manager, provider sessions, OAuth token managers | Reuse only when tenant, scope, model, tools, and credentials match. |

## Non-Negotiable Constraints

- Do not add a second agent loop. Subagent spawn wires child runs only; planning, execution, tool calls, gates, retries, checkpointing, and completion stay in the existing Reborn runner/driver/executor path.
- Do not pass trusted trigger requests from product adapters or host-runtime handlers. Use untrusted inbound requests unless the trigger-worker-owned path minted the request.
- Do not move module-owned initialization into `src/main.rs` or `src/app.rs`.
- Do not add production panic-on-error calls for these patterns.
- Do not use examples that imply a missing crate, missing source checkout, or generated implementation can be pasted as-is.

## Pattern Details

### Wire-Format Adapter

Provider-specific transport details belong inside `crates/ironclaw_llm`. The stable boundary is `LlmProvider` plus typed request/response structs. A new adapter should include:

- request conversion tests,
- response/tool-call parsing tests,
- error classification tests,
- one factory-level test proving `create_llm_provider` or registry wiring constructs it correctly.

### Streaming Reassembly

Streaming code must preserve ordering, partial tool-call arguments, finish reasons, and usage observations. UI-facing SSE/WebSocket events are downstream projections, not the source of truth.

Validation should include fragmented deltas, interleaved tool-call arguments, terminal errors, and a gateway-level test when UI behavior changes.

### Resumable Checkpoint

A checkpoint may restore local control state, but it must not re-run side effects silently. Tool calls require audit records; approval and auth gates require their existing resume paths.

Good checkpoint contents:

- run id, tenant/user/project scope,
- model and tool surface fingerprint,
- current phase and pending gate ids,
- last durable event sequence,
- safe local scratch state.

Bad checkpoint contents:

- raw secrets,
- unscoped tool handles,
- stale auth decisions,
- hidden system prompt mutations.

### Metacognitive Monitor

Monitor signals should be cheap, deterministic, and explainable. Examples:

- same tool fails with same normalized error repeatedly,
- the loop alternates between two incompatible plans,
- token or cost budget approaches a hard gate,
- context pressure rises while progress stays flat,
- user correction invalidates the current plan.

Start in shadow mode. Enforcement options should be limited to asking for clarification, summarizing/compacting, escalating model tier, or failing with a clear reason.

### Harness Adapter

External processes and fake services should expose probe, capabilities, and cleanup methods. Tests should not rely on global state unless isolated by temp dirs and explicit env guards.

### Composable Scorers

Scorers should be pure functions returning score plus evidence. The caller decides what the score means. This keeps tests simple and prevents helper-level tests from becoming false confidence.

### Budget Guardrail Runner

Budget checks must happen before spending and recording must happen after spending. If a new runner spends LLM or tool resources, test through the runner boundary, not only through `CostGuard`.

### Classified Retry

Retry policy should classify errors before sleeping:

| Retry | Do Not Retry |
| --- | --- |
| network timeout, HTTP server error, rate limit with retry-after, transient provider failure | auth failure, schema error, context length overflow, policy denial, user cancellation |

### Composition Operators

Composition is workflow structure, not model routing. Model selection remains inside `crates/ironclaw_llm`; workflow branching remains in scheduler/routine/Reborn orchestration.

### Warm Session Reuse

Reuse is valid only when the scoped identity, provider account, model/tool surface, and approval policy match. Add fingerprint checks before any reuse that crosses turns, threads, jobs, or tenants.

## Tests

For each adopted pattern, include:

- pure unit tests for local scoring/state logic,
- caller-level tests at the boundary that performs side effects,
- trace fixture updates when model tool choice or request shape is the protected behavior,
- Reborn QA fixture checks for user-visible cross-layer behavior,
- both DB backends for new persistence behavior.
