# Integration Roadmap

This roadmap gives the order of operations for evaluating captured ideas in
IronClaw. It does not replace subsystem specs, `FEATURE_PARITY.md`, or
implementation readiness docs.

Use it with:

- [priority-matrix/README.md](priority-matrix/README.md) for priority tiers.
- [priority-matrix/benchmarking-plans.md](priority-matrix/benchmarking-plans.md) for measurement gates.
- [priority-matrix/implementation-sketches.md](priority-matrix/implementation-sketches.md) for minimal integration contracts.

## Guiding Constraints

- Extend IronClaw-owned boundaries before adding new crates or generalized frameworks.
- Reborn child runs must continue through the existing runner/driver/executor path; do not introduce a second agent loop.
- Tool-initiated state changes must preserve `ToolDispatcher` auditing unless a documented exception already exists.
- Database changes require PostgreSQL and libSQL support.
- Security-sensitive routes, auth, secrets, sandboxing, listeners, approvals, and outbound HTTP require explicit security review.
- Behavior changes need caller-level tests and, where relevant, `FEATURE_PARITY.md` or subsystem doc updates.

## Phase 0: Baseline And Design Check

Do this before coding any broad runtime change or autonomous background feature.

| Work | Output |
|------|--------|
| Inspect current owner modules and subsystem docs. | Confirm where the feature belongs. |
| Capture baseline metrics. | Cost, latency, quality, retrieval, false-positive, or defect-catch baseline. |
| Define rollback. | Flag, inert metadata, or small revert. |
| Define caller-level tests. | Test the boundary where side effects happen. |
| Check security and DB impact. | Decide if threat model or dual-backend migration is needed. |

## Phase 1: Narrow Reliability And Memory Work

These are the first implementation candidates because they are small, reversible, and useful without broad architecture changes.

| Order | Feature | Primary areas | Key gate |
|-------|---------|---------------|----------|
| 1 | Robust statistics | `src/estimation/`, small utility helpers | Estimator caller handles outliers without clean-series regression. |
| 2 | Exact memory dedup | `src/workspace/`, `src/tools/builtin/memory.rs` | Duplicate writes merge through the real memory tool path. |
| 3 | Memory decay | `src/workspace/`, heartbeat/background-safe context | Stale memory is archived, identity/system memory is never archived. |
| 4 | Metacognitive monitor | `src/agent/agentic_loop.rs`, `src/agent/self_repair.rs`, `src/agent/cost_guard.rs` | Alternating stuck loops detected with low false intervention rate. |
| 5 | Composable scorers | `src/evaluation/` or current evaluation owner | One real caller consumes the scorer result. |
| 6 | Hierarchical cancellation | `src/tools/dispatch.rs`, process/tool integrations | Cancelling a turn stops a long-running tool subprocess. |

Phase 1 exit criteria:

- Targeted tests and clippy pass for changed crates.
- Memory changes are recoverable and do not delete content.
- Internal diagnostics use debug/trace logging.
- Any config flags default conservatively.

## Phase 2: Measured Core Enhancements

These change model selection, verification, or retrieval. They need baselines and staged rollout.

| Feature | Build stance | Required gate |
|---------|--------------|---------------|
| Cascade router | Start with shadow decisions around existing smart routing. | No safety override violations; quality within tolerance; cost/latency benefit shown in traces. |
| Gate verification expansion | Extend existing gate/builder validation in rungs. | Safe command construction, bounded/redacted diagnostics, known-defect fixtures. |
| HDC similarity signal | Prototype offline as an additional retrieval signal. | Relevant@10 lift on compositional queries with no ordinary-query regression. |
| Cognitive speed labels | Add only if consumed by router, budgets, or UI. | Caller test proves the label changes a real decision. |
| Enhanced heartbeat | Consider after decay and cost caps exist. | Background work is disabled by default, budget-capped, and excluded during active sessions. |

Phase 2 exit criteria:

- Shadow/canary data supports any active routing or ranking change.
- Kill switches return to baseline behavior.
- Security review completed for gates and any command execution.
- Dual-backend tests exist for any new persistence tables.

## Phase 3: Architecture Evolution

These are conditional. They should start only when a product requirement cannot be met by existing runtime boundaries.

| Feature | Proceed only if | Main constraint |
|---------|-----------------|-----------------|
| DAG workflow runner | Users need repeatable multi-step workflows that existing Reborn execution cannot express. | All cells dispatch through existing tool/runtime paths; no second agent loop. |
| Conductor/provider health | Router episodes and provider health data exist. | Start observe-only; avoid noisy predictive failover. |
| Resumable checkpoints | A concrete crash/retry case is not already handled by turn/run state. | Persist metadata and refs only where required; avoid raw prompt/tool leakage. |
| Full consolidation/dreams | Decay, HDC, budget caps, and background isolation are validated. | Default off, strict daily budget, sampled quality review. |

Phase 3 exit criteria:

- Design review completed.
- Threat model completed for any code execution or autonomous LLM spend.
- Rollback has been tested.
- Documentation and parity files are updated if behavior status changes.

## Phase 4: Research And Deferred Ideas

Do not schedule these as implementation work without a separate design document and user-value trigger.

| Idea | Near-term stance |
|------|------------------|
| NEAR on-chain reputation | Start with off-chain trust signals only if needed. No chain dependency for internal scoring. |
| Pheromone coordination | Model as tagged workspace memory only if multi-agent coordination becomes active. |
| Broad code intelligence | Start Rust-only and fixture-backed if workspace code search has measurable gaps. |
| Full affect engine | Prefer simple engagement signals; avoid unvalidated psychological modeling. |
| Budget auctions / VCG | Start with deterministic cache-aware prompt ordering before auction mechanisms. |
| TDA/sheaves | Research reference only. |

## Risk Controls

| Risk | Applies to | Control |
|------|------------|---------|
| Database divergence | Any new persistence | Add shared trait operation first, implement PostgreSQL and libSQL, test both. |
| Memory loss | Dedup, decay, consolidation | Archive or merge with metadata; never delete user memory as part of these features. |
| Cost runaway | Router, heartbeat, consolidation | Baselines, budget caps, cheap-model limits, kill switch. |
| Security regression | Gates, tools, outbound HTTP, auth | Explicit args, path validation, sandboxing, redaction, threat review. |
| False automation | Monitor, conductor, gates | Observe-only mode or conservative thresholds before active intervention. |
| Feature flag drift | All staged features | Inventory flags, test kill switch, retire flags only after stability. |

## Implementation Rules

1. Keep PRs scoped to one candidate and one owning boundary.
2. Prefer metadata or existing tables for reversible experiments; add schema only when query requirements demand it.
3. Test through the caller when a helper gates a side effect.
4. Do not add live external dependencies or source links.
5. Do not port external UI themes, execution loops, or storage models directly.
6. Preserve existing defaults unless the feature explicitly changes them.
7. Update relevant docs, specs, and `FEATURE_PARITY.md` when behavior status changes.

## Rollout Checklist

Before enabling any staged feature by default:

- Baseline captured and linked.
- Targeted tests pass.
- Caller-level regression test added.
- Rollback tested.
- Security and DB review completed if applicable.
- User-visible behavior documented where relevant.
