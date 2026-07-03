# Synergy Analysis

Synergy claims are hypotheses. They should influence sequencing, but each combined value claim still needs a benchmark or caller-level test.

## Dependency Map

| Cluster | Build first | Then | Hard dependency? |
|---------|-------------|------|------------------|
| Memory hygiene | BLAKE3 dedup | Ebbinghaus decay, then optional HDC search | Dedup before decay is recommended, not required. Decay before full consolidation is hard. |
| Reliability | Robust statistics | Metacognitive monitor, cancellation, gate metrics | Robust stats before monitor thresholds is recommended. |
| Routing | Baseline logging | Cascade router, then cognitive speed labels and conductor | Shadow-mode routing data before active router is hard. |
| Verification | Composable scorers | Gate expansion | Scorers are useful but not mandatory if the first gate has one criterion. |
| Background learning | Decay and cost caps | Enhanced heartbeat, full dream consolidation | Decay and budget caps before consolidation are hard. |
| Workflow automation | Tool dispatch contract | DAG runner | Dispatch-through-tools is hard; no second agent loop. |

## Memory Hygiene Cluster

Pieces:

- Exact dedup reduces repeated facts before ranking logic is tuned.
- Decay lowers stale memory prominence without destroying data.
- HDC is only worth adding if it improves compositional retrieval beyond current FTS/vector RRF.

Recommended order:

1. Dedup exact duplicates.
2. Add decay metadata and archival semantics.
3. Measure search quality with and without decay.
4. Prototype HDC offline before adding it to active ranking.

Risk:

- Memory features can look successful by reducing volume while silently hiding useful context. Track relevant@10, restore events, and identity/system exemptions.

## Reliability Cluster

Pieces:

- Robust statistics stabilize thresholds and cost projections.
- Metacognitive monitor catches repeated failed turns.
- Cancellation makes stop actions effective across tools and processes.
- Gate expansion catches defects before completion.

Recommended order:

1. Robust statistics.
2. Monitor in conservative or observe-only mode.
3. Cancellation through real tool boundaries.
4. Gate expansion after command-safety review.

Risk:

- False positives frustrate users. Measure false interventions and false gate failures, not just caught failures.

## Routing Cluster

Pieces:

- Baseline logging provides model, cost, latency, and outcome data.
- Cascade router can learn only from completed episodes.
- Cognitive speed labels are useful only if consumed by routing, budgets, or UX.
- Conductor health signals need enough provider metrics to avoid noise.

Recommended order:

1. Add episode logging without changing routing.
2. Run cascade decisions in shadow mode.
3. Enable a small canary only after safety overrides are validated.
4. Add cognitive labels if they improve decisions.
5. Add conductor signals after routing has a place to consume them.

Risk:

- A learner can optimize cost by lowering quality. Quality and safety guardrails must be first-class metrics.

## Verification Cluster

Pieces:

- Composable scorers create a reusable result shape.
- Existing gate/builder validation should be extended before creating new verification infrastructure.
- Rungs should be added progressively: compile, lint, tests, symbol checks.

Recommended order:

1. Add one real scorer caller.
2. Add compile/lint rungs for a narrow language scope.
3. Add tests and symbol checks after diagnostics and redaction are safe.

Risk:

- Verification can execute untrusted code or leak secrets through diagnostics. Command construction, path validation, truncation, and redaction are required.

## Background Learning Cluster

Pieces:

- Decay and dedup make memory state cleaner.
- Enhanced heartbeat can strengthen high-value memories without user turns.
- Full consolidation and "dream" features add autonomous LLM calls and should remain research until cost and value are validated.

Recommended order:

1. Ship decay and archival.
2. Add explicit cost caps and active-session exclusion.
3. Run any consolidation in observe-only or low-call mode.
4. Require sampled human review before promotion.

Risk:

- Autonomous background work can spend money and write low-quality memory. Default off and cap calls in code.

## Interaction Effects To Measure

| Combination | Expected benefit | Measurement |
|-------------|------------------|-------------|
| Dedup + decay | Cleaner search results and fewer stale duplicates. | Duplicate rate, stale-hit rate, relevant@10. |
| Robust stats + monitor | Fewer false cost-runaway interventions. | False intervention rate on long legitimate traces. |
| Scorers + gates | Shared quality language across evaluation and verification. | Gate decision consistency and defect catch rate. |
| Router + cognitive labels | Better model choice for simple vs complex turns. | Cost, latency, quality by label bucket. |
| Decay + heartbeat | Background reinforcement of useful memory. | Manual precision sample of strengthened memories. |

## Anti-Synergies

- HDC plus decay can overcomplicate search if FTS/vector RRF already performs well. Benchmark first.
- Cascade router plus cognitive labels can create opaque routing if labels are not logged with decisions.
- Gate expansion plus DAG execution can produce a second workflow system if not routed through existing Reborn and tool-dispatch paths.
- Full dream consolidation plus weak cost controls can create unbounded background spend.
- UI event expansion plus existing SSE can fragment event taxonomies. Extend current web platform boundaries instead.
