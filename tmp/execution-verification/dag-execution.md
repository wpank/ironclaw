# DAG Execution

A DAG execution engine runs bounded workflows where steps have explicit dependencies. In IronClaw it should complement the agent loop, routines, and Reborn workflows. It must not replace the chat/job/container loop or introduce a second subagent executor.

## When To Use A DAG

Use a DAG for:

- deterministic multi-step workflows,
- fan-out/fan-in tool calls,
- conditional validation branches,
- periodic routines with multiple independent checks,
- bounded code-generation gates.

Do not use a DAG for:

- open-ended conversational planning,
- unbounded tool loops,
- approval/auth flows that already have gate-resume semantics,
- subagent execution outside the Reborn runner/driver/executor path.

## Core Concepts

| Concept | Meaning |
| --- | --- |
| Graph | Versioned workflow definition. |
| Node | A bounded step: tool call, LLM call, transform, gate, or notification. |
| Edge | Dependency plus optional condition. |
| Context | Scoped runtime state: tenant/user/project/job/run, budget, tool registry, LLM provider. |
| Output | JSON-like value with status, evidence, cost, and audit references. |

Inputs and outputs should use structured values already accepted by IronClaw tools and bridge adapters. Avoid inventing a second universal signal type.

## IronClaw Integration

| Node Type | Boundary |
| --- | --- |
| Tool node | `ToolDispatcher` for channel/system callers, or existing worker/effect adapter when inside an agent/Reborn run. |
| LLM node | `crates/ironclaw_llm::LlmProvider` with normal retry/circuit/failover/cache decorators. |
| Gate node | existing approval/gate or verification boundary. |
| Transform node | pure local code with deterministic tests. |
| Routine node | `RoutineEngine` only for routine-owned scheduling. |

All side-effecting nodes must produce audit references. Tool nodes should not call tools directly from registry internals when a caller boundary already exists.

## Execution Model

Start simple:

1. Load a graph definition from a controlled source.
2. Validate node ids, edge ids, acyclicity, required inputs, and budget caps.
3. Execute sequentially in topological order.
4. Skip downstream nodes when dependencies fail unless an explicit failure edge exists.
5. Persist run events if and only if DB parity is implemented.

Parallel execution can come later. The first value is correctness, auditability, and caller-level integration.

## Feature Modes

| Mode | Behavior |
| --- | --- |
| `off` | No DAG execution. |
| `validate` | Load and validate definitions only. |
| `shadow` | Build run plan and estimated cost; do not execute side effects. |
| `execute` | Execute allowlisted graph types. |

Graph execution should be allowlisted by source and workflow type. User-editable workflow definitions need schema validation, body limits, and security review before execution.

## Budget and Safety

- Every run has max nodes, max wall time, max LLM calls, max tool calls, and max cost.
- Tool nodes use the existing safety pipeline and approval policy.
- LLM nodes use the existing provider chain and budget gates.
- File and network capabilities stay owned by tools/WASM/MCP/sandbox policy.
- A failed checkpoint must fail the graph safely; it must not silently re-run side effects.

## Tests

- Unit-test graph validation, topological ordering, condition evaluation, and skip semantics.
- Caller-level test: a tool node executes through `ToolDispatcher` and persists an `ActionRecord`.
- Provider-chain test: an LLM node uses the configured `LlmProvider` decorators.
- Budget test: graph execution stops before exceeding configured limits.
- Reborn/routine contract test if a DAG is exposed as a user-visible workflow.

## Implementation Notes

- Keep graph definitions compact. Prefer declarative schema plus small transforms over embedded scripts.
- Do not claim scheduling overhead or parallel speedups before local measurements exist.
- Hot/resident graphs should reuse heartbeat/routine/runtime scheduling rather than creating a new daemon loop.
- If a graph mutates code or files, gate through the same review/test surfaces as normal agent work.
