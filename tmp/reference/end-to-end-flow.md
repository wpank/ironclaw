# End-to-End Flow

This file is the self-contained request-flow reference for translating captured
Roko ideas into IronClaw work. Captured labels are provenance labels only; the
implementation target is the local IronClaw architecture.

## Baseline Request Path

| Stage | What happens | IronClaw anchor |
|---|---|---|
| Ingress | CLI, web, webhook, routine, or extension input is normalized. | `src/channels/` |
| Admission | Session/thread/turn state decides whether this is chat, command, auth/approval continuation, or background work. | `src/agent/`, `crates/ironclaw_threads/`, `crates/ironclaw_turns/` |
| Context | Identity, skills, workspace memory, recent state, and tool surfaces are selected. | `src/workspace/`, prompt/run-profile code |
| Model call | Provider routing and cost controls choose the model path. | `crates/ironclaw_llm/`, `src/agent/cost_guard.rs` |
| Tool loop | Tool requests are approved, dispatched, observed, and returned to the model loop. | `src/tools/dispatch.rs` |
| Verification | Generated artifacts or risky side effects pass caller-owned gates. | caller-level tests and feature-owned verification |
| Persistence | Turns, tool results, memory, events, and settings are written through owned storage APIs. | `src/db/`, `src/workspace/` |
| Egress | The channel emits the final answer and projected events. | `src/channels/`, `src/channels/web/` |

## Captured Concepts In The Flow

| Captured concept | Flow role | Caution |
|---|---|---|
| Signal / Engram | Durable record for outcomes, memories, verdicts, and derived lessons. | Do not replace existing DB or workspace semantics casually. |
| HDC / VSA | Candidate similarity signal for memory and code search. | Treat as an index strategy, not sole proof of semantic equality. |
| Gate pipeline | Structured artifact acceptance. | Gates must run through the caller that owns the side effect. |
| Cascade routing | Adaptive provider choice. | Static safety/privacy rules stay authoritative. |
| Conductor | Health and stuck-loop supervision. | Advisory until enforced by explicit runtime policy. |
| Dream consolidation | Background learning from past sessions. | Derived memories need source links, taint, confidence, and budget caps. |
| Pheromone / coordination marker | Shared hint for future agents or tasks. | Use local workspace metadata before trust integrations. |

## Scenario Sketches

### Code Change

1. User asks for a webhook or tool change.
2. The agent edits through the normal workspace path.
3. The owning caller selects compile/lint/test/security checks based on risk.
4. Gate output returns structured remediation if a check fails.
5. Passing results are persisted with bounded artifact references.

Regression coverage should drive the production caller, not only the gate helper.

### Memory Write

1. A user fact or derived lesson is proposed for storage.
2. The memory layer records source, author, taint, confidence, and identity.
3. Similarity search may flag duplicate candidates.
4. Hard merges require stronger evidence than HDC proximity alone.
5. Search ranking respects user-authored facts over low-confidence derived facts.

### Provider Degradation

1. Provider metrics show latency, retry, or error drift.
2. A health monitor may bias routing in observe or canary mode.
3. Existing circuit breakers remain the hard stop.
4. Rollback returns the monitor to observe mode if false positives or
   oscillation exceed rollout guardrails.

### Background Consolidation

1. A heartbeat or routine selects high-utility episodes inside budget.
2. A consolidation job writes derived memories with source turn links.
3. Derived memories remain low confidence until verified by later successful
   use.
4. Disabling the feature stops scheduled work and leaves manual fixtures
   available.

## Implementation Rules

- Extend existing owners instead of bypassing composition roots.
- Keep feature-flag branching in the owner module where possible.
- Add DB behavior to the shared DB trait before backend implementations.
- Add security review for listeners, auth, secrets, sandboxing, approvals, and
  outbound HTTP.
- Update docs and `FEATURE_PARITY.md` when behavior changes require it.

## Further Reading

- Summary: [end-to-end-synthesis.md](end-to-end-synthesis.md)
- Examples: [examples/02-end-to-end-scenarios.md](examples/02-end-to-end-scenarios.md)
- Failure modes: [examples/failure-scenarios.md](examples/failure-scenarios.md)
- Architecture: [architecture-overview.md](architecture-overview.md)
