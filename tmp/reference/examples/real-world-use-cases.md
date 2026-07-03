# Practical Real-World Use Cases

These examples show how the concepts in docs 01-28 combine into workflows a
user can understand and an implementer can test.

## Use Case 1: Cheaper Routine Questions

Problem:

A user asks many simple questions in the same day: time zones, summaries,
formatting, and short rewrites. Sending all of them to the primary model wastes
money.

Roko-derived pieces:

- Cascade Router learns cheap-model suitability.
- Conductor watches provider latency and error rate.
- CostGuard enforces user budget.

IronClaw flow:

1. Existing static `SmartRoutingProvider` classifies the request as simple.
2. LinUCB checks historical reward for the cheap model on similar requests.
3. If confidence is high, route to cheap model.
4. If the answer is uncertain or user corrects it, update reward downward.
5. If provider health degrades, Conductor biases away from that provider.

Measurement:

- Cost/request before and after.
- User correction rate.
- Fallback-to-primary rate.
- p95 latency.

## Use Case 2: Memory That Stops Duplicating Itself

Problem:

The assistant writes the same preference or project fact repeatedly in slightly
different words, making memory search noisy.

Roko-derived pieces:

- BLAKE3 content identity.
- HDC similarity.
- Ebbinghaus decay and reinforcement.
- Taint metadata.

IronClaw flow:

1. Before `memory_write`, compute content hash and HDC fingerprint.
2. Search nearby fingerprints and exact content hashes.
3. Merge duplicate or near-duplicate memories instead of writing a new row.
4. Reinforce old memory stability when the user confirms or reuses it.
5. Keep taint on derived memories until verified.

Measurement:

- Duplicate memory rate.
- relevant@10 for memory search.
- Number of stale memories pruned or downweighted.

## Use Case 3: Generated Code With Progressive Verification

Problem:

An agent writes code that compiles locally but violates scope, fails hidden
tests, or regresses performance.

Roko-derived pieces:

- Gate rungs.
- Acceptance contracts.
- Forensic artifact chain.
- Benchmark regression gate.

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

## Use Case 4: Background Learning Without Surprise Bills

Problem:

The assistant repeats the same mistakes across sessions because it never
reflects on failed attempts, but unconstrained background LLM work can burn
money.

Roko-derived pieces:

- Dream replay utility.
- Threat rehearsal.
- Confidence staging.
- Cost-bounded heartbeat.

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

## Use Case 5: Provider Degradation Before Users Notice

Problem:

A provider starts getting slower and then fails. A reactive circuit breaker
trips only after several bad requests.

Roko-derived pieces:

- Holt forecasting.
- Compound event detection.
- Intervention levels.

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

## Use Case 6: Multi-File Refactor From The Web UI

Problem:

The user asks from the browser: "Refactor webhook signature validation so all
routes share one verifier." The task touches route handlers, auth helpers,
tests, and docs. A linear agent loop can lose track of dependencies and leave
the dashboard stale.

Roko-derived pieces:

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

- refactor completion rate.
- changed-file conflict count.
- gate pass rate after first repair.
- SSE reconnect event loss.

## Use Case 7: Assistant Learns A Recurring Deployment Failure

Problem:

The same deployment failure appears every few weeks, but the exact symptom text
changes enough that simple search misses prior fixes.

Roko-derived pieces:

- Signal lineage for previous incidents.
- Dream consolidation for lessons learned.
- Threat rehearsal for failure pattern extraction.
- HDC retrieval for paraphrased symptoms.

IronClaw flow:

1. Failed deployment session is tagged high surprise and high utility.
2. Dream job extracts a derived "deployment failure pattern" memory.
3. Later, HDC search retrieves the pattern despite different wording.
4. Gate feedback confirms the fix.
5. Signal confidence is promoted after reuse.

Measurement:

- time to first useful retrieval.
- repeated failure rate.
- derived memory precision.
- background cost.

## Use Case 8: Extension Marketplace Trust Decision

Problem:

A user installs a third-party tool that requests filesystem and network
permissions. The assistant should help the user decide without bypassing the
sandbox.

Roko-derived pieces:

- Plugin manifest permission model.
- Reputation ledger.
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

- denied permission attempts.
- sandbox violation count.
- user approval reversal rate.
- failed tool execution rate.

## Use Case 9: Long-Running Task Survives Restart

Problem:

A long task is interrupted by process restart. The user needs to know what
completed, what was cancelled, and what can safely resume.

Roko-derived pieces:

- Event-sourced orchestration.
- DAG node snapshots.
- bounded EventBus replay.
- control-plane projection.
- gate artifact retention.

IronClaw flow:

1. Each DAG node writes a run snapshot before effects.
2. Event bus records progress events with replay cursors.
3. On restart, runner reconciles durable snapshots with event log.
4. Completed idempotent nodes are not re-run.
5. Browser projection shows resumed, skipped, failed, and pending nodes.

Measurement:

- resume correctness over fixture restarts.
- duplicate side-effect count.
- event replay gap count.
- user-visible recovery time.
