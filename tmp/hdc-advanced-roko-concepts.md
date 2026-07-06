# Advanced HDC and Cognitive Architecture Design Inputs

This document covers Roko-derived design patterns that are NOT captured in
`tmp/hdc-roko-cognitive-usecases.md`. Each concept is rewritten as a
self-contained IronClaw design input. These are planned work, not current
behavior, unless explicitly labeled otherwise.

The companion document covers: store-native retrieval, context dedup,
four-factor scoring, admission gates, dream consolidation, counterfactual
transfer, code fingerprints, heartbeat clock, weighted index, HDC tool surface,
cross-domain resonance, and cognitive tier routing.

---

## 1. Engram Lineage and Derivation Tracking

**What it is.** Workspace documents carry a `lineage: Vec<ContentHash>` field
recording the content hashes of documents they were derived from. This enables
causal tracing: if document C synthesizes knowledge from A and B, C's lineage
lists A's and B's SHA-256 hashes. The hash is the existing SHA-256 computed by
`MemoryDocument` versioning.

**Why it matters for IronClaw.** IronClaw already stores per-document version
history in `memory_document_versions` with SHA-256 content hashes (within-document
evolution). Lineage adds cross-document derivation: a synthesized playbook knows
which source episodes it came from. This is required for taint propagation
(concept 13) and for safe promotion during dream consolidation (companion doc).

**Current behavior.** `MemoryDocument` has no `lineage` field. `memory_document_versions`
records SHA-256 of each version's content but has no cross-document pointer.

**Integration seam.** `src/workspace/document.rs` — add `lineage: Vec<String>` to
`DocumentMetadata.extra` via a typed field, or as a first-class field on
`MemoryDocument`. Database: new nullable JSONB/TEXT column on `memory_documents`
or stored in the existing `metadata` JSON field. Both backends must support it.

**Config/feature gate.** Behind the `hdc` feature flag. No separate env var needed
at first; `IRONCLAW_HDC_FINGERPRINT_SHADOW=true` is sufficient to enable shadow
recording of lineage.

**Fixture shape.**
```yaml
document:
  path: playbooks/db-migration.md
  lineage:
    - sha256: "a3f1..."   # episodes/thread-101.md at write time
    - sha256: "b72e..."   # episodes/thread-118.md at write time
  content: "Always write the shared DB trait before the backend..."
assertions:
  - lineage_resolved: true
  - each_hash_matches_existing_version: true
```

**Caller-level test requirement.** A test must drive `MemoryWriteTool` with
explicit lineage metadata, then read the document back and assert the lineage
field is persisted verbatim. Helper-only tests are insufficient.

**Benchmark/metric.** No measurable overhead expected (lineage is a metadata
write-through). Track: lineage roundtrip fidelity (hash matches source document
version at write time).

**Rollback/safety guardrail.** Lineage is append-only metadata. A document
without a lineage field behaves identically to today. Migration: nullable column,
no backfill required.

**Build order priority.** 2 (medium). Required before taint propagation (concept 13)
and heuristic falsification tracking (concept 6).

---

## 2. Demurrage (Gesellian Time-Decay Balance)

**What it is.** Each document carries an optional `balance: f64` in `[0.0, 1.0]`.
On every elapsed time unit (hours), the balance decays: `balance *= (1 - rate)^elapsed_hours`.
A `touch()` call resets balance to `1.0`. Documents with balance below a
configurable floor (e.g., `0.1`) are deprioritized in search results but not
deleted. This is distinct from Ebbinghaus confidence decay: demurrage is an
access-tax, not a rehearsal model.

**Why it matters for IronClaw.** Knowledge that is never retrieved or validated
gradually fades from active search consideration without being deleted. The
"LLM data is never deleted" invariant is preserved because balance only affects
ranking, not existence. This prevents stale advice from accumulating silently.

**Current behavior.** No balance field exists. Search result ranking is not
time-decay-aware beyond `updated_at` recency signals.

**Integration seam.**
- `src/workspace/document.rs`: optional `balance: Option<f64>` field on `MemoryDocument`.
- `src/workspace/mod.rs`: `touch(user_id, path)` resets balance to `1.0` and
  updates `updated_at`.
- `src/workspace/search.rs`: apply demurrage penalty to search result score when
  `balance < floor`.
- Database: nullable `REAL`/`DOUBLE` column on `memory_documents`.

**Config/feature gate.** `IRONCLAW_HDC_DEMURRAGE=off|shadow|active` (default: `off`).
Shadow mode computes and records balance on reads/writes without affecting ranking.

**Fixture shape.**
```json
{
  "scenario": "stale_advice_deprioritized",
  "document": {
    "path": "runbooks/old-approach.md",
    "balance": 0.08,
    "last_touched_days_ago": 45
  },
  "query": "deployment approach",
  "assertions": {
    "appears_in_results": true,
    "rank_vs_fresh_doc": "below",
    "balance_floor": 0.1
  }
}
```

**Caller-level test requirement.** A test must drive `memory_search` via the
`Workspace` caller, not just the balance computation helper, and assert that a
low-balance document appears below a high-balance document with equal semantic
similarity.

**Benchmark/metric.** Track: distribution of balance values across `memory_documents`
after 30 days of production use. Alert if >20% of documents are below the floor
(may indicate too-aggressive a rate).

**Rollback/safety guardrail.** Balance is a nullable ranking hint. Removing the
feature: set `IRONCLAW_HDC_DEMURRAGE=off`, no migration needed. The column
stays; the ranking code simply ignores it.

**Build order priority.** 3 (later). Requires search result score compositing to
be stable. Can be added independently of HDC vectors.

---

## 3. Two-Stage Evidence-Based Admission

**What it is.** A two-stage gate between `memory_write` and durable workspace
storage. Stage 1 (LightAdmissionGate): fast check — minimum confidence (0.5),
minimum novelty (0.3), minimum source trust (0.65). Stage 2
(KnowledgeAdmissionStore): accumulates evidence from multiple sources with
trust weights before promoting a candidate to durable memory. Source trust
weights: UserInput=1.0, GateOutcome=0.95, AgentOutput=0.75, DreamConsolidation=0.45.
Candidates that pass Stage 1 but lack sufficient evidence land in a staging area
(e.g., `staging/` workspace prefix) before promotion.

**Why it matters for IronClaw.** Single-agent claims can be wrong. Requiring
multiple independent confirmations before durable promotion prevents garbage
memory accumulation. The staging area is itself workspace-backed and auditable.

**Current behavior.** `MemoryWriteTool` runs a single HDC dedup preflight
(`Workspace::check_dedup`) when `IRONCLAW_HDC_DEDUP_MODE=warn|block`. There is
no multi-source evidence accumulation.

**Integration seam.**
- `src/tools/builtin/memory.rs`: admission layer inserted before the durable
  workspace write.
- `src/workspace/mod.rs`: new `promote_staged(user_id, staging_path, final_path)`
  method.
- New module: `src/workspace/admission.rs` (light gate + evidence accumulator).

**Config/feature gate.** `IRONCLAW_HDC_ADMISSION_MODE=off|shadow|advisory` (default: `off`).
- `off`: current behavior unchanged.
- `shadow`: gates run and log decisions without blocking writes.
- `advisory`: gate result returned to caller; caller decides whether to write or
  stage.

**Fixture shape.**
```json
{
  "scenario": "multi_source_promotion",
  "candidate": {
    "path": "candidates/migrations/approach-f.md",
    "confidence": 0.72,
    "source_trust": 0.75,
    "novelty_score": 0.45
  },
  "evidence": [
    {"source": "UserInput", "trust": 1.0, "confirms": true},
    {"source": "GateOutcome", "trust": 0.95, "confirms": true}
  ],
  "expected": {
    "stage1_pass": true,
    "stage2_decision": "promote",
    "staging_path": null
  }
}
```

**Caller-level test requirement.** Tests must drive `MemoryWriteTool` at the
integration tier (`cargo test --features integration`) to assert that: (a) a
low-evidence candidate is written to `staging/` not durable memory, and (b) a
subsequent evidence event promotes it.

**Benchmark/metric.** Track: staging dwell time (time from first evidence to
promotion), false-promotion rate (staged items later demoted), and evidence
accumulation latency.

**Rollback/safety guardrail.** `off` mode is the default. The staging prefix
`staging/` is never injected into the system prompt. Staged documents are not
searchable by default.

**Build order priority.** 2 (medium). Requires lineage tracking (concept 1) for
staged-to-durable promotion audit trail.

---

## 4. Tier Progression D1 to D2 to D3

**What it is.** Three-stage distillation run as an offline batch job:
- **D1 (episodes → insights):** raw completed threads distilled into insight
  documents. Minimum: 3 supporting episodes, minimum confidence 0.7.
- **D2 (insights → heuristics):** insights distilled into actionable heuristics.
  Minimum: 5 supporting episodes, with calibration actions (Confirm, Violate,
  Refine, Generalize, Refute). Promotion requires 3 passing gate checks;
  demotion requires 2 failing.
- **D3 (heuristics → PLAYBOOK.md):** top 12 heuristics by support score written
  to a durable `PLAYBOOK.md` in workspace.

**Why it matters for IronClaw.** The dream consolidation pattern in the companion
doc uses HDC for cluster grouping but lacks a promotion discipline. D1→D2→D3
gives promotion and demotion rules with explicit evidence thresholds, preventing
playbook pollution.

**Current behavior.** No tier progression job exists. Dream consolidation is
described as future work. `PLAYBOOK.md` is not a defined workspace path.

**Integration seam.**
- New offline job: `src/agent/jobs/tier_progression.rs` (or a heartbeat routine).
- `src/workspace/document.rs`: add `playbook_path = "PLAYBOOK.md"` to `paths`.
- Database: `metadata` field on `memory_documents` stores tier level (`d1`, `d2`,
  `d3`) and calibration counts.

**Config/feature gate.** `IRONCLAW_HDC_TIER_PROGRESSION=off|shadow|advisory`
(default: `off`). Shadow records proposed tier transitions without writing them.

**Fixture shape.**
```yaml
scenario: d1_to_d2_promotion
episodes:
  - thread_id: t-101
    task_type: db_parity_fix
    outcome: success
    confidence: 0.82
  - thread_id: t-118
    task_type: db_parity_fix
    outcome: success
    confidence: 0.77
  - thread_id: t-141
    task_type: db_parity_fix
    outcome: success
    confidence: 0.79
expected_insight:
  path: insights/db-parity-fix.md
  tier: d1
  min_support_count: 3
  promoted: true
```

**Caller-level test requirement.** An integration test must drive the tier
progression job against a populated workspace (3+ episode documents), assert
an insight is written to `insights/`, and assert no D3 promotion occurs without
the required 5-episode minimum.

**Benchmark/metric.** Track: promotion rate per task family, demotion frequency,
and playbook churn (how often PLAYBOOK.md entries change per week).

**Rollback/safety guardrail.** D3 writes to `PLAYBOOK.md` only in advisory mode
after user or operator confirmation. In shadow mode, the proposed PLAYBOOK.md is
written to `staging/playbook-candidate.md` instead.

**Build order priority.** 3 (later). Depends on dream consolidation (companion
doc) and heuristic falsification tracking (concept 6).

---

## 5. Kind-Specific Half-Lives

**What it is.** Six knowledge kinds with distinct decay half-lives:
- `Insight`: 30 days
- `Heuristic`: 90 days
- `Warning`: 1 hour
- `CausalLink`: 60 days
- `StrategyFragment`: 14 days
- `AntiKnowledge`: 30 days

Combined with four tier multipliers:
- `Transient`: 0.1x (short-lived scratch)
- `Working`: 0.5x
- `Consolidated`: 1.0x (baseline)
- `Persistent`: 5.0x (long-lived pinned knowledge)

Effective half-life = `kind_half_life * tier_multiplier`. Deprioritization in
search results uses the elapsed fraction of half-life, not deletion.

**Why it matters for IronClaw.** A `Warning` about a failing test should not
persist for 30 days. An `Insight` from a successful architectural decision should
not decay as fast as a transient scratch note. Kind + tier gives fine-grained
relevance control without extra tooling.

**Current behavior.** `DocumentMetadata` has no `kind` or `tier` fields. Search
ranking does not apply kind-specific decay.

**Integration seam.**
- `src/workspace/document.rs`: add `kind: Option<KnowledgeKind>` and
  `tier: Option<KnowledgeTier>` to `DocumentMetadata`.
- `src/workspace/search.rs`: apply half-life decay to search result score when
  kind is set.
- New enum types `KnowledgeKind` and `KnowledgeTier` in `document.rs`.

**Config/feature gate.** Activated by `IRONCLAW_HDC_DEMURRAGE=active` (shares gate
with concept 2) or a standalone `IRONCLAW_KNOWLEDGE_DECAY=true`.

**Fixture shape.**
```json
{
  "documents": [
    {
      "path": "warnings/test-failure.md",
      "kind": "Warning",
      "tier": "Working",
      "created_hours_ago": 3
    },
    {
      "path": "insights/arch-decision.md",
      "kind": "Insight",
      "tier": "Consolidated",
      "created_hours_ago": 3
    }
  ],
  "query": "current issues",
  "assertions": {
    "warning_deprioritized_after_threshold": true,
    "insight_still_ranked_high": true
  }
}
```

**Caller-level test requirement.** A test must drive `memory_search` and assert
that a `Warning` document with 2-hour age ranks below an `Insight` with 2-hour
age for a neutral query, while a `Warning` with 0.5-hour age ranks above both.

**Benchmark/metric.** Track: half-life expiry rate per kind per day. Alert if
`Warning` documents accumulate beyond 6 hours (likely not being consumed).

**Rollback/safety guardrail.** Kind and tier fields are optional metadata. Without
them, ranking is unchanged. Setting `IRONCLAW_KNOWLEDGE_DECAY=false` disables
the decay penalty without schema changes.

**Build order priority.** 3 (later). Shares implementation surface with concept 2
(demurrage).

---

## 6. Heuristic Falsification Tracking

**What it is.** Each heuristic document (D2-tier from concept 4) tracks
validation events as a list of `CalibrationAction` items: `Confirm`, `Violate`,
`Refine`, `Generalize`, or `Refute`. Promotion requires 3 passing gate checks
(`PROMOTION_SUCCESS_THRESHOLD=3`). Demotion to D1 or deletion from PLAYBOOK.md
requires 2 consecutive or recent failing checks (`DEMOTION_FAILURE_THRESHOLD=2`).
At 2x the kind half-life (`EXPIRY_REVIEW`), the heuristic is flagged for review
regardless of calibration count.

**Why it matters for IronClaw.** Heuristics derived from 5 successful episodes
can later be invalidated by a change in architecture or toolchain. Tracking
`Violate` and `Refute` events prevents stale guidance from persisting in
PLAYBOOK.md past its useful life.

**Current behavior.** No calibration tracking exists on workspace documents.

**Integration seam.**
- `src/workspace/document.rs`: add `calibration_log: Vec<CalibrationEvent>` to
  `DocumentMetadata` (stored in the `extra` JSON field or a dedicated column).
- `src/workspace/mod.rs`: `record_calibration(user_id, path, action, evidence)`.
- Tier progression job (concept 4) reads calibration log before D2 promotion and
  D3 inclusion.

**Config/feature gate.** `IRONCLAW_HDC_TIER_PROGRESSION=advisory|active`. Not
useful without tier progression.

**Fixture shape.**
```json
{
  "heuristic_path": "heuristics/db-parity.md",
  "calibration_log": [
    {"action": "Confirm", "thread_id": "t-201", "timestamp": "2026-06-01"},
    {"action": "Confirm", "thread_id": "t-218", "timestamp": "2026-06-10"},
    {"action": "Violate", "thread_id": "t-225", "timestamp": "2026-06-15"},
    {"action": "Violate", "thread_id": "t-231", "timestamp": "2026-06-20"}
  ],
  "expected": {
    "promotion_blocked": true,
    "demotion_triggered": true,
    "reason": "two_consecutive_failures"
  }
}
```

**Caller-level test requirement.** A test must call `record_calibration` twice
with `Violate` actions and then assert that the tier progression job demotes the
heuristic from D2 to D1 in the next run.

**Benchmark/metric.** Track: demotion rate per task family, average calibration
events per promoted heuristic, and PLAYBOOK.md churn.

**Rollback/safety guardrail.** Calibration log is append-only. Demotion is
advisory by default: writes to `staging/demotion-candidates.md` before
modifying PLAYBOOK.md.

**Build order priority.** 3 (later). Depends on tier progression (concept 4).

---

## 7. Role-Filler HDC Binding for Structured Knowledge

**What it is.** Instead of encoding only flat text, structured facts are encoded
with directional role binding: `bind(role_vec("cause"), text_hv("failed migration"))`.
The composite vector encodes both the fact and its role. Unbinding recovers
fillers: `composite XOR role_vector`. `CausalLink` documents use distinct
permutation shifts for `cause` vs `effect` roles to prevent conflation.

**Why it matters for IronClaw.** Flat text encoding of "A causes B" and "B causes A"
produce similar vectors. Role binding makes the direction distinguishable. This
is the foundation for accurate counterfactual transfer (companion doc) and taint
propagation (concept 13).

**Current behavior.** `encode_document` uses weighted bundle of content trigrams,
tag role-bindings, and path segments. `encode_structured(fields: &[(&str, &str)], codebook)` and
`encode_causal_link(cause, effect, codebook)` already exist in `crates/ironclaw_hdc/src/encoder.rs`
and are re-exported from `lib.rs`.

**Integration seam.** The primitive already exists:
- `crates/ironclaw_hdc/src/encoder.rs`: `encode_structured` and `encode_causal_link` are implemented.
- `crates/ironclaw_hdc/src/lib.rs`: both are re-exported.
- Remaining work: caller-level tests in `crates/ironclaw_hdc/tests/encoding.rs` and a golden test
  covering `encode_structured` output to prevent seed drift.

**Config/feature gate.** Behind the `hdc` feature flag. No separate env var.
The function is always available when `hdc` is enabled; callers opt in by using it.

**Fixture shape.**
```yaml
scenario: causal_link_directionality
inputs:
  - role: "cause"
    value: "failed migration after schema drift"
  - role: "effect"
    value: "libsql write error on version table"
assertions:
  cause_effect_similarity_lt: 0.6
  # Reversing roles produces a clearly different vector
  reversed_similarity_lt: 0.65
  # Same cause, different effect: lower similarity than same cause/effect
  different_effect_similarity_lt: 0.7
```

**Caller-level test requirement.** A unit test in `crates/ironclaw_hdc/src/encoder.rs`
must assert that `encode_structured([("cause", X), ("effect", Y)])` produces a
vector with similarity < 0.6 to `encode_structured([("cause", Y), ("effect", X)])`.
A second test must assert the function is deterministic across calls.

**Benchmark/metric.** Encoding latency for 10-field structured documents: target
< 5ms per document (consistent with existing `encode_document` baseline).

**Rollback/safety guardrail.** New public function; no existing behavior changes.
Golden test must cover `encode_structured` output to prevent seed drift.

**Build order priority.** 2 (medium). Prerequisite for k-medoids clustering
(concept 8) and taint propagation (concept 13).

---

## 8. K-Medoids Clustering over HDC Vectors

**What it is.** PAM (Partitioning Around Medoids) clustering using Hamming distance
(`1.0 - similarity`) as the metric. Seeds are chosen by farthest-first initialization
(deterministic, no RNG). Clusters past episode fingerprints or workspace documents
into behavioral groups for pattern discovery. Returns medoid IDs and cluster
assignments; does not mutate workspace state.

**Why it matters for IronClaw.** Dream consolidation (companion doc) uses
hand-written task-type keys for clustering. HDC clustering detects structural
similarity without requiring explicit task labels. It groups episodes by behavioral
fingerprint, which can surface novel task families that were not anticipated.

**Current behavior.** `crates/ironclaw_hdc/src/cluster.rs` exists with `k_medoids<Id>(vectors: &[(Id, HdcVector)], k: usize) -> Result<Vec<Cluster<Id>>, HdcError>` and `struct Cluster<Id> { medoid_id, members, inertia }`. This concept describes building offline job integrations on top of that primitive, not the primitive itself.

**Integration seam.** The module already exists:
- `crates/ironclaw_hdc/src/cluster.rs` — implemented.
- Public API: `k_medoids(vectors: &[(Id, HdcVector)], k: usize) -> Result<Vec<Cluster<Id>>, HdcError>`.
- `struct Cluster<Id> { medoid_id: Id, members: Vec<Id>, inertia: f64 }` — implemented.
- Remaining work: wire it into offline dream consolidation and memory audit jobs. Not called from
  the live agent loop.

**Config/feature gate.** Behind the `hdc` feature flag. Exposed only to offline
batch jobs; not a user-facing tool.

**Fixture shape.**
```yaml
scenario: episode_clustering
inputs:
  - id: t-101
    fingerprint_hex: "..."   # db_parity_fix episode
  - id: t-118
    fingerprint_hex: "..."   # db_parity_fix episode
  - id: t-205
    fingerprint_hex: "..."   # auth_route_refactor episode
k: 2
assertions:
  cluster_0_members: [t-101, t-118]
  cluster_1_members: [t-205]
  deterministic: true   # same input, same output, no RNG
```

**Caller-level test requirement.** A unit test must verify: (a) determinism across
multiple calls with identical input, (b) medoid IDs are always members of their
own cluster, and (c) k > number of inputs returns an error.

**Benchmark/metric.** `k_medoids` over 1,000 vectors with k=10: target < 500ms
(brute-force PAM is O(k * n^2) per iteration). Track in Criterion.

**Rollback/safety guardrail.** Clustering is read-only. It reads fingerprints from
workspace but writes nothing. Safe to disable by not calling from batch jobs.

**Build order priority.** 2 (medium). Used in dream consolidation follow-up and
episode pattern discovery (concept 9).

---

## 9. Episode Fingerprinting Per Agent Turn

**What it is.** Every completed agent turn gets an HDC fingerprint computed from
a normalized representation of the prompt (query) and outcome (final response
summary). The fingerprint is stored alongside the turn record. During subsequent
turns, a template suggestion step finds past episodes with HDC similarity >= 0.7
within a 30-day window and surfaces them as candidate context.

**Why it matters for IronClaw.** Today, completed turns are stored in database
tables but are not fingerprinted. Without fingerprints, finding structurally
similar past episodes requires expensive full-text search on large turn records.
HDC similarity provides a fast structural lookup before any LLM call.

**Current behavior.** Turn records exist in the database. No HDC fingerprint is
computed or stored per turn.

**Integration seam.**
- `src/agent/agent_loop.rs`: after a turn completes, compute episode fingerprint
  and store alongside the turn record.
- `src/db/libsql/mod.rs` and `src/db/postgres.rs`: nullable `hdc_fingerprint`
  column on the turns/events table (or stored in turn metadata).
- Episode retrieval: new `Workspace::similar_episodes(query_hv, window_days, threshold)`
  that scans stored episode fingerprints.

**Config/feature gate.** `IRONCLAW_HDC_EPISODE_PATTERNS=true` (default: false).
Behind the `hdc` feature flag.

**Fixture shape.**
```json
{
  "scenario": "similar_episode_retrieval",
  "stored_episodes": [
    {
      "turn_id": "t-101",
      "prompt_summary": "fix libsql migration parity",
      "outcome_summary": "wrote shared trait, added parity tests"
    }
  ],
  "query": "libsql parity fix after schema drift",
  "threshold": 0.7,
  "window_days": 30,
  "expected_hits": ["t-101"],
  "expected_miss": ["t-205"]
}
```

**Caller-level test requirement.** An integration test must run the agent loop on
two distinct prompts, assert that the second prompt retrieves the first episode
when similarity exceeds the threshold, and assert no retrieval when similarity
is below threshold.

**Benchmark/metric.** Episode fingerprint scan over 10,000 stored turns: target
< 50ms. Track: hit rate (fraction of turns where a similar past episode exists)
and false-positive rate (retrieved episode from a different task family).

**Rollback/safety guardrail.** Feature is default-off. Disabling
`IRONCLAW_HDC_EPISODE_PATTERNS` stops fingerprint computation and retrieval;
stored fingerprints are inert.

**Build order priority.** 2 (medium). Can be built independently of clustering.

---

## 10. Pheromone Coordination

**Implementation Status.** DEFERRED. No inter-session signaling layer exists.
Scope promotion (Local→Subnet→Mesh→Global) requires multi-session workspace
semantics that are not built. The Alpha anti-herding mechanic and HDC diversity
gate on Consensus signals are speculative until basic signal deposit/read is
demonstrated. Do not implement until episode fingerprinting (concept 9) and
subagent goal dedup (integration doc Section 14) have produced measurable data.
The `signals/` workspace prefix design can be prototyped independently of HDC.

**What it is.** Bio-inspired typed signals deposited in workspace and readable by
concurrent sessions. Six pheromone kinds with distinct half-lives:

| Kind | Half-life |
|------|-----------|
| Threat | 2 hours |
| Opportunity | 8 hours |
| Wisdom | 24 hours |
| Alpha | 12 hours (shortens with confirmations — anti-herding) |
| Pattern | 6 hours |
| Consensus | 4 hours |

Signals decay exponentially. Scope promotion: `Local → Subnet → Mesh → Global`
as independent confirmations increase. Alpha pheromone half-life decreases with
confirmations to prevent herding (anti-Alpha paradox). HDC vector diversity is a
quality gate on Consensus signals: if all contributing sessions have similar HDC
fingerprints, the consensus is suspect (homogeneous input).

**Why it matters for IronClaw.** In multi-session or multi-agent deployments,
one session discovering a `Threat` (e.g., a failing integration test) can signal
other sessions to avoid a code path. `Wisdom` signals share distilled insights
across sessions without requiring them to run full retrieval.

**Current behavior.** No inter-session signaling mechanism exists in workspace.

**Integration seam.**
- New workspace prefix: `signals/` (e.g., `signals/threat/2026-07-06T14:00:00Z.json`).
- New module: `src/workspace/signals.rs` with `PheromoneKind`, `Pheromone`,
  `deposit(user_id, kind, content, scope)`, `read_active(user_id, kind)`.
- Decay computed at read time from `created_at` and kind half-life. Documents
  below `0.05` balance are excluded from `read_active` results but not deleted.
- `Workspace::read_active` filters by scope (only `Global` signals visible across
  user IDs; `Local` signals are user-scoped).

**Config/feature gate.** `IRONCLAW_HDC_COORDINATION_MODE=off|shadow|active`
(default: `off`). Shadow mode records signals but does not expose them to other
sessions.

**Fixture shape.**
```json
{
  "scenario": "threat_signal_propagation",
  "deposits": [
    {
      "kind": "Threat",
      "content": "libsql migration V33 fails on row insert",
      "scope": "Local",
      "deposited_hours_ago": 1
    }
  ],
  "read_after_hours": 1,
  "assertions": {
    "signal_active": true,
    "scope": "Local"
  },
  "read_after_hours_3": {
    "signal_active": false,
    "reason": "half_life_expired"
  }
}
```

**Caller-level test requirement.** Tests must drive `deposit` and `read_active`
through workspace methods (not raw DB calls) and assert: (a) signals expire after
their kind half-life, (b) `Local` signals are not visible to a different `user_id`,
and (c) `Global` signals are visible across user IDs.

**Benchmark/metric.** `read_active` scan over 1,000 signals: target < 10ms.
Track: signal utilization rate (fraction of deposited signals that are ever read
by another session).

**Rollback/safety guardrail.** `signals/` prefix is excluded from the system
prompt and from `memory_search` by default (treat like `.system/`). Setting
`IRONCLAW_HDC_COORDINATION_MODE=off` stops deposit and retrieval; the `signals/`
documents are inert.

**Build order priority.** 3 (later). Requires stable multi-session workspace
semantics.

---

## 11. Mattar-Daw Prioritized Replay

**Implementation Status.** DEFERRED. Dream consolidation does not exist yet.
The `compute_utility` function is self-contained arithmetic and can be
implemented as a pure function in `ironclaw_hdc::replay` without touching the
agent loop or workspace. However, it provides no value until: (a) episode
fingerprints exist (concept 9), (b) there is an offline consolidation job to
plug it into. Implement `compute_utility` as a documented stub alongside
concept 9; do not wire it to any job until the dream consolidation job exists.

**What it is.** A priority function for selecting which episodes to replay during
offline consolidation (dream job). Priority: `utility = gain × need × spacing_inv`.
- `gain`: prediction error, i.e., how surprising the outcome was (novel + unexpected
  results score higher).
- `need`: policy relevance — novel task types and recent episodes score higher
  than stale familiar ones.
- `spacing_inv`: spaced-repetition inverse — episodes not recently replayed score
  higher than those replayed in the last consolidation cycle.

The dream job replays the top-N episodes by utility rather than processing all
completed threads.

**Why it matters for IronClaw.** The dream consolidation companion concept
processes all completed threads uniformly. Prioritized replay focuses
consolidation budget on the most informative episodes: surprising outcomes and
underexplored task types.

**Current behavior.** No dream consolidation job exists. No replay priority is
computed.

**Integration seam.**
- Dream consolidation job (future work, see companion doc): replace the uniform
  episode iterator with a priority queue using `utility` scores.
- New function: `ironclaw_hdc::replay::compute_utility(gain, need, spacing_inv) -> f64`.
- Episode metadata (stored in `metadata` JSON): `last_replayed_at` timestamp,
  `replay_count`, `prediction_error_score`.

**Config/feature gate.** `IRONCLAW_HDC_TIER_PROGRESSION=advisory|active`. Replay
priority is only meaningful when dream consolidation runs.

**Fixture shape.**
```yaml
scenario: prioritized_replay_ordering
episodes:
  - id: t-101
    gain: 0.9        # very surprising outcome
    need: 0.8        # novel task type
    spacing_inv: 0.7 # not replayed recently
    expected_utility: 0.504   # 0.9 * 0.8 * 0.7
  - id: t-118
    gain: 0.3
    need: 0.5
    spacing_inv: 0.9
    expected_utility: 0.135
expected_replay_order: [t-101, t-118]
```

**Caller-level test requirement.** The dream consolidation job test must use a
fixture with 10 episodes of varying utility and assert the top-5 processed match
the expected priority order.

**Benchmark/metric.** Utility computation for 10,000 episodes: target < 5ms
(simple arithmetic, no HDC encoding needed for the priority step itself).

**Rollback/safety guardrail.** If `gain`, `need`, or `spacing_inv` are not
available for an episode, fall back to recency ordering (current behavior).
Utility is advisory; episodes are not permanently excluded.

**Build order priority.** 3 (later). Depends on tier progression and dream
consolidation.

---

## 12. DAG Execution with File-Overlap Inference

**Implementation Status.** NOT an HDC proposal. This concept is independent of
HDC vectors entirely — the config gate is `IRONCLAW_ROUTINE_DAG`, not any HDC
env var. It belongs in a routine/scheduler track, not in this HDC document.
It addresses a real concurrency hazard (two heartbeat routines writing the
same daily log). If prioritized, it should be tracked as a scheduler
improvement, not as an HDC integration. Keeping the design here for reference;
do not implement under an HDC umbrella.

**What it is.** A cross-routine/job DAG that adds serialization edges when two
tasks access the same workspace document path. BFS waves identify which tasks
can run in parallel (no shared file dependencies) and which must be serialized.
An atomic reserve step claims a file before execution; a ready batch releases
the claim on completion. File-conflict-aware merge queue prevents concurrent
writes to the same workspace path.

**Why it matters for IronClaw.** The routine/job scheduler (`src/agent/`) can
schedule multiple routines that may write to the same workspace document (e.g.,
two heartbeat routines both updating `daily/YYYY-MM-DD.md`). Concurrent writes
cause TOCTOU races and content loss. File-overlap inference converts a potential
race into an explicit serialization edge.

**Current behavior.** The job scheduler does not analyze workspace path dependencies
between concurrent jobs. Concurrent writes to the same document rely on
database-level locking (present in both backends) but can still produce logical
content conflicts.

**Integration seam.**
- `src/agent/mod.rs` or new `src/agent/scheduler.rs`: DAG builder that reads
  declared workspace paths from routine metadata and adds serialization edges.
- `src/workspace/mod.rs`: `reserve_path(user_id, path, job_id)` returning a
  `PathReservation` guard that releases on drop.
- Routines declare their expected write paths in their manifest or config.

**Config/feature gate.** `IRONCLAW_ROUTINE_DAG=off|active` (default: `off`).
No HDC required; this concept is independent of HDC vectors.

**Fixture shape.**
```yaml
scenario: concurrent_daily_log_write
routines:
  - id: routine-a
    writes: ["daily/2026-07-06.md"]
  - id: routine-b
    writes: ["daily/2026-07-06.md"]
  - id: routine-c
    writes: ["MEMORY.md"]
expected_dag:
  parallel_pairs: [[routine-a, routine-c], [routine-b, routine-c]]
  serialized_pairs: [[routine-a, routine-b]]
```

**Caller-level test requirement.** An integration test must launch two routines
with overlapping write paths and assert: (a) one holds the reservation while the
other waits, and (b) final document content is the union of both writes (no
content lost).

**Benchmark/metric.** DAG construction over 100 routines with 50% path overlap:
target < 5ms. Track: reservation wait time distribution.

**Rollback/safety guardrail.** Without `IRONCLAW_ROUTINE_DAG=active`, current
behavior is unchanged. Path reservation errors are non-fatal: the routine logs a
warning and proceeds (degraded, not broken).

**Build order priority.** 2 (medium). Independent of HDC. Addresses a real
concurrency hazard in routine scheduling.

---

## 13. Taint Propagation

**What it is.** A content-hash-keyed taint tracker. If any parent document in a
signal's lineage (concept 1) is marked tainted, the derived document inherits
taint. Taint categories: `External` (data from outside the system), `UserInput`
(verbatim user text, not validated), `Propagated` (inherited via lineage). Sink
operations (git commits, network egress tools, workspace promotion to durable
memory) consult `is_tainted(content_hash)` before proceeding. Tainted data
requires explicit clearance before reaching sinks.

**Why it matters for IronClaw.** The system ingests external content (web fetch,
file read from untrusted paths, MCP tool outputs). Without taint tracking, a
synthesis document derived from untrusted input could be promoted to PLAYBOOK.md
or injected into a system prompt without notice. Taint tracking closes this gap
without requiring content scanning on every write.

**Current behavior.** No taint tracking exists. Safety validation
(`ironclaw_safety`) runs on tool inputs at call time but does not propagate taint
through derived documents.

**Integration seam.**
- New module: `src/workspace/taint.rs`. `TaintTracker` stores `HashMap<ContentHash, TaintRecord>`.
- `src/tools/builtin/memory.rs`: when writing a document derived from tainted
  sources (external tool output), call `taint_tracker.mark(content_hash, External)`.
- `src/workspace/mod.rs`: `check_taint(content_hash) -> Option<TaintRecord>` for
  sink operations.
- Lineage (concept 1) is required: taint propagation traverses lineage hashes.
- Sinks: `memory_write` to `PLAYBOOK.md`, `memory_write` to `AGENTS.md`,
  network egress tools (`http`, `web_fetch`).

**Config/feature gate.** `IRONCLAW_TAINT_TRACKING=off|warn|block` (default: `off`).
- `warn`: tainted sink operations log a warning and proceed.
- `block`: tainted sink operations are rejected with an explanation.

**Fixture shape.**
```json
{
  "scenario": "external_content_reaches_playbook",
  "source": {
    "tool": "web_fetch",
    "url": "https://example.com/untrusted",
    "content": "Always use TRUNCATE instead of DELETE for performance."
  },
  "derived_document": {
    "path": "candidates/db-advice.md",
    "lineage": ["sha256_of_web_fetch_output"]
  },
  "target_sink": "PLAYBOOK.md",
  "expected": {
    "taint_check": "tainted",
    "category": "External",
    "sink_blocked": true
  }
}
```

**Caller-level test requirement.** An integration test must: (a) call `web_fetch`
(which marks its output tainted), (b) write a derived document with lineage
pointing to the fetched content, (c) attempt to promote the derived document to
`PLAYBOOK.md` with `IRONCLAW_TAINT_TRACKING=block`, and assert the promotion
is rejected.

**Benchmark/metric.** Taint lookup for 100,000 entries: target < 1ms (hash map
lookup). Track: false-positive taint blocks (documents incorrectly flagged as
tainted) via a canary of known-clean documents.

**Rollback/safety guardrail.** Default is `off`. Setting to `warn` introduces
no blocking behavior. The taint store is in-memory by default; restart clears it.
Persistent taint (across restarts) requires a database column and is a separate
feature.

**Build order priority.** 2 (medium). Requires lineage tracking (concept 1).
High safety value relative to implementation cost.

---

## 14. Bayesian Confidence Updater

**What it is.** A Beta-Binomial conjugate model for tracking per-tool or per-gate
reliability. Each tracked subject starts with a prior `Beta(α=1, β=1)` (uniform).
`observe(success: bool)` updates `α += 1` on success, `β += 1` on failure.
`confidence() = α / (α + β)`. `credible_interval(0.95)` returns the 95% HDI
using the Beta distribution. Applied to: tool success rates, admission gate
accuracy, and calibration gate pass rates.

**Why it matters for IronClaw.** Tool selection in the agent loop uses static
configuration or LLM judgment. A tool that fails 40% of the time should be
deprioritized or flagged without requiring manual configuration. The Beta-Binomial
model is simple, requires no external ML library, and provides calibrated
uncertainty rather than a raw success count.

**Current behavior.** Tool execution outcomes are recorded in `ActionRecord` but
no per-tool reliability model is maintained. Tool selection does not use
historical success rates.

**Integration seam.**
- New module: `crates/ironclaw_hdc/src/bayes.rs` (or `src/tools/reliability.rs`).
- `BayesianTracker { alpha: f64, beta: f64 }` with `observe(bool)`, `confidence()`,
  `credible_interval(coverage: f64) -> (f64, f64)`.
- `src/tools/registry.rs`: per-tool `BayesianTracker` updated after each
  `ToolDispatcher::dispatch()` call.
- Tool selection: `ToolRegistry::ranked_tools(task_context)` includes reliability
  score as a tiebreaker when multiple tools match.

**Config/feature gate.** `IRONCLAW_TOOL_RELIABILITY_TRACKING=true` (default: false).
No HDC vectors involved; does not require the `hdc` feature flag.

**Fixture shape.**
```json
{
  "scenario": "tool_reliability_after_failures",
  "tool": "http",
  "observations": [
    {"success": true},
    {"success": true},
    {"success": false},
    {"success": false},
    {"success": false}
  ],
  "expected": {
    "confidence": 0.375,
    "alpha": 3,
    "beta": 4,
    "credible_interval_95": [0.09, 0.72]
  }
}
```

**Caller-level test requirement.** A unit test must verify the Beta-Binomial
update math for known sequences. An integration test must drive two tool
dispatches (one succeeding, one failing) through `ToolDispatcher` and assert
the tracker state is updated for the correct tool name.

**Benchmark/metric.** `observe` + `confidence` for 10,000 updates: target < 1ms
total (arithmetic operations). Track: per-tool confidence distribution in
production; alert if any critical tool falls below `0.5` confidence.

**Rollback/safety guardrail.** Tracker state is in-memory and resets on restart.
A tool with low confidence is deprioritized but never disabled automatically
without explicit configuration. `IRONCLAW_TOOL_RELIABILITY_TRACKING=false`
restores current behavior with no state changes.

**Build order priority.** 2 (medium). Independent of HDC vectors. Directly
improves tool selection quality with minimal implementation cost.

---

## Build Order Summary

| Priority | Concept | Blocking | Notes |
|----------|---------|---------|-------|
| 2 (medium) | 1. Engram lineage | Blocks: 13 (taint), 6 (falsification) | |
| 2 (medium) | 3. Two-stage admission | Standalone, uses existing `check_dedup` | |
| 2 (medium) | 7. Role-filler binding | Blocks: 8 (clustering), 13 (taint) | |
| 2 (medium) | 8. K-medoids clustering | Requires: 7 | |
| 2 (medium) | 9. Episode fingerprinting | Standalone | |
| 2 (medium) | 13. Taint propagation | Requires: 1 | |
| 2 (medium) | 14. Bayesian tracker | Standalone | |
| 3 (later) | 2. Demurrage | Standalone | |
| 3 (later) | 4. Tier progression | Requires: dream consolidation (companion doc) | |
| 3 (later) | 5. Kind-specific half-lives | Requires: 2 | |
| 3 (later) | 6. Heuristic falsification | Requires: 4 | |
| DEFERRED | 10. Pheromone coordination | Requires: stable multi-session workspace | No inter-session signaling layer exists; no measured baseline from episode dedup |
| DEFERRED | 11. Mattar-Daw replay | Requires: 4, dream consolidation | Implement `compute_utility` as stub alongside concept 9; do not wire until dream job exists |
| NOT HDC | 12. DAG execution | Standalone, no HDC dependency | Routine scheduler hazard, not an HDC integration; track separately |

## Relationship to Existing IronClaw HDC Work

Current live behavior (as of the HDC PR on branch `wp-extensions`):

- `hdc_fingerprint: Option<Vec<u8>>` on `MemoryDocument` (shadow mode).
- `IRONCLAW_HDC_FINGERPRINT_SHADOW`, `IRONCLAW_HDC_DEDUP_MODE`, `IRONCLAW_HDC_SIMILAR_THRESHOLD`,
  `IRONCLAW_HDC_DUPLICATE_THRESHOLD`, `IRONCLAW_HDC_SEARCH_SHADOW`,
  `IRONCLAW_HDC_HEARTBEAT_OBSERVE` env vars.
- `Workspace::check_dedup` as the single dedup primitive.
- `MemoryWriteTool` HDC preflight for `warn|block` dedup modes.
- `HEARTBEAT_HDC_ENABLED`, `HEARTBEAT_HDC_DECAY_FACTOR`, `HEARTBEAT_HDC_REPEATED_THRESHOLD`,
  `HEARTBEAT_HDC_NOVEL_THRESHOLD`, `HEARTBEAT_HDC_SUPPRESS_REPEATED` for heartbeat novelty.
- `ironclaw_hdc` crate: `HdcVector`, `Codebook`, `BundleAccumulator`,
  `DecayingBundleAccumulator`, `encode_document`, `encode_text`, `check_dedup`,
  `top_k_scan`.

None of the 14 concepts in this document are current behavior. All require new
code behind feature flags or env vars with `off` as the default. Concepts 10
and 11 are explicitly deferred (see Build Order Summary). Concept 12 is not an
HDC integration and should be tracked as a separate scheduler improvement.
