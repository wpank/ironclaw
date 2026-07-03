# Gate Verification Pipeline

A gate decides whether a result is safe or complete enough to proceed. In IronClaw, gates must sit at caller boundaries that already own side effects: tools, LLM calls, jobs, approvals, Reborn runs, and gateway actions.

## Existing Boundaries

| Gate Type | Current Boundary |
| --- | --- |
| Tool safety | `src/tools/execute.rs::execute_tool_with_safety` |
| Channel/system tool dispatch | `src/tools/dispatch.rs::ToolDispatcher` |
| Approval/auth | `src/agent` pending approval/auth paths, `src/gate/store.rs`, Reborn gate refs |
| Budget | `src/agent/cost_guard.rs`, `src/bridge/cost_guard_gate.rs` |
| LLM provider failure | retry/circuit/failover in `crates/ironclaw_llm` |
| Reborn execution | `crates/ironclaw_engine`, `src/bridge/effect_adapter.rs`, product workflow/composition crates |

Do not build a generic gate path that bypasses these owners.

## Progressive Verification

Use the narrowest rung that protects the side effect:

| Rung | Use |
| --- | --- |
| Shape | JSON/schema/type validation. |
| Safety | prompt-injection, secret-leak, sandbox, allowlist, approval policy. |
| Compile/lint | code generation or generated tool artifacts. |
| Unit test | local logic changes. |
| Caller contract | helper gates a real side effect. |
| Integration | DB, gateway, provider chain, routine, scheduler, Reborn runtime. |
| E2E/trace | user-visible path, model tool choice, request shape, or recorded Reborn behavior. |

Rungs are not always linear. A high-risk tool call may need safety and approval gates without compile/lint. A code patch may need compile, lint, targeted tests, and a caller contract.

## Verdict Model

Keep verdicts explicit:

| Verdict | Meaning |
| --- | --- |
| `pass` | Continue. |
| `fail` | Stop and surface evidence. |
| `retryable` | Caller may retry within its existing retry budget. |
| `needs_approval` | Route through existing approval/auth gate. |
| `inconclusive` | Do not enforce automatically; ask or escalate. |

Every verdict needs evidence, source ids, gate version, and scope. User-facing summaries should be concise and redact sensitive data.

## Adaptive Thresholds

Adaptive gates are useful only after static gates are reliable. Roll out as:

1. static gate,
2. shadow adaptive score,
3. advisory threshold suggestion,
4. enforcement behind a per-gate flag.

Adaptive logic cannot weaken security, auth, budget, CORS/origin, body-limit, allowlist, or secret-handling rules.

## Integration Plan

1. Inventory current gates and name their caller boundaries.
2. Add a shared verdict/evidence shape where it reduces duplication.
3. Add gate result logging without changing behavior.
4. Add one caller-level enforcement path at a time.
5. Add dashboard/debug visibility only after data is redacted and scoped.

Candidate first gates:

- generated tool validation in tool builder,
- post-tool result safety evidence through `execute_tool_with_safety`,
- Reborn post-approval execution cap around existing inline gate retry limit,
- acceptance-contract checks for job completion.

## Tests

- Unit-test pure gate predicates.
- Caller-level test: a malformed tool argument is rejected at `execute_tool_with_safety`.
- Caller-level test: `ToolDispatcher` persists sanitized audit output while returning the intended caller output.
- Reborn gate test: approval/auth gate resumes through the existing gate path, not a direct call.
- DB parity tests for persisted gate records.
- Trace fixture tests when gate behavior changes model/tool request shape or user-visible state.

## Cautions

- Do not paste full crate implementations into docs or code reviews.
- Do not claim false-positive/false-negative rates before measuring local traffic.
- Do not let generated tests become the only verifier for generated code.
- If a gate can block user work, provide clear evidence and a disable path.
