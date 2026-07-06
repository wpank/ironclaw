# Practical Real-World Use Cases

These examples show how the reference concepts become practical IronClaw
workflows. They describe desired behavior and measurement points without
assuming extra source inputs.

## Use Case 1: Cheaper Routine Questions

Problem:

A user asks many simple questions in the same day: time zones, summaries,
formatting, and short rewrites. Sending all of them to the primary model wastes
money.

IronClaw touchpoints:

- `crates/ironclaw_llm` routing and provider abstractions.
- Static safety rules for privacy and high-stakes requests.
- Cost guard and routing metric events.

IronClaw flow:

1. Static routing classifies the request as low risk and non-private.
2. A learning router evaluates whether a cheaper provider is eligible.
3. If confidence is high, canary traffic can route to the cheap model.
4. If the answer is uncertain or the user corrects it, reward is updated
   downward.
5. If provider health degrades, provider-health bias lowers that provider's
   score.

Measurement:

- Cost/request before and after.
- User correction rate.
- Fallback-to-primary rate.
- Latency distribution.

Benchmark fixture: [`tmp/implementation/benchmarking/scenarios/cascade-router.yaml`](../../implementation/benchmarking/scenarios/cascade-router.yaml).

## Use Case 2: Memory That Stops Duplicating Itself

Problem:

The assistant writes the same preference or project fact repeatedly in slightly
different words, making memory search noisy.

IronClaw touchpoints:

- `src/workspace/` memory write/search.
- BLAKE3 content identity.
- HDC similarity as a candidate near-duplicate signal.
- Decay, reinforcement, and taint metadata.

IronClaw flow:

1. Before `memory_write`, compute content hash and HDC fingerprint.
2. Search nearby fingerprints and exact content hashes.
3. Link exact duplicates or record near-duplicate candidates before writing or
   merging.
4. Reinforce old memory stability when the user confirms or reuses it.
5. Keep taint on derived memories until verified.

Measurement:

- Duplicate memory rate.
- memory-search relevance.
- Number of stale memories pruned or downweighted.

Benchmark fixture: [`tmp/implementation/benchmarking/scenarios/memory-dedup.yaml`](../../implementation/benchmarking/scenarios/memory-dedup.yaml).

## Use Case 3: Generated Code With Progressive Verification

Problem:

An agent writes code that compiles locally but violates scope, fails hidden
tests, or regresses performance.

IronClaw touchpoints:

- Code-generation caller and tool execution path.
- Progressive gate rungs selected from changed files and risk.
- Redacted artifact retention.
- Benchmark gate for performance-sensitive modules only.

IronClaw flow:

1. Code generation caller produces artifact and declared scope.
2. Rung selector chooses gates based on changed files and risk.
3. Rung 0 validates format and manifest.
4. Rung 1 runs compile/check.
5. Rung 2 runs clippy/lint where appropriate.
6. Rung 3 runs targeted tests.
7. Benchmark gate runs only for performance-sensitive modules.
8. Failed gate returns remediation to the agent loop.

Measurement:

- Defects caught before user review.
- Gate runtime.
- False positive rate.
- Remediation success after one repair turn.

Benchmark fixture: [`tmp/implementation/benchmarking/scenarios/gate-pipeline.yaml`](../../implementation/benchmarking/scenarios/gate-pipeline.yaml).

## Use Case 4: Background Learning Without Surprise Bills

Problem:

The assistant repeats the same mistakes across sessions because it never
reflects on failed attempts, but unconstrained background LLM work can burn
money.

IronClaw touchpoints:

- `src/agent/` heartbeat/background runtime.
- Workspace memory writes for derived lessons.
- Confidence staging for LLM-generated memory.
- Explicit background budget ledger.

IronClaw flow:

1. Heartbeat selects recent high-surprise or high-risk episodes.
2. CostGuard approves a small background budget.
3. Replay generates lessons and failure patterns.
4. Lessons are written as low-confidence derived memories.
5. Later successful use promotes the memory.
6. Unused derived memories decay.

Measurement:

- Background cost/day.
- Repeated failure rate.
- Promoted memory acceptance rate.

Benchmark fixture: [`tmp/implementation/benchmarking/scenarios/dream-consolidation.yaml`](../../implementation/benchmarking/scenarios/dream-consolidation.yaml).

## Use Case 5: Provider Degradation Before Users Notice

Problem:

A provider starts getting slower and then fails. A reactive circuit breaker
trips only after several bad requests.

IronClaw touchpoints:

- LLM provider wrapper metrics.
- Forecasting over latency and retry pressure.
- Existing reactive circuit breaker.
- Observe/canary/enforce rollout modes.

IronClaw flow:

1. LLM wrapper emits latency/error/cost events.
2. Latency watcher forecasts next-step breach.
3. Error watcher tracks retry pressure.
4. Compound detector sees latency plus errors rising.
5. Router receives a bias to use another healthy provider.
6. Existing circuit breaker remains the final hard stop.

Measurement:

- Failed requests avoided.
- False warning rate.
- Provider switch count.
- Oscillation cooldown effectiveness.

Benchmark fixture: [`tmp/implementation/benchmarking/scenarios/provider-degradation.yaml`](../../implementation/benchmarking/scenarios/provider-degradation.yaml).

## Use Case 6: Multi-File Refactor From The Web UI

Problem:

The user asks from the browser: "Refactor webhook signature validation so all
routes share one verifier." The task touches route handlers, auth helpers,
tests, and docs. A linear agent loop can lose track of dependencies and leave
the dashboard stale.

IronClaw touchpoints:

- `src/channels/web/` browser-facing API and event stream.
- Code intelligence for symbol and dependency discovery.
- Budget composition for prompt packing.
- DAG execution for independent subtasks.
- Gate verification for generated code.
- Control-plane events for web progress.

IronClaw flow:

1. Web channel creates a normal agent turn.
2. Code index finds verifier symbols, route call sites, and tests.
3. Prompt composer packs only relevant symbols and recent memory.
4. DAG runner splits implementation, tests, and docs into dependent nodes.
5. Gates block submission until compile, lint, and targeted tests pass.
6. Event stream updates the browser with node status and gate verdicts.

Measurement:

- Refactor completion rate.
- Changed-file conflict count.
- Gate pass rate after first repair.
- SSE reconnect event loss.

Benchmark fixtures:

- [`tmp/implementation/benchmarking/scenarios/workspace-code-search.yaml`](../../implementation/benchmarking/scenarios/workspace-code-search.yaml)
- [`tmp/implementation/benchmarking/scenarios/gate-pipeline.yaml`](../../implementation/benchmarking/scenarios/gate-pipeline.yaml)

## Use Case 7: Assistant Learns A Recurring Deployment Failure

Problem:

The same deployment failure appears every few weeks, but the exact symptom text
changes enough that simple search misses prior fixes.

IronClaw touchpoints:

- Signal lineage for previous incidents.
- Dream consolidation for lessons learned.
- Failure-pattern extraction from successful repairs.
- HDC retrieval for paraphrased symptoms.

IronClaw flow:

1. Failed deployment session is tagged high surprise and high utility.
2. Dream job extracts a derived "deployment failure pattern" memory.
3. Later, HDC search retrieves the pattern despite different wording.
4. Gate feedback confirms the fix.
5. Signal confidence is promoted after reuse.

Measurement:

- Time to useful retrieval.
- Repeated failure rate.
- Derived memory precision.
- Background cost.

Benchmark fixture: [`tmp/implementation/benchmarking/scenarios/dream-consolidation.yaml`](../../implementation/benchmarking/scenarios/dream-consolidation.yaml).

## Use Case 8: Extension Marketplace Trust Decision

Problem:

A user installs a third-party tool that requests filesystem and network
permissions. The assistant should help the user decide without bypassing the
sandbox.

IronClaw touchpoints:

- Plugin manifest permission model.
- Local reputation events.
- Approval gates.
- Sandbox policy.

IronClaw flow:

1. Extension declares required permissions in its manifest.
2. Reputation ledger summarizes prior local outcomes and signed evidence if
   available.
3. Approval UI shows permission scope and trust evidence.
4. Tool execution still routes through `ToolDispatcher`.
5. User revocation disables future calls immediately.

Measurement:

- Denied permission attempts.
- Sandbox violation count.
- User approval reversal rate.
- Failed tool execution rate.

Benchmark fixture: no fixture exists yet. Add an extension-trust fixture before
using reputation to affect production tool selection.

## Use Case 9: Long-Running Task Survives Restart

Problem:

A long task is interrupted by process restart. The user needs to know what
completed, what was cancelled, and what can safely resume.

IronClaw touchpoints:

- Event-sourced orchestration.
- DAG node snapshots.
- Bounded EventBus replay.
- Control-plane projection.
- Gate artifact retention.

IronClaw flow:

1. Each DAG node writes a run snapshot before effects.
2. Event bus records progress events with replay cursors.
3. On restart, runner reconciles durable snapshots with event log.
4. Completed idempotent nodes are not re-run.
5. Browser projection shows resumed, skipped, failed, and pending nodes.

Measurement:

- Resume correctness over fixture restarts.
- Duplicate side-effect count.
- Event replay gap count.
- User-visible recovery time.

Benchmark fixture: no single fixture exists yet. Combine a DAG snapshot fixture
with the SSE reconnect regression test before rollout.
