# Decision Log — IronClaw × Roko Integration

Architecture Decision Records (ADRs) for the Roko concept adoption project.
Each record captures context, the decision made, alternatives considered, and
consequences. ADRs are append-only: superseded decisions get a "Superseded by"
line rather than being edited or deleted.

---

## ADR Template

```markdown
## ADR-NNN: <Short Title>

**Date**: YYYY-MM-DD
**Status**: Proposed | Accepted | Deprecated | Superseded by ADR-NNN
**Deciders**: <names or roles>
**Category**: Architecture | Testing | Database | Security | Rollout | Performance

### Context

<1-3 paragraphs. What is the situation, constraint, or question that requires
a decision? What forces are in play? What would happen if no decision is made?>

### Decision

<One clear statement of what was decided. Imperative voice. "We will ..." or
"All new X must ..."  No hedging.>

### Alternatives Considered

| Alternative | Reason Rejected |
|-------------|-----------------|
| <alt 1> | <why not> |
| <alt 2> | <why not> |

### Consequences

**Positive:**
- <benefit 1>
- <benefit 2>

**Negative / Trade-offs:**
- <cost or constraint 1>
- <cost or constraint 2>

**Risks:**
- <risk 1 and how it is mitigated>

### Implementation Notes

<Optional. Specific files, flags, or patterns that implement this decision.>
```

---

## ADR-001: Use Feature Flags for All New Features (Default Off)

**Date**: 2026-07-03
**Status**: Accepted
**Deciders**: IronClaw maintainers
**Category**: Rollout

### Context

The Roko adoption plan introduces 25 new behaviors across four phases, touching
the agent loop, LLM routing, workspace memory, a new gate verification crate,
and optional NEAR on-chain integration. IronClaw is a personal AI assistant
running in production for its users — a broken LLM routing change or a
memory-decay bug that archives the wrong documents would be immediately visible
and disruptive.

Several features (Cascade Router, Ebbinghaus Decay, HDC memory search) also
require a warm-up period or calibration data before they perform as well as the
baseline they are intended to replace. Shipping them "hot" would mean users
experience degraded quality before the system adapts.

Finally, IronClaw's `07-implementation-readiness-contract.md` already requires
a rollback switch for every new feature. The simplest, most consistent rollback
switch is a feature flag.

### Decision

Every new feature derived from the Roko adoption plan must be placed behind an
IronClaw-owned feature flag with key `experimental.<feature_name>` that
defaults to `false`. No feature may change observable behavior for users who
have not opted in. Feature flags must be readable at runtime without restart
for any feature that controls routing, background jobs, gates, or network
behavior.

### Alternatives Considered

| Alternative | Reason Rejected |
|-------------|-----------------|
| Ship features as opt-out (default on, users disable) | Too risky for production users. Calibration period means early behavior could be worse than baseline. |
| Use Rust `#[cfg(feature = "...")]` compile flags | These control dependency footprint, not runtime rollout. Cannot be changed without recompile. Per CLAUDE.md: "Compile features are for dependency footprint only, not rollout control." |
| A/B test infrastructure with percentage rollouts | Overkill for a personal assistant used by a single user at a time. Adds complexity without benefit at this scale. |
| No flags — just land each feature and iterate | No rollback path. Violates `07-implementation-readiness-contract.md` requirements. |

### Consequences

**Positive:**
- Every feature has a zero-cost rollback: set flag to false, restart (or hot-reload).
- Users who want to experiment can enable individual features independently.
- Enables shadow mode: run candidate alongside baseline, record metrics, compare before enabling.
- Satisfies `07-implementation-readiness-contract.md` rollback requirement.
- Prevents feature interactions: each feature can be enabled in isolation for debugging.

**Negative / Trade-offs:**
- Flag proliferation: 25 flags to manage. Mitigated by a central flag inventory (`rollout/03-feature-flag-inventory.md`).
- Dead code paths remain in the binary when flags are off. Acceptable at this scale.
- "Flag debt": flags that should have been cleaned up after GA. Each flag's README entry must include a `removal_criteria` field.

**Risks:**
- Risk: flag state not persisted across restarts → features re-disable themselves.
  Mitigation: flags read from `~/.ironclaw/settings.json` which persists across restarts.
- Risk: flag key collisions with future features.
  Mitigation: namespaced under `experimental.` prefix; register in `rollout/03-feature-flag-inventory.md`.

### Implementation Notes

```rust
// Pattern for checking a feature flag inside a module:
if config.features.experimental.get("metacognitive_monitor").copied().unwrap_or(false) {
    monitor.observe(action_hash, cost_cents);
}
```

Flag keys follow snake_case: `experimental.metacognitive_monitor`,
`experimental.memory_decay`, `experimental.cascade_router`,
`experimental.hdc_memory_search`, `experimental.progressive_gates`,
`experimental.dream_consolidation`, `experimental.dag_workflow_runner`,
`experimental.provider_conductor`, `experimental.cognitive_speeds`,
`experimental.full_dream_consolidation`, `experimental.local_reputation`,
`experimental.pheromone_trails`, `experimental.workspace_code_search`,
`experimental.vcg_budget_composition`, `experimental.affect_engine`.

---

## ADR-002: Store All New Per-Document Learning State in `metadata: serde_json::Value`

**Date**: 2026-07-03
**Status**: Accepted
**Deciders**: IronClaw maintainers
**Category**: Database

### Context

Several Phase 1 and Phase 2 features need to persist new per-document state:

- **Ebbinghaus Decay** (Phase 1.3): needs `stability_seconds`, `last_accessed_epoch`,
  `access_count`, `archived` flag per `MemoryDocument`.
- **BLAKE3 Content Dedup** (Phase 1.4): needs `content_hash` per `MemoryDocument`.
- **HDC Similarity** (Phase 2.2): needs `hdc_fingerprint: Vec<u8>` per `MemoryDocument`.
- **Dream Consolidation** (Phase 2.4 / 3.4): needs `staging_tier`, `confidence`,
  `source_episode_ids` per derived memory document.

IronClaw uses dual-backend persistence: PostgreSQL and libSQL/Turso. Per `CLAUDE.md`:
"All new persistence features must support both backends." A conventional column
migration must be written and tested twice. For five features adding columns across
two backends, that is ten migration files and ten integration tests just for schema
plumbing — before any feature logic ships.

`MemoryDocument` already has `metadata: serde_json::Value` — an unstructured JSON
column present in both backends.

### Decision

All new per-document learning state introduced by Phase 1–3 features (decay
state, content hash, HDC fingerprint, staging metadata) must be stored as fields
inside `MemoryDocument.metadata` rather than as new schema columns. New Rust
types (`DecayVariant`, `DecayState`) will serialize into and deserialize from
the existing `metadata` JSON field. No schema migrations are required for these
features.

The HDC fingerprint (a `Vec<u8>` of potentially 1,280 bytes per document) is
the one field where a dedicated column becomes worth the migration cost if query
performance requires it. This will be evaluated in Phase 2.2 after benchmarking.
If needed, a `hdc_fingerprint BYTEA / BLOB` column migration will be written for
both backends at that point.

### Alternatives Considered

| Alternative | Reason Rejected |
|-------------|-----------------|
| Add dedicated columns for each new field | Requires dual-backend migration for every field. 10+ migration files across Phase 1–3. High ceremony for exploratory features that may be removed. |
| Store in separate side tables | Even higher migration burden. Adds JOIN complexity to every memory read. |
| New append-only JSONL files (Roko-style) | Does not integrate with IronClaw's existing workspace search, tool boundary, or audit trail. Adds a third storage system. |
| In-memory only (no persistence) | Learning state is lost on restart. Ebbinghaus decay is meaningless without persistence. |

### Consequences

**Positive:**
- Zero schema migrations for Phase 1 and most of Phase 2.
- Both DB backends handled automatically (metadata is already a JSON column in both).
- Features can be removed by simply not writing the metadata fields — no schema cleanup.
- Consistent with IronClaw's existing pattern (identity files, heartbeat metadata already use this field).

**Negative / Trade-offs:**
- No database-level index on decay strength or content hash. Range queries (e.g., "find all documents with strength < 0.1") require a full table scan and Rust-side filtering. Acceptable for personal assistant scale (typically < 10,000 memory documents).
- JSON deserialization adds a small per-document overhead during search. Measured and verified acceptable before shipping Phase 1.3.
- `metadata` field grows larger over time as more subfields are added. Must be monitored; archived documents can have decay-only metadata (other fields omitted).

**Risks:**
- Risk: `metadata` field becomes a "junk drawer" with undocumented fields.
  Mitigation: every metadata field used by a feature is documented in that feature's `schema.md` in the evidence bundle.
- Risk: JSON key conflicts between features (e.g., two features both writing `metadata["hash"]`).
  Mitigation: each feature namespaces its keys (e.g., `metadata["decay"]`, `metadata["blake3"]`, `metadata["hdc"]`, `metadata["staging"]`).

### Implementation Notes

```rust
// Ebbinghaus state in metadata:
// metadata["decay"] = { "variant": "ebbinghaus", "stability_seconds": 3600.0,
//                       "last_accessed_epoch": 1751500000, "access_count": 3 }

// BLAKE3 content hash in metadata:
// metadata["blake3"] = "b3:af1234..."

// HDC fingerprint in metadata (base64 encoded):
// metadata["hdc"] = "AQID..."

// Staging metadata for Dream Consolidation:
// metadata["staging"] = { "tier": 1, "confidence": 0.72, "source_ids": ["abc", "def"] }
```

Deserialization uses `serde_json::from_value()` with `#[serde(default)]` on all
optional fields so documents without the field do not fail to deserialize.

---

## ADR-003: Start with Robust Statistics Before Implementing LinUCB (Cascade Router)

**Date**: 2026-07-03
**Status**: Accepted
**Deciders**: IronClaw maintainers
**Category**: Architecture, Performance

### Context

The Cascade Router (Phase 2.1) is the highest-ROI "Big Bets" feature: it
replaces IronClaw's static 13-dimension complexity scorer with a LinUCB
contextual bandit that learns which LLM provider is optimal for each task class.
The priority matrix gives it a composite ROI of 4.15 and projects 30–50% LLM
cost reduction.

The bandit learns from reward signals — primarily cost and pass rate per request.
These reward signals are computed from the same estimation infrastructure
(`src/estimation/learner.rs`) that currently uses a naive EMA over raw cost
ratios. A single anomalous LLM response (e.g., a $50 misclassified API call)
can distort the EMA cost factor by 50%+, propagating a corrupted reward signal
to the bandit for the next 20+ requests and destabilizing learning.

Robust statistics (Phase 1.1 — trimmed mean, MAD-based outlier detection) are a
1-2 day fix that costs essentially nothing and stabilizes the very signals the
bandit will train on. Shipping the bandit on top of corrupted reward signals
would make it harder to evaluate whether the bandit is working or whether
performance problems are signal noise.

The dependency graph in `strategy/integration-roadmap.md` shows a dashed arrow
from Robust Stats to Cascade Router: a soft dependency that means "benefits
significantly from." In practice, given the ease of Phase 1.1 (1-2 days) and
the risk of Phase 2.1 (3 weeks of effort on a critical path component), the
"soft" dependency should be treated as "required before Phase 2.1 begins."

### Decision

Phase 1.1 (Robust Statistics) must be merged and validated before Phase 2.1
(Cascade Router) work begins. The estimation module's EMA outlier-dampening
must be live and tested before the LinUCB bandit receives any real routing
episode data. The Cascade Router implementation must consume reward signals from
the robust-statistics-hardened learner, not the raw EMA.

### Alternatives Considered

| Alternative | Reason Rejected |
|-------------|-----------------|
| Build Cascade Router first, add robust stats later | Bandit trains on corrupted reward signals during its warm-up period. Corrupted learning is hard to detect and correct; the bandit may converge to a locally optimal but globally wrong routing policy that is expensive to undo. |
| Build Cascade Router with its own internal outlier filtering | Duplicates logic that belongs in the estimation module. Creates two separate outlier-filtering implementations that can diverge. Estimation module is the canonical source for cost/time learning signals. |
| Skip robust statistics entirely, use raw EMA for bandit rewards | LLM cost distributions are demonstrably heavy-tailed. Roko's primitives docs cite this explicitly. The EMA without dampening is provably distorted by single outlier calls. |
| Use median instead of trimmed mean in the bandit reward | Median is fine for ranking but loses magnitude information needed for accurate cost projections. Trimmed mean preserves magnitude while removing extremes. |

### Consequences

**Positive:**
- Bandit trains on clean signals from day 1 of its operation.
- Outlier-resistance in the estimation module benefits the cost guard and daily
  budget enforcement independently of the bandit — a free improvement.
- Phase 1.1 takes 1-2 days and has zero risk; it is a natural warm-up for Phase 2.1.
- Provides a concrete correctness baseline: after Phase 1.1, estimation tests
  verify that a 10x cost spike dampens the EMA update to < 10% of undampened.
  This bound is what the bandit training budget assumes.

**Negative / Trade-offs:**
- Phase 2.1 cannot start until Phase 1.1 is merged. In practice this is a 2-day
  serialization on what is otherwise a parallel plan. Acceptable given the risk
  reduction.

**Risks:**
- Risk: Phase 1.1 introduces a regression in estimation accuracy for normal inputs.
  Mitigation: regression test explicitly verifies "no behavioral change for
  non-outlier inputs" (the EMA convergence check in the PR checklist).
- Risk: The MAD outlier threshold (3× MAD) is wrong and dampens too aggressively.
  Mitigation: threshold is a constant (`OUTLIER_MAD_MULTIPLIER = 3.0`) that can
  be tuned via config. The test verifies convergence speed is not degraded for
  normal inputs with the default threshold.

### Implementation Notes

Ordering within Phase 1 when two developers are working in parallel:

```
Dev A:  [1.1 Robust Stats → 1 day] → [1.2 Metacog Monitor → 2 days]
Dev B:  [1.4 BLAKE3 Dedup → 1 day] → [1.3 Ebbinghaus Decay → 2 days]

Phase 2 gate: wait for 1.1 to merge before starting 2.1 Cascade Router
```

The Cascade Router's `reward_signal` function in
`crates/ironclaw_llm/src/cascade_router.rs` should import from
`crate::estimation::learner` for its cost signal, not recompute cost ratios
independently.

---

## ADR-004: Clean-Room Reimplementation — No Roko Source Imports

**Date**: 2026-07-03
**Status**: Accepted
**Deciders**: IronClaw maintainers
**Category**: Architecture

### Context

The Roko framework is a research codebase at `github.com/wpank/roko`. The
integration project began with a comprehensive analysis of Roko's source code
captured into 31 documents under `tmp/`. These documents include: algorithm
descriptions, data type definitions, Mermaid architecture diagrams, trait
signatures, test patterns, and academic citations.

The question of whether to depend on Roko as an upstream Rust crate or to
reimplement its concepts inside IronClaw-owned modules requires a clear answer
before any code is written.

### Decision

All Roko concepts are reimplemented inside IronClaw-owned modules using the
captured analysis documents as the specification. No Roko crate is imported as
a `[dependencies]` entry in any IronClaw `Cargo.toml`. Roko source file paths
appearing in the analysis documents (e.g.,
`crates/roko-primitives/src/robust.rs`) are treated as provenance labels — they
identify the original source of an algorithm for traceability but are not live
dependencies.

### Alternatives Considered

| Alternative | Reason Rejected |
|-------------|-----------------|
| Import Roko crates as upstream dependencies | Roko is a research codebase, not a stable library. Its API is subject to change. Adding it as a dependency would couple IronClaw's stability to Roko's development velocity. License compatibility would require audit. |
| Fork Roko and vendored-import selected crates | Fork maintenance overhead. Selected Roko crates have transitive dependencies that are unlikely to compile cleanly in IronClaw's dependency graph without conflicts. |
| Vendor individual files from Roko | Technically a copy, not a clean-room implementation. Would require attribution and license review. Does not allow the implementation to be adapted to IronClaw's types and patterns. |

### Consequences

**Positive:**
- IronClaw owns all new code completely; no upstream breakage risk.
- Implementations are adapted to IronClaw's exact types (`MemoryDocument`, `ToolDispatcher`, `SmartRoutingProvider`) rather than being forced into Roko's API shape.
- No license audit required.
- The analysis documents are a complete specification; no access to the Roko repo is needed after the initial capture.

**Negative / Trade-offs:**
- Some reimplementation effort for algorithms that could theoretically be imported directly (e.g., HoltForecast is ~30 lines). Accepted — the adaption to IronClaw's types is the value.
- Bug fixes or improvements in Roko's upstream implementations do not automatically flow into IronClaw. Mitigation: the analysis documents are versioned snapshots; revisit after major Roko releases if warranted.

### Implementation Notes

When the analysis documents describe a Roko algorithm, the implementation should:
1. Use the algorithm (trimmed mean, Holt smoothing, HDC XOR, LinUCB) exactly as specified.
2. Adapt the surrounding types to IronClaw conventions (error types via `thiserror`, logging via `tracing::debug!`, async via `tokio`).
3. Add the Roko source path as a provenance comment, not a live link:
   ```rust
   /// Holt double exponential smoothing.
   /// Algorithm reference: roko-conductor/src/holt.rs (captured 2026-06)
   /// Academic reference: Holt (1957) ONR Memorandum 52.
   ```
