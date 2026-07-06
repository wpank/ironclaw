# HDC Innovation Integrations for IronClaw

This document maps the more ambitious HDC memory patterns from the Roko-derived
material into practical IronClaw integrations. It assumes the reader does not
have any other repository open. It is intentionally scoped to IronClaw concepts,
files, tests, and rollout constraints.

For concrete fixture rows, benchmark commands, telemetry shapes, and full
scenario examples, see `tmp/hdc-innovation-examples-and-benchmarks.md`.

The core idea is simple: HDC should not stop at document deduplication. It can
become a typed pattern layer over turns, memory documents, tool outcomes,
heartbeat findings, routines, subagents, and code structure. That gives
IronClaw a way to remember shapes of situations, not just snippets of text.

## Executive Summary

Current HDC work gives IronClaw a local 10,240-bit fingerprint for workspace
documents. That is useful for near-duplicate memory writes and search
diagnostics. The larger opportunity is to use the same algebra for agent
experience:

- "Have we seen this kind of task before?"
- "What worked last time?"
- "What failed last time?"
- "Does this proposed plan resemble a known bad pattern?"
- "Is this heartbeat finding truly new, or a repeat with changed details?"
- "Are two code paths structurally similar enough to require parity tests?"
- "Are subagents repeating each other, or covering complementary niches?"

The most impactful IronClaw integrations are:

1. **Episode pattern memory:** fingerprint completed turns and retrieve
   similar successful or failed episodes before planning.
2. **Anti-knowledge immunity:** store verified failed approaches as suppressive
   memory near the same HDC location as the claim they refute.
3. **Context attention and dedup:** fingerprint candidate prompt snippets,
   keep higher-authority duplicates, and measure saved tokens.
4. **Cross-domain resonance:** deliberately query across domains, such as
   "auth route failure" against "rate-limit route success", to propose
   analogical fixes.
5. **Subagent goal and result dedup:** fingerprint subagent task descriptions
   at spawn time and results at completion to eliminate redundant work within
   a TurnScope. (Section 14 — concrete seams exist in `goal_store.rs` and
   `completion_observer.rs`.)
   Routine/subagent stigmergy coordination field (Track 8): DEFERRED. The
   typed CoordinationField with attention interference calculus is a
   mini-scheduler with no baseline infrastructure. Hold until items 1 and 5
   produce measured wins.
6. **Replay and counterfactual labs:** replay high-utility episodes offline,
   compare variants, and promote only evidence-backed playbooks or warnings.

These are follow-up tracks, not requirements for the current dedup PR. They
should be built as observable, feature-gated, caller-tested slices.

## Current IronClaw Base

IronClaw already has enough primitives to start:

| Area | Current anchor |
|---|---|
| HDC algebra | `crates/ironclaw_hdc::{HdcVector, encode_text, encode_document}` |
| Binary vector operations | bind, bundle, permute, Hamming similarity |
| Document fingerprints | nullable `memory_documents.hdc_fingerprint` metadata |
| Workspace preflight | `Workspace::check_dedup` |
| Memory tool behavior | `src/tools/builtin/memory.rs` |
| Search diagnostics | `SearchConfig.use_hdc`, `SearchResult.hdc_rank`, `hdc_score` |
| Native memory provider | `crates/ironclaw_memory_native` indexing/repository layer |
| Memory context loading | `MemoryPromptContextService` and `ProductionMemoryPromptContextService` |
| Heartbeat novelty | `HeartbeatRunner::with_hdc_config`, observe/suppress config |
| Reborn loop boundary | `crates/ironclaw_agent_loop`, `crates/ironclaw_loop_support` |
| Subagents | `crates/ironclaw_reborn/src/subagent/` (goal_store, completion_observer, tombstone_store, flavors), `builtin.spawn_subagent` capability |
| Subagent HDC seams | `goal_store.rs::put_goal`, `completion_observer.rs::handle_terminal` — no HDC integration exists yet; Section 14 defines the first seams |
| Routines | `src/agent/routine_engine.rs`, routine tests and fixtures; no HDC integration seam exists |

Important existing constraints:

- HDC remains compile-time gated by the `hdc` feature and runtime-default-off.
- HDC data is derived metadata inside existing memory/workspace systems.
- Provider-neutral memory contracts should stay provider-neutral; HDC indexing
  belongs in the concrete memory provider or workspace layer unless the public
  contract needs generic score fields.
- Scope filtering happens before similarity search.
- Behavior changes must be tested through the real caller, not only helper
  functions.
- PostgreSQL and libSQL parity is required for new persistence behavior.
- Heartbeat suppression, search reranking, routing changes, and automatic
  memory promotion must start in observe-only mode.

## What Transfers From Roko

The useful Roko-derived ideas are not implementation dependencies. They are
portable patterns:

| Pattern | What matters for IronClaw |
|---|---|
| Role-filler HDC | Bind typed roles such as `cause`, `effect`, `tool`, `gate`, `outcome`, and `scope` to values, then bundle them into a structured memory vector. |
| Anti-knowledge | Negative memory should live near the claim it refutes, so retrieval naturally surfaces warnings beside tempting but disproven advice. |
| Episode fingerprints | Completed work should produce a durable pattern record with task, plan, tools, gates, outcome, cost, and evidence. |
| Cross-domain resonance | A similarity query that excludes the current domain can find structural analogies across unrelated subsystems. |
| Replay and counterfactuals | Offline jobs can cluster past episodes, compare failures to successes, and propose playbooks or warnings. |
| Replay utility | Rank episodes by surprise, current need, replay spacing, and expected learning value instead of replaying FIFO. |
| Falsifier receipts | Convert repeated gate failures, test failures, approval rejections, and tool errors into evidence against stale beliefs. |
| Anti-correlated review | Query distant or inverse-neighborhood validated memories to stress-test a design, not to drive automatic actions. |
| Stigmergy | Agents can coordinate indirectly by depositing scoped, decaying traces at an HDC "location." DEFERRED: requires multi-session coordination layer that does not yet exist. First do subagent goal dedup (Section 14). |
| Attention interference | Treat active traces as a field where threats suppress opportunities, wisdom stays visible, and per-kind caps prevent saturation. DEFERRED: depends on stigmergy infrastructure. Diagnostics-only until coordination field measured. |
| Specialization | Subagents can broadcast role vectors and receive soft inhibition when too many agents occupy the same niche. DEFERRED: no role-vector broadcast exists in `builtin.spawn_subagent`; cannot add without scope-widening the capability surface. |
| Trust barriers | Imported or broader-scope patterns are discounted and require confirmations before crossing scope boundaries. |
| Pattern metabolism | Patterns should be reinforced by successful reuse, decayed by age/nonuse, and promoted only with evidence. |

## Architecture Contract

HDC should be an integration layer over existing IronClaw systems:

```text
workspace documents
turn outcomes
tool calls
gate results
heartbeat findings
subagent/routine traces
code symbols
        |
        v
typed HDC pattern records
        |
        v
scoped query, scoring, diagnostics, and prompt admission
```

It must not become:

- a separate transcript-backed memory store,
- an unscoped cross-user vector scan,
- a hidden model/router decision engine,
- a substitute for tests, parser facts, or authorization checks,
- a reason to expose raw fingerprints in normal tool output.

## Document Split

| Document | Use it for |
|---|---|
| `tmp/hdc-implementation-plan.md` | Current HDC PR scope: workspace fingerprints, dedup, search diagnostics, heartbeat novelty. |
| `tmp/hdc-roko-cognitive-usecases.md` | Self-contained source map and portable design patterns. |
| `tmp/hdc-ironclaw-innovation-integrations.md` | Advanced IronClaw architecture and integration tracks. |
| `tmp/hdc-innovation-examples-and-benchmarks.md` | Scenario examples, fixture rows, telemetry, benchmark commands, and promotion gates. |

## Use Case Portfolio

| Use case | Net-new capability | First integration |
|---|---|---|
| Episode pattern memory | Agent recalls similar completed work before planning. | Record turn fingerprints after completion; retrieve top similar successes/failures as context diagnostics. |
| Anti-knowledge immunity | Agent remembers disproven approaches and warns before repeating them. | Store verified failures as `anti_knowledge` documents/patterns with shared HDC neighborhood. |
| Gate failure receipts | Repeated failed gates become structured falsifier evidence. | Convert failed tests, denied approvals, and tool errors into anti-pattern candidates. |
| Causal memory | Agent can answer "what tends to cause this failure?" from structured HDC links. | Role-filler encoding over cause/effect/condition/outcome fields. |
| Context dedup | Prompt budget is spent on unique, high-authority context. | Dedup `MemoryPromptContextService` candidates before admission. |
| Cross-domain resonance | Failures in one area can retrieve successful patterns in another. | Diagnostic `memory hdc resonance` command over scoped pattern records. |
| Anti-correlated design review | Distant validated memories expose blind spots and threat-model gaps. | Query inverse/distant neighborhoods into a staging review, not live context. |
| Replay consolidation | Offline job turns repeated episodes into playbook candidates. | Batch completed turns/routines; cluster and promote only after evidence gates. |
| Counterfactual run lab | Completed runs can be replayed with different context, model, or tool ordering. | Promote only variants that pass real gates; failed variants become anti-knowledge. |
| Heartbeat attention | Repeated findings become lower-noise while changed findings stay visible. | Persist heartbeat novelty metadata; keep suppression off until canary-safe. |
| Subagent goal/result dedup | Redundant subagent spawns and near-identical results are deduplicated within a TurnScope. | Fingerprint task descriptions at `put_goal`; fingerprint results at `handle_terminal`. See Section 14. |
| Subagent stigmergy | DEFERRED: Agents leave traces so others avoid duplicate work and find hot spots. | No multi-session coordination layer exists. Build Section 14 first; revisit after measured wins. |
| Attention interference | DEFERRED: Active traces influence priority without hiding safety signals. | Depends on stigmergy infrastructure. Diagnostics-only until coordination field exists. |
| Tool/model routing evidence | Routing learns from pattern outcomes instead of static heuristics. | Observe-only routing annotations keyed by task fingerprint. |
| Code structural memory | Similar implementations and parity gaps are found across backends. | Parser-backed symbol fingerprints, diagnostic only. |

## Integration Track 1: Episode Pattern Memory

### Why It Matters

Workspace memory stores facts. Episode memory stores experience. The difference
is what unlocks useful behavior:

- A new task resembles three previous libSQL/Postgres parity fixes.
- One previous plan failed because it tested only a helper, not the caller.
- A cheaper model handled similar documentation-only edits successfully.
- A certain migration style repeatedly failed on libSQL.

This lets the agent retrieve "what happened when we tried this before" before
spending tokens on a fresh plan.

### Proposed Record

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdcPatternKind {
    Episode,
    Playbook,
    AntiKnowledge,
    CausalLink,
    HeartbeatFinding,
    CoordinationTrace,
    CodeSymbol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpisodeOutcome {
    Succeeded,
    Failed,
    Blocked,
    Interrupted,
}

pub struct EpisodePatternRecord {
    pub id: uuid::Uuid,
    pub tenant_id: String,
    pub user_id: String,
    pub agent_id: Option<String>,
    pub project_id: Option<String>,
    pub thread_id: Option<String>,
    pub kind: HdcPatternKind,
    pub task_summary: String,
    pub plan_summary: Option<String>,
    pub tools_used: Vec<String>,
    pub gates_observed: Vec<String>,
    pub outcome: EpisodeOutcome,
    pub evidence_refs: Vec<String>,
    pub confidence: f32,
    pub fingerprint: ironclaw_hdc::HdcVector,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

The record can initially live as a workspace document with metadata, then move
to a proper DB-backed pattern table if query volume warrants it. If a DB table
is added, it must go through the shared DB trait first and implement both
PostgreSQL and libSQL.

### Encoding Sketch

```rust
use ironclaw_hdc::bundle::BundleAccumulator;
use ironclaw_hdc::codebook::Codebook;
use ironclaw_hdc::HdcVector;

fn role(codebook: &mut Codebook, name: &str) -> HdcVector {
    codebook.get_or_create(&format!("role:{name}"))
}

fn atom(codebook: &mut Codebook, namespace: &str, value: &str) -> HdcVector {
    codebook.get_or_create(&format!("{namespace}:{value}"))
}

fn bind_field(
    codebook: &mut Codebook,
    role_name: &str,
    namespace: &str,
    value: &str,
) -> HdcVector {
    role(codebook, role_name).bind(atom(codebook, namespace, value))
}

fn encode_episode_pattern(record: &EpisodePatternRecord, codebook: &mut Codebook) -> HdcVector {
    let mut acc = BundleAccumulator::new();
    acc.add_weighted(&bind_field(codebook, "kind", "kind", "episode"), 2);
    acc.add_weighted(&bind_field(codebook, "outcome", "outcome", outcome_key(record.outcome)), 3);
    acc.add_weighted(&ironclaw_hdc::encode_text(&record.task_summary, codebook), 5);

    if let Some(plan) = &record.plan_summary {
        acc.add_weighted(&bind_field(codebook, "plan", "text", plan), 2);
    }

    for (idx, tool) in record.tools_used.iter().enumerate() {
        acc.add(&bind_field(codebook, "tool", "tool", tool).permute(idx));
    }
    for (idx, gate) in record.gates_observed.iter().enumerate() {
        acc.add_weighted(&bind_field(codebook, "gate", "gate", gate).permute(idx), 2);
    }

    acc.finalize()
}

fn outcome_key(outcome: EpisodeOutcome) -> &'static str {
    match outcome {
        EpisodeOutcome::Succeeded => "succeeded",
        EpisodeOutcome::Failed => "failed",
        EpisodeOutcome::Blocked => "blocked",
        EpisodeOutcome::Interrupted => "interrupted",
    }
}
```

### IronClaw Seams

| Seam | Role |
|---|---|
| `crates/ironclaw_agent_loop` | Emit episode summary after loop completion. |
| `crates/ironclaw_loop_support` | Include run profile, tool/capability, and gate metadata. |
| `crates/ironclaw_turns` | Carry scope and actor identity into pattern write/retrieval. |
| `crates/ironclaw_memory_native` | Own any Reborn/native derived HDC index and update it after writes. |
| `crates/ironclaw_memory` | Stay provider-neutral unless generic score/result fields are required. |
| `crates/ironclaw_host_runtime/src/memory_context.rs` | Admit selected similar episodes into prompt context with untrusted wrappers. |

### First Observable Behavior

Start with diagnostics:

```text
current task: "add libSQL parity for HDC fingerprint storage"
similar successful episodes:
  0.84 tests/workspace parity fix: DB trait first, then libSQL and Postgres
  0.79 memory tool caller fix: test through MemoryWriteTool, not helper only
similar failed episodes:
  0.81 migration shortcut failed: libSQL syntax diverged from Postgres
```

Do not inject this into default prompts until recorded traces show it helps.

## Integration Track 2: Anti-Knowledge Immunity

### Why It Matters

Agents repeat mistakes when failures are stored only as logs. Anti-knowledge
turns verified failures into retrievable warnings:

- "Do not use Postgres-only migration syntax in shared libSQL migrations."
- "Do not test only `check_dedup`; the side effect is in `MemoryWriteTool`."
- "Do not suppress heartbeat notifications until changed-detail fixtures pass."
- "Do not treat HDC similarity as proof of code equivalence."

The key trick: the anti-knowledge fingerprint should occupy the same HDC
neighborhood as the thing it refutes. A query that retrieves the tempting claim
also retrieves the warning.

### Minimal Shape

```rust
pub struct AntiKnowledgeRecord {
    pub id: uuid::Uuid,
    pub refuted_pattern_id: Option<uuid::Uuid>,
    pub warning: String,
    pub evidence: String,
    pub evidence_refs: Vec<String>,
    pub scope_tags: Vec<String>,
    pub confidence: f32,
    pub fingerprint: ironclaw_hdc::HdcVector,
}

fn encode_anti_knowledge(
    refuted_fingerprint: ironclaw_hdc::HdcVector,
    warning: &str,
    codebook: &mut ironclaw_hdc::codebook::Codebook,
) -> ironclaw_hdc::HdcVector {
    let mut acc = ironclaw_hdc::bundle::BundleAccumulator::new();
    acc.add_weighted(&refuted_fingerprint, 5);
    acc.add_weighted(&ironclaw_hdc::encode_text(warning, codebook), 2);
    acc.add_weighted(
        &codebook
            .get_or_create("role:anti_knowledge")
            .bind(codebook.get_or_create("kind:warning")),
        2,
    );
    acc.finalize()
}
```

### Admission Rule

```text
candidate advice
  -> compute fingerprint
  -> query scoped anti-knowledge
  -> if similarity >= warning_threshold:
       include warning or block promotion
     else:
       proceed through normal memory admission
```

The first implementation should warn, not block. Blocking requires very high
precision and a clear `force` or approval escape hatch.

### Gate Failure Receipts

Anti-knowledge should not rely only on a model-written summary. IronClaw already
has stronger evidence sources: failed tests, approval denials, rejected tool
calls, review findings, failed migrations, and routine failures. Convert those
events into falsifier receipts first, then let repeated receipts promote an
anti-pattern candidate.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FalsifierSource {
    TestFailure,
    ApprovalDenied,
    ToolError,
    ReviewFinding,
    MigrationFailure,
    RoutineFailure,
}

pub struct FalsifierReceipt {
    pub source: FalsifierSource,
    pub refutes_summary: String,
    pub evidence_ref: String,
    pub run_id: Option<String>,
    pub confidence: f32,
    pub fingerprint: ironclaw_hdc::HdcVector,
}
```

Promotion rule:

```text
if same-pattern falsifier_receipts >= 2
and no newer successful episode refutes the warning
and source scope matches the candidate scope:
  stage anti-knowledge candidate
else:
  keep as diagnostic receipt only
```

This avoids turning one flaky run into durable negative memory while still
letting repeated contradictions demote stale positive advice.

### Tests

Caller-level tests must drive the real path:

- memory write that would promote a redundant positive lesson,
- memory write that resembles anti-knowledge,
- repeated gate failure that stages an anti-pattern candidate,
- newer successful evidence that prevents stale anti-knowledge from blocking,
- agent-loop context load that includes a warning,
- tool execution plan where warning is diagnostic only.

Helper-only tests for similarity thresholds are useful but insufficient.

## Integration Track 3: Role-Filler Causal Memory

### Why It Matters

Plain text memory answers "what did we write?" Causal memory answers "what
tends to cause what?" That is useful for debugging and planning:

- Cause: "missing DB trait method"; effect: "backend parity compile failure".
- Cause: "helper-only test"; effect: "caller regression escapes".
- Cause: "heartbeat repeated threshold too low"; effect: "novel finding hidden".
- Cause: "cross-user HDC scan"; effect: "scope leak risk".

### Encoding

Use role-filler binding:

```rust
pub struct CausalPatternInput<'a> {
    pub cause: &'a str,
    pub effect: &'a str,
    pub condition: Option<&'a str>,
    pub domain: &'a str,
    pub strength_bucket: &'a str,
}

pub fn encode_causal_pattern(
    input: CausalPatternInput<'_>,
    codebook: &mut ironclaw_hdc::codebook::Codebook,
) -> ironclaw_hdc::HdcVector {
    let mut acc = ironclaw_hdc::bundle::BundleAccumulator::new();
    acc.add_weighted(&bind_field(codebook, "cause", "text", input.cause), 4);
    acc.add_weighted(&bind_field(codebook, "effect", "text", input.effect), 4);
    acc.add_weighted(&bind_field(codebook, "domain", "domain", input.domain), 2);
    acc.add(&bind_field(
        codebook,
        "strength",
        "bucket",
        input.strength_bucket,
    ));
    if let Some(condition) = input.condition {
        acc.add_weighted(&bind_field(codebook, "condition", "text", condition), 2);
    }
    acc.finalize()
}
```

Later, unbinding can support targeted diagnostics such as "show likely causes
for this effect." The first version does not need full symbolic decode; it only
needs typed records with HDC similarity.

## Integration Track 4: Context Attention and Dedup

### Why It Matters

IronClaw already has a memory context service that fetches snippets and admits
them through host safety wrappers. HDC can improve this without changing the
memory backend:

- detect near-duplicate snippets from daily logs, runbooks, and memories,
- keep the higher-authority version,
- preserve warnings even when they duplicate positive claims,
- measure prompt token savings and answer quality.

### Candidate Ranking Sketch

```rust
pub enum ContextAuthority {
    UserPinned,
    SystemInstruction,
    VerifiedRunbook,
    AntiKnowledge,
    DurableMemory,
    DailyLog,
    RecentTurn,
}

pub struct HdcContextCandidate {
    pub snippet_ref: String,
    pub authority: ContextAuthority,
    pub baseline_score: f32,
    pub token_count: usize,
    pub model_content: String,
    pub fingerprint: ironclaw_hdc::HdcVector,
}

pub struct HdcContextDecision {
    pub kept: Vec<HdcContextCandidate>,
    pub dropped: Vec<DroppedContextCandidate>,
    pub saved_tokens: usize,
}
```

Rules:

- never drop system instructions,
- never drop anti-knowledge solely because it is similar to positive memory,
- prefer user-pinned and verified runbook content over transient logs,
- record every dropped snippet with `kept_ref`, similarity, and saved tokens,
- shadow mode first: compute decisions but return the original snippets.

### Integration Seam

The clean seam is after `MemoryService::retrieve_context` returns snippets and
before `ProductionMemoryPromptContextService` admits them into
`LoopContextSnippet`.

This keeps provider-specific memory retrieval unchanged and preserves the host
adapter as the place where prompt admission is enforced.

## Integration Track 5: Cross-Domain Resonance

### Why It Matters

Normal retrieval searches within the current topic. Cross-domain resonance does
the opposite: it excludes the current domain to find structural analogies.

Examples:

- Gateway auth failure resembles an earlier rate-limit route fixture fix.
- libSQL/Postgres parity issue resembles provider config parity issue.
- Repeated heartbeat suppression risk resembles notification dedup risk.
- Subagent duplicate work resembles context duplicate chunk selection.

### Diagnostic API

```text
ironclaw memory hdc resonance \
  --query "webhook auth route rejects valid callback" \
  --exclude-domain "gateway_auth" \
  --include-domains "workspace,db,routines,heartbeat" \
  --min-similarity 0.526 \
  --limit 10 \
  --explain
```

The threshold `0.526` is useful as a statistical starting point for independent
10,240-bit vectors, but it is not a quality guarantee. IronClaw should tune
domain-specific thresholds with fixtures and show raw scores.

### Result Shape

```json
{
  "query_domain": "gateway_auth",
  "matches": [
    {
      "domain": "rate_limit_routes",
      "similarity": 0.548,
      "pattern_ref": "episode:route-rate-limit-2026-06-18",
      "summary": "Caller-level route fixture caught missing header propagation.",
      "suggested_transfer": "Add a route-level fixture, not only helper tests.",
      "risk": "medium"
    }
  ]
}
```

This must remain advisory. A cross-domain analogy can suggest a test or plan,
but it cannot claim correctness.

### Anti-Correlated Review Mode

Cross-domain resonance looks for structurally similar patterns. A separate
review mode should look for validated but distant patterns, because those are
useful for threat modeling and architecture review:

```text
ironclaw memory hdc design-review \
  --query "enable HDC-based memory admission warnings" \
  --mode anti-correlated \
  --include-kinds "anti_knowledge,playbook,incident" \
  --limit 8
```

Expected output is a review checklist, not prompt injection into the live agent
loop:

```text
Potential blind spots:
  - memory prompt safety wrappers failed when provider returned pre-shaped text
  - scope leak appeared when helper used user_id but not agent_id
  - heartbeat suppression hid changed metric in fixture X
```

Implementation notes:

- Compute distant candidates by low positive similarity plus high evidence,
  not by blindly inverting bits and trusting the result.
- Keep outputs in staging or diagnostics until a caller-level test or human
  review confirms usefulness.
- Never use anti-correlated output as automatic negative memory. It is a design
  review prompt, not a fact.

## Integration Track 6: Replay Consolidation and Counterfactuals

### Why It Matters

Some learning should happen outside the live agent loop. An offline
consolidation job can:

- cluster completed episodes,
- identify repeated failures,
- identify repeated successes,
- propose playbook candidates,
- reject redundant lessons,
- generate counterfactual repair ideas from similar success clusters.

### Replay Utility Scheduler

Replay should be prioritized by expected learning value, not by recency alone.
A practical scheduler can score each episode with four terms:

```text
utility = gain * need * replayability * spacing
```

| Term | Meaning | Example source |
|---|---|---|
| `gain` | How surprising or costly the episode was. | failed gate, unexpected success, high token spend |
| `need` | How similar it is to current or recent work. | HDC similarity to recent episode centroid |
| `replayability` | Whether the run has enough recorded inputs to replay safely. | fixtures, deterministic tool mocks, no live secrets |
| `spacing` | Penalty for replaying the same episode too recently. | `replay_count`, `last_replayed_at` |

```rust
pub struct ReplayMetadata {
    pub episode_id: uuid::Uuid,
    pub replay_count: u32,
    pub last_replayed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expected_assertions: Vec<String>,
}

pub fn replay_utility(
    gain: f32,
    need: f32,
    replayability: f32,
    replay_count: u32,
    hours_since_replay: Option<f32>,
) -> f32 {
    let spacing = match hours_since_replay {
        Some(hours) => 1.0 / (1.0 + replay_count as f32 * 0.5 / hours.max(0.25)),
        None => 1.0,
    };
    (gain * need * replayability * spacing).clamp(0.0, 1.0)
}
```

Store `replay_count` and `last_replayed_at` only after the replay job actually
runs. This prevents scheduling attempts from poisoning the spacing term.

### Counterfactual Run Lab

A counterfactual lab replays a completed episode with one controlled variable
changed:

- model tier,
- prompt/context budget,
- context dedup on/off,
- tool ordering,
- memory snippets included/excluded,
- approval path or gate threshold.

The lab must use recorded fixtures or hermetic mocks. It must not replay live
secrets, live outbound requests, or user-private raw transcripts.

Promotion rule:

```text
counterfactual variant passes real gates
  -> stage playbook candidate with evidence
counterfactual variant fails reproducibly
  -> stage anti-knowledge/falsifier receipt
counterfactual variant is inconclusive
  -> keep diagnostic only
```

### Minimal Offline Flow

```text
load completed episodes in scope
  -> cluster by HDC similarity and outcome
  -> summarize each cluster
  -> compare failed clusters against successful clusters
  -> emit draft playbook or anti-knowledge candidates
  -> require evidence gate before durable memory write
```

### K-Medoids Is Enough First

For 10K or fewer episodes per user/project, a simple k-medoids or greedy
medoid clustering pass is adequate and easier to audit than a complex ANN
stack. Run it offline, not in the hot request path.

### Promotion Gate

Promote a playbook candidate only if:

- at least `N` episodes support it,
- evidence refs include passing tests, user confirmation, or successful runs,
- HDC redundancy against existing playbooks is below a tuned threshold,
- anti-knowledge does not refute it,
- the candidate stays within the same user/project scope unless explicitly
  published through a broader gate.

## Integration Track 7: Heartbeat Attention

### Why It Matters

The current heartbeat HDC path is in-memory novelty classification. The bigger
use is attention management:

- repeated unchanged findings can be marked as repeat,
- repeated findings with changed metrics stay novel,
- severe or rare findings can raise urgency,
- long-stale repeated findings can become novel again after decay,
- cadence can be proposed from novelty/severity without immediately changing
  runtime scheduling.

### Near-Term Improvements

1. Persist heartbeat novelty metadata per user/checklist.
2. Include novelty classification in notification metadata.
3. Add trace/log assertions around runner-loop behavior.
4. Keep `HEARTBEAT_HDC_SUPPRESS_REPEATED=false` by default.
5. Add an observe-only proposed interval:

```rust
pub fn proposed_heartbeat_interval(
    base_secs: u64,
    severity: f32,
    novelty: f32,
    repeated_count: u32,
) -> u64 {
    let urgent = severity >= 0.8 || novelty >= 0.8;
    if urgent {
        return (base_secs / 2).max(300);
    }
    if repeated_count >= 5 && novelty <= 0.2 {
        return (base_secs * 2).min(86_400);
    }
    base_secs
}
```

The function should first be recorded as telemetry only. It should not change
cadence until false-suppression fixtures and canaries are clean.

## Integration Track 8: Subagent and Routine Stigmergy

DEFERRED: No multi-session coordination layer exists in IronClaw. This track
requires infrastructure that is not yet built.

### Why It Is Deferred

The stigmergy model proposes a typed coordination field (`CoordinationField`)
with six pheromone-like trace kinds, half-life decay, scope promotion
(Local→Project→Global), and an attention interference calculus. In the Roko
context this makes sense because concurrent agents share a workspace. In the
current IronClaw Reborn architecture:

- Subagents are scoped to a TurnScope within a single run; there is no
  multi-session shared workspace they write coordination traces to.
- The routine engine fires routines independently; routines do not
  communicate with each other or with concurrent subagents through any
  shared signal layer.
- The `CoordinationField` with interference functions (`sensed_opportunity`,
  `sensed_wisdom`) is a scheduling primitive — the doc explicitly notes it
  is "not a scheduler yet." Without measured duplicate-work reduction from
  Section 14 (goal/result dedup), the interference model has no baseline.
- Adding a coordination trace layer before goal dedup is measured would
  mean building a coordination scheduler for a duplication problem that may
  be smaller than expected.

### What to Build First Instead

Section 14 (Subagent Goal and Result Dedup) targets the concrete near-term
win: preventing redundant spawns and filtering near-duplicate results within
a TurnScope. It uses real seams (`goal_store.rs::put_goal`,
`completion_observer.rs::handle_terminal`) and does not require a new
coordination layer.

### Minimal Diagnostic Kept

The diagnostic display concept is worth preserving for future use. When
stigmergy is revisited, the first observable should be a read-only
diagnostic — not a live field affecting spawn decisions:

```text
# Example future diagnostic output (not implemented)
field near "memory HDC rollout":
  threat      0.76  "heartbeat suppression fixtures incomplete"
  wisdom      0.68  "test through MemoryWriteTool"
  opportunity 0.32  "search diagnostics can reuse HDC rank fields"
```

### Specialization Extension

DEFERRED: Subagent role vector broadcasting requires changes to the
`builtin.spawn_subagent` capability surface. This capability surface is
attenuation-controlled (coder flavor explicitly excludes `spawn_subagent`).
Adding a role vector broadcast to spawn logic risks widening capability
scopes or adding implicit coordination side-effects to the spawn contract.
Do not implement until spawn dedup (Section 14) is stable and measured.

## Integration Track 9: Tool and Model Routing Evidence

### Why It Matters

Routing decisions should learn from similar prior episodes:

- Which tools were used in similar successful turns?
- Which model tier was enough?
- Which gates failed?
- Which capability caused approval friction?

### Safe First Step

Add observe-only route annotations:

```json
{
  "task_fingerprint_similarity": 0.82,
  "similar_episode_count": 6,
  "historical_success_rate": 0.83,
  "tools_used_in_successes": ["memory_search", "apply_patch", "cargo_test"],
  "model_tier_suggestion": "standard",
  "action": "observe_only"
}
```

Do not alter model/tool routing until:

- routing decisions are replayed on recorded fixtures,
- cost savings are measured,
- failure/regression rate is bounded,
- approval/auth flows have caller-level tests.

## Integration Track 10: Code Structural Memory

### Why It Matters

HDC can fingerprint code structure in ways that text search misses:

- "find the Postgres implementation matching this libSQL method",
- "find handlers with the same auth shape",
- "find DB migration parity gaps",
- "find tests that exercise analogous caller paths."

### Contract

HDC suggests candidates. Parser facts and tests decide truth.

### Symbol Encoding Sketch

```rust
pub struct SymbolPatternInput<'a> {
    pub language: &'a str,
    pub symbol_kind: &'a str,
    pub name: &'a str,
    pub module_path: &'a str,
    pub parameters: &'a [&'a str],
    pub return_type: Option<&'a str>,
    pub callees: &'a [&'a str],
}

pub fn encode_symbol_pattern(
    input: SymbolPatternInput<'_>,
    codebook: &mut ironclaw_hdc::codebook::Codebook,
) -> ironclaw_hdc::HdcVector {
    let mut acc = ironclaw_hdc::bundle::BundleAccumulator::new();
    acc.add_weighted(&bind_field(codebook, "language", "lang", input.language), 2);
    acc.add_weighted(&bind_field(codebook, "kind", "symbol_kind", input.symbol_kind), 2);
    acc.add_weighted(&bind_field(codebook, "name", "symbol_name", input.name), 2);
    acc.add(&bind_field(codebook, "module", "path", input.module_path));

    for (idx, param) in input.parameters.iter().enumerate() {
        acc.add(&bind_field(codebook, "param", "type", param).permute(idx));
    }
    if let Some(return_type) = input.return_type {
        acc.add(&bind_field(codebook, "return", "type", return_type));
    }
    for (idx, callee) in input.callees.iter().enumerate() {
        acc.add(&bind_field(codebook, "callee", "symbol_name", callee).permute(idx));
    }

    acc.finalize()
}
```

## Persistence Options

### Option A: Workspace Documents First

Use generated documents under scoped memory paths such as:

```text
patterns/episodes/{date}/{run_id}.md
patterns/anti-knowledge/{slug}.md
patterns/playbooks/{slug}.md
```

Pros:

- minimal schema change,
- reuses existing workspace semantics,
- easy to inspect and delete,
- HDC fingerprint column already exists for documents.

Cons:

- query fields are metadata/text until formalized,
- clustering large corpora may be inefficient,
- no clean first-class pattern API.

### Option B: Pattern Table

Add `memory_hdc_patterns`:

```sql
CREATE TABLE memory_hdc_patterns (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  agent_id TEXT,
  project_id TEXT,
  thread_id TEXT,
  kind TEXT NOT NULL,
  domain TEXT NOT NULL,
  summary TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  evidence_json TEXT NOT NULL,
  confidence REAL NOT NULL,
  hdc_fingerprint BLOB NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

Required indexes:

```sql
CREATE INDEX idx_hdc_patterns_scope
  ON memory_hdc_patterns (tenant_id, user_id, agent_id, project_id, kind);

CREATE INDEX idx_hdc_patterns_created
  ON memory_hdc_patterns (tenant_id, user_id, created_at);
```

Pros:

- clean query API,
- easier clustering and diagnostics,
- avoids overloading document paths.

Cons:

- requires DB trait and backend parity,
- needs migration/rollback review,
- needs API design before product exposure.

Recommended path: start with workspace documents for experiments; move to a
table only when episode/pattern query volume and schema stability justify it.

### Option C: Native Memory Derived Index

For Reborn/native memory, add HDC inside `crates/ironclaw_memory_native`, not in
the provider-neutral `crates/ironclaw_memory` contract.

Best seam:

```text
NativeMemoryService::write/profile_set
  -> native indexer chunks document
  -> optional HDC encoder computes chunk/document/pattern fingerprints
  -> native repository stores derived index state
  -> search/retrieve_context can use HDC only when enabled
  -> host runtime still admits sanitized snippets
```

Rules:

- keep `ironclaw_hdc` as the math crate,
- add an optional native-memory `hdc` feature if needed and let the root
  `hdc` feature forward to it,
- do not make provider-neutral memory depend on HDC,
- expose only bounded scores or sanitized snippets across provider-neutral
  boundaries,
- never pass raw HDC vectors into prompt materialization.

### Scoped Sharing and Trust Barriers

If patterns are ever shared beyond the originating local scope, apply explicit
trust barriers:

| Boundary | Default action | Promotion requirement |
|---|---|---|
| Thread to project | allow diagnostic lookup | same user/project scope |
| Project to user-wide | discount confidence | user approval or repeated success |
| User to group/team | do not auto-publish | confirmations from at least two members |
| Subnet/private scope to broader scope | deny by default | publish gate plus optional human approval |
| Imported external pattern | stage only | provenance trust discount and local validation |

```rust
pub enum PatternProvenance {
    SelfGenerated,
    SameProject,
    SameUserDifferentProject,
    TeamShared,
    ImportedExternal,
}

pub fn provenance_trust_multiplier(provenance: PatternProvenance) -> f32 {
    match provenance {
        PatternProvenance::SelfGenerated => 1.0,
        PatternProvenance::SameProject => 0.9,
        PatternProvenance::SameUserDifferentProject => 0.75,
        PatternProvenance::TeamShared => 0.65,
        PatternProvenance::ImportedExternal => 0.45,
    }
}
```

Imported patterns can suggest tests or review questions. They should not become
durable local knowledge until local evidence confirms them.

## Implementation Roadmap

### Phase 0: Keep Current HDC Dedup Stable

- Finish existing HDC dedup/search/heartbeat tests.
- Normalize live-write tags versus backfill tags.
- Keep all active behavior default-off.
- Document remaining PostgreSQL coverage gaps.

### Phase 1: Episode Fingerprints in Shadow Mode

- Define an internal `EpisodePatternInput`.
- Encode task summary, outcome, tool names, gate names, and scope tags.
- Emit a scoped diagnostic record after turn completion.
- For Reborn/native memory, keep derived indexing in `ironclaw_memory_native`
  or a host-runtime adapter, not in the provider-neutral contract.
- Add a CLI/debug command to show similar past episodes.
- Do not inject into prompts yet.

Tests:

- deterministic fingerprint for same episode input,
- scope isolation for episode query,
- no raw fingerprint in normal user output,
- caller-level loop completion test records a pattern when enabled.

### Phase 2: Anti-Knowledge Warnings

- Represent verified failed approaches as warning records.
- Query anti-knowledge during memory admission or context assembly.
- Surface warning diagnostics without blocking actions.
- Add evidence refs and confidence.

Tests:

- proposed positive memory similar to warning yields warning,
- unrelated memory is not warned,
- anti-knowledge is not dropped by context dedup,
- warning stays scoped to tenant/user/project.

### Phase 3: Context Attention Dedup

- Fingerprint memory context snippets.
- Compute keep/drop decisions in shadow mode.
- Record saved tokens and dropped-citation risk.
- Enable live context dedup only for bounded profiles after QA traces pass.

Tests:

- daily log duplicate dropped in favor of verified runbook,
- anti-knowledge retained,
- host wrapper invariants preserved,
- answer quality does not regress on recorded fixtures.

### Phase 4: Cross-Domain Resonance Diagnostics

- Add a bounded diagnostic command.
- Require domain filters and scope filters.
- Show raw similarity, source domain, target domain, evidence, and transfer
  risk.
- Keep suggestions advisory.

Tests:

- same-domain exclusion works,
- unscoped/cross-user query is impossible,
- low-similarity random matches are filtered,
- route/auth example returns caller-level test suggestion only as advice.

### Phase 5: Offline Replay Consolidation

- Batch completed episodes.
- Score replay candidates by gain, need, replayability, and spacing.
- Cluster by HDC similarity.
- Run counterfactual variants only against hermetic fixtures or mocks.
- Generate playbook/anti-knowledge candidates.
- Require evidence gate before durable write.

Tests:

- repeated successful DB parity episodes produce one playbook candidate,
- repeated failures produce anti-knowledge candidate,
- replay utility respects spacing and does not rerun the same episode forever,
- counterfactual variants do not use live secrets, live outbound requests, or
  raw private transcripts,
- redundant playbook is not rewritten,
- live loop behavior is unchanged by the offline job unless promotion is
  approved.

### Phase 6: Subagent/Routine Coordination Traces

DEFERRED: See Integration Track 8 for the full rationale.

Prerequisite: Phase 5 (offline replay) must produce measured duplicate-work
data. Section 14 (subagent goal/result dedup) must be stable and producing
metrics. Only after those two provide evidence of duplicate-work magnitude
should coordination traces be designed.

When this phase is eventually scheduled, the gate tests remain as stated:

- child subagent cannot see traces outside its scope,
- capability scopes are unaffected,
- no trusted trigger path is introduced.

## Benchmarks and Quantification

### Core Operation Benchmarks

| Benchmark | Purpose | Gate |
|---|---|---|
| Encode episode pattern | Hot-path overhead after turn completion | p95 under 1 ms for normal records |
| Query 10K episode fingerprints | Interactive diagnostics | p95 under 5 ms flat scan or add cache/index |
| Query 100K episode fingerprints | Scaling decision | Report only; decide whether index is needed |
| Context dedup 50 snippets | Prompt assembly overhead | p95 under 2 ms |
| Anti-knowledge check 1K warnings | Admission overhead | p95 under 2 ms |
| Offline clustering 10K episodes | Replay feasibility | batch job under operator-configured window |

### Quality Metrics

| Metric | How to measure |
|---|---|
| Similar episode precision@5 | Human/fixture labels for retrieved prior episodes |
| Similar episode usefulness | Recorded trace answer improves or no regression |
| Anti-knowledge false positive rate | Unique valid plans incorrectly warned |
| Anti-knowledge false negative rate | Known bad plans not warned |
| Context token savings | Tokens removed by live/shadow dedup |
| Dropped useful context rate | Dropped snippet later required/cited in expected answer |
| Cross-domain transfer precision | Suggested analogy leads to useful test/plan |
| Heartbeat false suppression | Novel or materially changed finding hidden |
| Subagent duplicate-work reduction | Fewer agents inspect same files without coverage loss |
| Routing cost delta | Cost saved at equal or better recorded outcome |

### Fixture Families

```text
tests/fixtures/hdc_memory/advanced/
  episodes/
    db_parity_successes.jsonl
    gateway_auth_failures.jsonl
    helper_only_test_regression.jsonl
  anti_knowledge/
    postgres_only_migration_refutation.jsonl
    hdc_similarity_not_code_equivalence.jsonl
  context_attention/
    runbook_vs_daily_log.jsonl
    anti_knowledge_must_survive.jsonl
  resonance/
    auth_route_to_rate_limit_route.jsonl
    db_parity_to_provider_config_parity.jsonl
  heartbeat/
    repeated_with_changed_metric.jsonl
    stale_repeat_becomes_novel.jsonl
  subagent_dedup/   # Section 14 — goal dedup and result dedup fixtures
    goal_dedup.jsonl
    result_dedup.jsonl
  coordination/     # DEFERRED (Track 8) — do not write harness until undeferred
    duplicate_subagent_progress.jsonl       # placeholder only
    uncovered_testing_niche.jsonl           # placeholder only
```

Each fixture should include:

- input records,
- scope,
- expected candidates,
- expected decisions,
- raw similarity ranges,
- rationale,
- allowed false positives, if any.

## Caller-Level Test Matrix

| Behavior | Helper tests | Caller tests |
|---|---|---|
| Episode encoding | deterministic vector, threshold behavior | agent-loop completion records pattern only when enabled |
| Episode retrieval | top-k sorting, scope filter | memory context or diagnostic command returns scoped episodes |
| Anti-knowledge warning | similarity and warning threshold | memory write/admission path surfaces warning |
| Context dedup | keep/drop ordering | `ProductionMemoryPromptContextService` preserves wrappers and budget |
| Heartbeat novelty | accumulator classification | `HeartbeatRunner::run` or runner-equivalent path records/suppresses correctly |
| Resonance | domain exclusion, score ordering | CLI/tool command enforces scope and returns advisory output |
| Subagent goal dedup | fingerprint encoding, in-flight scan | `SubagentGoalStore::put_goal` returns ExistingRunReturned for near-duplicate goal within TurnScope |
| Subagent result dedup | fingerprint encoding, sibling scan | `SubagentCompletionObserver::handle_terminal` marks SimilarToDelivered; result row retained in DB |
| Subagent coord. traces | DEFERRED — trace encoding and decay | DEFERRED — spawn/capability path cannot cross scope or alter permissions |
| Code symbols | parser-backed fingerprint | DB parity/caller tests confirm HDC candidates are not truth |

Concrete caller tests to add when those tracks are implemented:

- `MemoryWriteTool` with `--features hdc`: write similar content and assert
  warn/block/force behavior through the tool.
- Public memory search/tool/service after writes: assert HDC rank/fusion appears
  only when enabled.
- `NativeMemoryService::write` then `search`/`retrieve_context`: prove the
  native derived HDC index updates and prompt snippets remain sanitized.
- `ThreadBackedLoopContextPort` or `HostManagedLoopPromptPort`: assert memory
  snippets are loaded, materialized, and bounded.
- `HeartbeatRunner` notification path: repeated `NeedsAttention` is classified
  through the runner path, not only helper classification.
- `SubagentGoalStore::put_goal` (Section 14): near-duplicate goal within a TurnScope
  returns ExistingRunReturned; capability scopes of the original run are unchanged.
- `SubagentCompletionObserver::handle_terminal` (Section 14): near-duplicate result
  is tombstoned as SimilarToDelivered; result row is retained in DB (not deleted).
- NOTE: Routine firing and `builtin.spawn_subagent` coordination traces (Track 8)
  are deferred. No caller test for the coordination field until the dedup wins
  from Section 14 are measured and the coordination layer is undeferred.

## Security and Privacy Rules

- Always derive scope from host/workspace/turn context before HDC lookup.
- Never run HDC scans across users, tenants, or projects by default.
- Do not log raw fingerprints unless logs are explicitly diagnostic and scoped.
- Do not expose raw fingerprints in normal memory tool output.
- Treat HDC matches as untrusted suggestions.
- Preserve prompt injection wrappers for memory context snippets.
- Do not use HDC to mint trusted trigger requests or bypass product workflow
  ingress rules.
- Do not let subagent coordination traces widen tool capability scopes.
- Do not use cross-domain resonance to move private subnet or project-local
  knowledge into broader scopes without a publish gate.

## Prioritized Recommendations

The best next three advanced tracks are:

1. **Episode pattern memory in shadow mode.**
   This is the foundation. Without episode fingerprints, anti-knowledge,
   playbooks, routing evidence, and cross-domain transfer have little shared
   substrate.

2. **Anti-knowledge warnings.**
   This has high practical value and low automation risk if it starts as a
   warning. It directly prevents repeated mistakes.

3. **Context attention dedup.**
   This improves prompt quality and cost while staying inside an existing host
   admission seam. It can be measured cleanly with token savings and trace
   regression tests.

Hold back on automatic routing, live heartbeat suppression, and subagent
coordination trace federation (Track 8) until the first three tracks and
Section 14 (subagent goal/result dedup) produce measured wins.

**Implementation Status annotations:**
- Track 1 (Episode Pattern Memory): not started, Phase 1 work
- Track 2 (Anti-Knowledge Immunity): not started, Phase 2 work
- Track 3 (Role-Filler Causal Memory): not started
- Track 4 (Context Attention Dedup): not started, Phase 3 work
- Track 5 (Cross-Domain Resonance): not started, Phase 4 work
- Track 6 (Replay Consolidation): not started, Phase 5 work
- Track 7 (Heartbeat Attention): in-memory novelty exists; persistence and
  proposed-interval function not started
- Track 8 (Subagent/Routine Stigmergy): DEFERRED — no coordination layer exists
- Track 9 (Tool/Model Routing Evidence): not started, observe-only annotation only
- Track 10 (Code Structural Memory): not started, diagnostic only
- Section 12 (Native Memory HDC): not started
- Section 13 (Context Admission Novelty Gate): not started
- Section 14 (Subagent Goal/Result Dedup): not started; concrete seams identified
- Section 15 (K-Medoids Clustering): not started
- Section 16 (Fixture Corpus Harness): not started; priority-1 quality gate

## Example End-to-End Scenario

User asks IronClaw to add a new database-backed memory feature.

Current behavior:

1. Search memory by text.
2. Plan from available snippets.
3. Implement and test.

With advanced HDC integrations:

1. Encode the task into an episode query fingerprint.
2. Retrieve similar successful DB parity episodes.
3. Retrieve anti-knowledge warning: "helper-only tests missed caller behavior."
4. Dedup memory snippets, keeping the DB trait runbook over repeated daily logs.
5. Suggest caller-level tests and both backend implementations.
6. After completion, record the episode outcome.
7. Offline consolidation later sees this as another DB parity success and
   strengthens or updates a playbook candidate.

The result is not just cleaner memory. It is experience-aware behavior: the
agent changes its plan because it recognizes the shape of the work.

## 12. Native Memory (Reborn Path) HDC Integration

### Why It Matters

The Reborn/native memory path in `crates/ironclaw_memory_native/` is the
primary memory backend for production agent runs. It has a chunking indexer,
a service layer, and a repository backend — but none of these three seams
has any HDC integration. The existing `src/tools/builtin/memory.rs`
dedup preflight runs only in the workspace tool path and does not reach
native memory writes. All three seams need targeted integration.

### Three Integration Points

**Point A — Chunk-time fingerprinting in the indexer**

`ChunkingMemoryDocumentIndexer::reindex_document_with_audit_context()`
already iterates document chunks and updates the native index. After
chunking and before returning, also compute an HDC fingerprint for the
document as a whole and persist it:

```rust
// crates/ironclaw_memory_native/src/indexer.rs
#[cfg(feature = "hdc")]
if config.hdc_native_index_enabled {
    let fingerprint = ironclaw_hdc::encode_document(
        &doc.content,
        &doc.path,
        &doc.tags,
        codebook,
    );
    repo.upsert_document_fingerprint(&doc.path, &fingerprint).await?;
}
```

This requires a new method on the repository trait:

```rust
// crates/ironclaw_memory_native/src/schema.rs or repository trait
async fn upsert_document_fingerprint(
    &self,
    path: &MemoryPath,
    fingerprint: &ironclaw_hdc::HdcVector,
) -> Result<(), NativeMemoryError>;
```

**Point B — Inter-snippet dedup in retrieve_context**

`NativeMemoryService::retrieve_context()` calls `backend.search()` then
assembles snippets via `collect_context_snippets()`. Between those two
steps, run an HDC inter-snippet dedup pass. Snippets that are near-
duplicate of an already-admitted snippet (measured by Hamming similarity)
are dropped in favor of the higher-scored original. This prevents ten
paraphrased entries about the same fact from consuming context budget:

```rust
// crates/ironclaw_memory_native/src/service.rs
#[cfg(feature = "hdc")]
let search_results = if config.hdc_native_index_enabled {
    ironclaw_hdc::dedup::dedup_candidates(search_results, |r| {
        r.hdc_fingerprint.as_ref()
    }, config.hdc_similarity_threshold)
} else {
    search_results
};
```

Dedup runs in descending score order: the highest-scored snippet is kept
and any later snippet within threshold is dropped. The drop list is
included in debug-level telemetry; it never surfaces in the returned
context snippets or in prompt materialization.

**Point C — Write-time dedup preflight in the repository backend**

`RepositoryMemoryBackend::write_document_with_backend_options()` is the
write path for native memory. Before completing the write, check HDC
similarity against existing documents in the same scope. This is
analogous to the workspace tool preflight already in
`src/tools/builtin/memory.rs`:

```rust
// crates/ironclaw_memory_native/src/backend.rs
#[cfg(feature = "hdc")]
if config.hdc_native_index_enabled {
    let candidate_fp = ironclaw_hdc::encode_document(
        &doc.content, &doc.path, &doc.tags, codebook,
    );
    if let Some(match_) = repo
        .find_similar_document(&candidate_fp, config.hdc_dedup_threshold)
        .await?
    {
        return Err(NativeMemoryError::NearDuplicate {
            existing_path: match_.path,
            similarity: match_.similarity,
        });
    }
}
```

### Config / Feature Gate

```bash
IRONCLAW_HDC_NATIVE_INDEX=true   # enables all three points above
```

Compile gate: `hdc` feature on `crates/ironclaw_memory_native`. The root
`Cargo.toml` `hdc` feature should forward to `ironclaw_memory_native/hdc`.
All three integration points are dead code unless both the compile feature
and the runtime env var are active. Default: off.

### Fixture Shape

Fixture file: `tests/fixtures/hdc_memory/native_memory/write_dedup.jsonl`

Each line is a JSON object:

```json
{ "path": "notes/db-parity.md", "content": "...", "tags": ["db", "libsql"], "action": "write" }
{ "path": "notes/db-parity-v2.md", "content": "... (paraphrase)", "tags": ["db"], "action": "write_expect_dedup" }
{ "path": "notes/unrelated.md", "content": "...", "tags": ["frontend"], "action": "write_expect_unique" }
```

Manifest entry:

```json
"native_memory_write_dedup": {
  "file": "native_memory/write_dedup.jsonl",
  "expected_decision": "duplicate",
  "expected_similarity_range": { "min": 0.75, "max": 0.95 },
  "rationale": "Paraphrase of a db-parity note should trigger write-time dedup preflight."
}
```

### Caller-Level Test

```
NativeMemoryService::write("db parity rule for libSQL and Postgres")
  -> HDC fingerprint stored (assert fingerprint field non-null in repo row)

NativeMemoryService::write("backend migration must keep both DBs in sync")
  -> near-duplicate of first write
  -> when hdc_native_index_enabled=true: returns NearDuplicate error
  -> when hdc_native_index_enabled=false: write succeeds (rollback path)

NativeMemoryService::search("backend migration parity")
  -> when hdc_native_index_enabled=true: result includes hdc_score field
  -> when hdc_native_index_enabled=false: hdc_score field is absent

retrieve_context(budget=4000 tokens)
  -> write 5 near-duplicate snippets, then retrieve
  -> when enabled: context snippets count is 1 (deduped), budget not wasted
  -> when disabled: context snippets count is 5 (unchanged behavior)
```

Key invariants that must hold in all modes:

- Prompt snippets are always wrapped as untrusted memory content. The HDC
  path must never bypass or weaken the host safety wrapper.
- No raw HDC vector appears in any returned snippet, tool output, or log
  line at info level or above.
- The `hdc_score` field, when present, is a bounded `f32` in `[0.0, 1.0]`
  and appears only in diagnostic output, not in the materialized prompt.

### Benchmark / Metric

| Benchmark | Target |
|---|---|
| `reindex_document_with_audit_context` overhead with HDC | p95 under 2 ms per document |
| Write-time dedup preflight (`find_similar_document`) | p95 under 3 ms for up to 10K docs in scope |
| `retrieve_context` inter-snippet dedup pass (50 snippets) | p95 under 1 ms |
| Context token savings from inter-snippet dedup | Measured per run; report min/max/median |

### Rollback

Set `IRONCLAW_HDC_NATIVE_INDEX=` (empty/unset) to disable at runtime.
Removing the `hdc` compile feature drops all three seams entirely. No
database schema change is introduced in this integration point; the
fingerprint is stored in an existing nullable metadata column. A migration
is only needed if a dedicated column is added; that decision belongs to the
DB schema owner following the dual-backend (PostgreSQL + libSQL) rule.

---

## 13. Context Admission Novelty Gate

### Why It Matters

`ProductionMemoryPromptContextService` in
`crates/ironclaw_host_runtime/src/memory_context.rs` enforces host safety
wrappers and re-validates content before admitting snippets into the context
budget. It does not perform any semantic dedup. If memory contains ten near-
duplicate entries covering the same fact (common after repeated heartbeat
writes, daily logs, or parallel agent runs), all ten can pass through the
safety check and consume context budget before the model sees any novel
content. HDC dedup at this admission gate prevents that waste without
changing the upstream memory backend or retrieval contract.

### Integration Seam

The clean seam is immediately after `MemoryService::retrieve_context()`
returns its ranked list and before the host service begins accumulating
snippets against the budget:

```rust
// crates/ironclaw_host_runtime/src/memory_context.rs
let mut admitted: Vec<LoopContextSnippet> = Vec::new();
let mut seen_fingerprints: Vec<ironclaw_hdc::HdcVector> = Vec::new();

for candidate in ranked_snippets {
    #[cfg(feature = "hdc")]
    if config.hdc_context_dedup_mode != HdcContextDedupMode::Off {
        let fp = ironclaw_hdc::encode_text(&candidate.content, codebook);
        let is_near_dup = seen_fingerprints
            .iter()
            .any(|seen| seen.hamming_similarity(&fp) >= config.hdc_dedup_threshold);
        if is_near_dup {
            telemetry::record_context_dedup_drop(&candidate, mode);
            if config.hdc_context_dedup_mode == HdcContextDedupMode::Shadow {
                // shadow: record drop but still admit
            } else {
                continue; // live: skip this snippet
            }
        }
        seen_fingerprints.push(fp);
    }
    admitted.push(candidate.into_loop_snippet());
}
```

Note: `AntiKnowledge`-authority snippets must never be dropped by this gate.
A similarity match against a positive snippet does not justify dropping a
warning. The drop rule applies only to `DailyLog`, `RecentTurn`, and
`DurableMemory` authority levels.

### Config / Feature Gate

```bash
IRONCLAW_HDC_CONTEXT_DEDUP_MODE=off      # default, no HDC at this gate
IRONCLAW_HDC_CONTEXT_DEDUP_MODE=shadow   # dedup computed and logged; snippets still admitted
IRONCLAW_HDC_CONTEXT_DEDUP_MODE=live     # near-duplicate snippets dropped before budget accumulation
```

Compile gate: `hdc` feature on `crates/ironclaw_host_runtime`. Must be off
by default. Enable `shadow` first in canary; promote to `live` only after
false-drop rate is below threshold on recorded fixtures.

### Fixture Shape

Fixture file: `tests/fixtures/hdc_memory/context_attention/near_dup_snippets.jsonl`

```json
{ "id": "ctx-001", "authority": "DurableMemory", "content": "libSQL and Postgres must both support HDC fingerprint storage.", "score": 0.92 }
{ "id": "ctx-002", "authority": "DailyLog",       "content": "Both backends require HDC fingerprint column parity.",         "score": 0.85 }
{ "id": "ctx-003", "authority": "DailyLog",       "content": "PostgreSQL and libSQL need matching fingerprint columns.",    "score": 0.78 }
{ "id": "ctx-004", "authority": "AntiKnowledge",  "content": "Do not use Postgres-only JSONB for cross-DB fingerprints.", "score": 0.76 }
{ "id": "ctx-005", "authority": "DurableMemory",  "content": "Frontend auth token lifecycle is unrelated to DB parity.",  "score": 0.55 }
```

Expected in `live` mode: ctx-001 admitted, ctx-002 dropped (near-dup of
001), ctx-003 dropped (near-dup of 001), ctx-004 admitted (AntiKnowledge
must survive), ctx-005 admitted (unrelated content).

### Caller-Level Test

```
Write 5 near-duplicate memory entries about the same db-parity fact.
Retrieve context with budget = 2000 tokens and hdc_context_dedup_mode=live.
Assert:
  - exactly 1 snippet about db-parity fact is admitted
  - AntiKnowledge snippets are NOT dropped even if near-duplicate of positive snippet
  - host safety wrappers are applied to every admitted snippet
  - total admitted token count is within budget
  - telemetry records N dropped snippets with snippet_ref and similarity
  - no raw HdcVector appears in LoopContextSnippet or in the materialized prompt
```

Test must drive `ProductionMemoryPromptContextService` directly (caller
level), not a helper similarity function alone.

### Benchmark / Metric

| Benchmark | Target |
|---|---|
| Fingerprint 50 context snippets | p95 under 2 ms total |
| Inter-snippet Hamming comparison (50 × 50) | p95 under 0.5 ms |
| Context token savings (shadow mode, 14-day sample) | Reported per user; gate on no false-drop regressions |
| False-drop rate | Anti-knowledge snippets dropped = 0; unique positive snippets dropped < 2% |

### Rollback

Set `IRONCLAW_HDC_CONTEXT_DEDUP_MODE=off` to restore original behavior
without redeployment. The shadow mode is the safe default for canary
exposure: it measures without altering model-visible context. Disable the
`hdc` compile feature to remove the code entirely.

---

## 14. Subagent Goal and Result Dedup

### Why It Matters

When multiple user turns or routines trigger overlapping goals, the Reborn
subagent system can spawn redundant subagents for the same task within a
TurnScope. Conversely, parallel subagents may produce near-identical results
that each consume delivery budget and post-processing overhead. HDC
fingerprinting at the goal-store and completion-observer boundaries prevents
both kinds of waste without changing spawn capability scopes.

### Three Integration Points

**Point A — Goal dedup at spawn time**

`SubagentGoalStore::put_goal()` in
`crates/ironclaw_reborn/src/subagent/goal_store.rs` accepts a new goal
for a TurnScope and returns a `TurnRunId`. Before persisting and spawning,
fingerprint the task description and compare against in-flight goals with
the same TurnScope:

```rust
// crates/ironclaw_reborn/src/subagent/goal_store.rs
#[cfg(feature = "hdc")]
if config.hdc_subagent_dedup_enabled {
    let candidate_fp = ironclaw_hdc::encode_text(&goal.task_description, codebook);
    if let Some(existing_run_id) = self
        .find_similar_in_flight_goal(&scope, &candidate_fp, config.hdc_dedup_threshold)
        .await?
    {
        return Ok(SubagentPutResult::ExistingRunReturned(existing_run_id));
    }
}
```

The returned `ExistingRunId` lets the caller attach to the already-running
subagent rather than spawning a duplicate. This does not change capability
scopes — the returned run was spawned with the same scope constraints.

**Point B — Result dedup at completion**

`SubagentCompletionObserver::handle_terminal()` in
`crates/ironclaw_reborn/src/subagent/completion_observer.rs` fires when a
subagent reaches a terminal state. Fingerprint the result content and
compare against sibling results already delivered within the same
TurnScope:

```rust
#[cfg(feature = "hdc")]
if config.hdc_subagent_dedup_enabled {
    let result_fp = ironclaw_hdc::encode_text(&result.content, codebook);
    if let Some(_) = self
        .find_similar_delivered_result(&scope, &result_fp, config.hdc_dedup_threshold)
        .await?
    {
        tombstone_store.mark_similar(run_id, result_fp).await?;
        return Ok(TerminalHandleOutcome::SimilarToDelivered);
    }
}
```

The `SimilarToDelivered` outcome is recorded in the tombstone but the
result is still stored. Dedup at this boundary prevents forwarding a
redundant result to the turn output but does not discard the subagent's
work from the database. (LLM data is never deleted.)

**Point C — Tombstone fingerprint field**

`SubagentResultTombstone` in
`crates/ironclaw_reborn/src/subagent/tombstone_store.rs` gains an optional
fingerprint field:

```rust
#[derive(Debug, Clone)]
pub struct SubagentResultTombstone {
    pub run_id: TurnRunId,
    pub scope: TurnScope,
    pub outcome: TombstoneOutcome,
    // ... existing fields ...
    #[cfg(feature = "hdc")]
    pub result_fingerprint: Option<[u8; 1280]>, // 10,240-bit vector as byte array
}
```

The `1280`-byte field is gated behind `#[cfg(feature = "hdc")]` so it adds
zero size when the crate is built without HDC. It is stored as BLOB in any
persistent tombstone table under the same dual-backend rule. New columns
require migration files for both PostgreSQL and libSQL.

### Config / Feature Gate

No separate env var is required for goal dedup; it follows the existing
`IRONCLAW_HDC_ENABLED` gate plus a sub-option:

```bash
IRONCLAW_HDC_SUBAGENT_DEDUP=true   # default false; requires hdc feature + IRONCLAW_HDC_ENABLED
```

### Fixture Shape

Fixture file: `tests/fixtures/hdc_memory/subagent/goal_dedup.jsonl`

```json
{ "scope": "turn:abc123", "task": "add libSQL parity for HDC fingerprint column", "action": "put_goal", "expect": "spawned" }
{ "scope": "turn:abc123", "task": "implement libSQL HDC fingerprint storage parity", "action": "put_goal", "expect": "existing_returned" }
{ "scope": "turn:abc123", "task": "write frontend CSS for dashboard", "action": "put_goal", "expect": "spawned" }
```

Fixture file: `tests/fixtures/hdc_memory/subagent/result_dedup.jsonl`

```json
{ "scope": "turn:abc123", "run_id": "run-1", "content": "Added hdc_fingerprint BLOB column to libSQL migration.", "action": "handle_terminal", "expect": "delivered" }
{ "scope": "turn:abc123", "run_id": "run-2", "content": "libSQL migration now includes hdc_fingerprint binary column.", "action": "handle_terminal", "expect": "similar_to_delivered" }
{ "scope": "turn:abc123", "run_id": "run-3", "content": "Updated gateway OAuth redirect handler.", "action": "handle_terminal", "expect": "delivered" }
```

### Caller-Level Test

```
SubagentGoalStore::put_goal(scope=turn:T1, task="add libSQL HDC parity")
  -> returns Spawned(run_id_1)

SubagentGoalStore::put_goal(scope=turn:T1, task="implement libSQL fingerprint column parity")
  -> near-duplicate of first goal
  -> when hdc_subagent_dedup=true: returns ExistingRunReturned(run_id_1)
  -> when hdc_subagent_dedup=false: returns Spawned(run_id_2) — new agent spawned

SubagentCompletionObserver::handle_terminal(run_id_1, result="Added hdc_fingerprint column.")
  -> TombstoneOutcome::Delivered
  -> tombstone.result_fingerprint is Some(_) when feature enabled

SubagentCompletionObserver::handle_terminal(run_id_2, result="libSQL now has hdc_fingerprint.")
  -> near-duplicate of run_id_1 result
  -> TombstoneOutcome::SimilarToDelivered
  -> result row still present in DB (not deleted)
  -> result NOT forwarded to turn output

Assert throughout:
  - capability scopes of both runs are unchanged
  - no cross-scope goal or result comparison occurs
  - tombstone fingerprint field absent in binary when feature is off
```

### Benchmark / Metric

| Benchmark | Target |
|---|---|
| `put_goal` fingerprint + in-flight scan (up to 50 in-flight goals) | p95 under 2 ms |
| `handle_terminal` fingerprint + sibling scan (up to 50 siblings) | p95 under 2 ms |
| Duplicate subagent spawn rate reduction (shadow mode, 7-day sample) | Measured; no false-merge regressions |

### Rollback

Set `IRONCLAW_HDC_SUBAGENT_DEDUP=false` or remove the `hdc` compile
feature. The tombstone `result_fingerprint` column defaults to NULL when the
feature is absent; no data is lost. No existing tombstone rows are modified
by enabling or disabling the feature.

---

## 15. K-Medoids HDC Clustering for Memory Audit

### Why It Matters

Episode dedup, context dedup, and anti-knowledge warnings all operate at
single-document resolution. A complementary capability is corpus-level
clustering: group all documents or all completed-turn fingerprints by
behavioral similarity, identify the representative (medoid) of each cluster,
and expose redundancy groups and knowledge gaps. This is the foundation for
memory audit, dream consolidation, and knowledge GC.

### Module Location

`crates/ironclaw_hdc/src/cluster.rs` — already implemented and exposed from `lib.rs`
under the `hdc` feature.

### Algorithm: K-Medoids PAM (Hamming Distance)

K-medoids Partitioning Around Medoids (PAM) is a better fit than k-means
for binary HDC vectors because:

- Hamming distance is cheap to compute on 10,240-bit vectors.
- The medoid is always an actual document/episode from the corpus, making
  it easy to explain and audit.
- No centroid arithmetic is required, which avoids binarization ambiguity.

The actual implemented API (not a proposal — this exists in the crate):

```rust
// crates/ironclaw_hdc/src/cluster.rs

pub struct Cluster<Id> {
    pub medoid_id: Id,
    pub members: Vec<Id>,
    /// Average intra-cluster distance (0.0 = perfect, 0.5 = random).
    pub inertia: f64,
}

/// Run k-medoids PAM. Returns `k` clusters. If k > n, returns n clusters.
/// Uses farthest-first initialization, no RNG, fully deterministic.
pub fn k_medoids<Id: Clone + PartialEq>(
    vectors: &[(Id, HdcVector)],
    k: usize,
) -> Result<Vec<Cluster<Id>>, HdcError>;
```

Note: the earlier design proposed `k_medoids_pam` with `ClusterInput`/`ClusterResult` types.
The implementation uses `k_medoids` with generic `(Id, HdcVector)` pairs and `Cluster<Id>`.
The `intra_cluster_max_distance` field is not on the current struct; use `inertia` (mean distance).
If richer per-cluster stats are needed, add them to `Cluster<Id>` rather than introducing a new type.

Complexity: O(k × n²) per iteration, O(k × n² × max_iterations) overall.
Acceptable for n ≤ 10,000 items in an offline audit job. For larger corpora,
expose a `k_medoids_sample` variant that runs PAM on a random sample and
assigns remaining items to the nearest medoid.

### Three Use Cases

**Memory audit CLI**

```bash
ironclaw memory hdc audit --cluster-k 20 --min-cluster-size 3
```

Output per cluster:

```text
Cluster 4  (medoid: "notes/db-parity-libsql.md", 8 members)
  max_distance:  1847 bits  (18.0 % of 10240)
  mean_distance: 1204 bits  (11.8 %)
  members:
    notes/db-parity-libsql.md     [medoid]
    notes/db-parity-postgres.md   dist=1102
    notes/backend-parity-rule.md  dist=1341
    ...
  recommendation: 7 near-duplicate members; consider consolidating into medoid.
```

The audit command is read-only and diagnostic. It does not delete or merge
documents.

**Dream consolidation input**

The offline replay/consolidation job (Integration Track 6) needs to group
completed turns by behavioral similarity before comparing failures and
successes. K-medoids clustering provides the grouping:

```text
load all EpisodePatternRecord fingerprints in scope
  -> k_medoids(fingerprints, k=30)
  -> for each cluster: compare failure ratio against overall base rate
  -> clusters with high failure concentration are consolidation candidates
```

**Knowledge GC**

A knowledge GC pass preserves at least one document per cluster (the medoid)
and marks redundant cluster members for review before any deletion. Deletion
still requires user confirmation or an approved promotion gate. The GC pass
never deletes anything automatically.

```rust
pub struct KnowledgeGcRecommendation {
    pub cluster_medoid_id: String,
    pub candidates_for_review: Vec<String>,  // non-medoid members above similarity threshold
    pub rationale: String,
}
```

### Fixture Shape

Fixture file: `tests/fixtures/hdc_memory/cluster/db_parity_redundancy.jsonl`

Each line is a document record with an `id`, `content`, and optional
ground-truth `cluster_label`:

```json
{ "id": "doc-1", "content": "libSQL and Postgres both need HDC column.", "cluster_label": "db_parity" }
{ "id": "doc-2", "content": "Both backends require binary fingerprint storage.", "cluster_label": "db_parity" }
{ "id": "doc-3", "content": "Frontend dashboard CSS grid layout.", "cluster_label": "frontend" }
```

Test: run `k_medoids(vectors, 2)` and assert that `db_parity` ground-truth
documents land in one cluster and `frontend` in another.

### Caller-Level Test

```
encode 10 documents: 7 about db-parity (varying paraphrase), 3 about frontend auth

k_medoids(vectors, k=2)
  -> cluster 0 contains ≥ 6 of the 7 db-parity documents
  -> cluster 1 contains all 3 frontend documents
  -> each Cluster.medoid_id references a real id from the input
  -> no cluster has members from both ground-truth groups (purity ≥ 0.9)
```

### Benchmark / Metric

| Benchmark | Target |
|---|---|
| k_medoids(n=1000, k=20) | Under 500 ms on a single core |
| k_medoids(n=10000, k=30) | Under 30 s offline; alert if exceeded |
| Cluster purity on fixture corpus (ground-truth labels) | ≥ 0.85 |

### Rollback

The cluster module is compile-gated behind `feature = "hdc"`. Removing the
feature flag removes all cluster code. The offline audit/GC jobs are invoked
explicitly; they have no hot-path presence and cannot affect live agent runs.

---

## 16. Fixture Corpus Test Harness

### Status: Implemented

`tests/workspace_hdc_fixtures.rs` exists and covers all three fixture families
from `tests/fixtures/hdc_memory/`. Run with:

```bash
cargo test --features hdc --test workspace_hdc_fixtures
```

### What Is Covered

**Dedup scenarios (9 fixtures)** — per-scenario tests assert `similarity ∈ [min, max]`.
Test names: `fixture_dedup_exact_duplicate_similarity_range`,
`fixture_dedup_reformatted_similarity_range`, `fixture_dedup_paraphrase_similarity_range`,
`fixture_dedup_added_paragraph_similarity_range`, `fixture_dedup_same_topic_different_facts_is_unique`,
`fixture_dedup_same_tags_different_content_is_unique`, `fixture_dedup_same_path_different_content_is_unique`,
`fixture_dedup_content_subset_is_similar`, `fixture_dedup_translated_is_similar`.

**Heartbeat scenarios (4 fixtures)** — feeds observations through
`DecayingBundleAccumulator` and asserts novelty at declared cycle numbers.
Test names: `fixture_heartbeat_repeated_observation_converges`,
`fixture_heartbeat_recurring_after_gap`,
`fixture_heartbeat_similar_different_metric_is_novel`,
`fixture_heartbeat_completely_novel_all_novel`.

**Search scenarios (4 fixtures)** — asserts best relevant document ranks
above best irrelevant document via `top_k_scan`.
Test names: `fixture_search_structural_query_ranks_relevant`,
`fixture_search_tag_heavy_query_ranks_relevant`,
`fixture_search_path_oriented_query_ranks_relevant`,
`fixture_search_semantic_only_query_ranks_relevant`.

**Manifest consistency tests** — verifies all referenced fixture files exist,
all JSONL files are parseable, and all `expected_similarity_range` values
are valid `[0.0, 1.0]` intervals.

### Remaining Gaps

- The dedup harness asserts similarity range but not the `expected_decision`
  label from `manifest.json` (`"unique"` / `"similar"` / `"duplicate"`).
  That assertion requires wiring `check_dedup` through configurable thresholds
  per scenario.
- Heartbeat tests use `DecayingBundleAccumulator` directly. The
  `HeartbeatNoveltyAccumulator` runner-level path in `heartbeat_hdc.rs`
  is covered separately.

### Benchmark / Metric

| Metric | Target |
|---|---|
| Time to run all fixture scenarios | Under 5 s with `--features hdc` |
| Dedup scenario pass rate | 9/9 (100%) |
| Heartbeat novelty scenario pass rate | All declared cycle expectations met |
| Search ranking pass rate | 4/4 (100%) |

### Rollback

This test file adds no production code and no database schema. Removing
`tests/workspace_hdc_fixtures.rs` is the complete rollback. The fixture
files and `manifest.json` remain in place for documentation and future
use.
