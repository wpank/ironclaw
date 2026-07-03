# User Stories

These stories describe the user-visible reason to build each capability. The
acceptance checks are phrased so they can become caller-level tests or
benchmark fixture assertions.

## Individual User: Stop Repeating My Preferences

As an individual user, I want the assistant to remember my preferences without
writing the same fact repeatedly, so memory search stays clean.

Relevant mechanisms:

- Signal content identity.
- HDC near-duplicate detection.
- Decay and reinforcement.

Acceptance:

- Repeated preference writes produce one canonical memory or a linked duplicate.
- The assistant can explain which memory it used.
- Benchmark with [`memory-dedup.yaml`](../../implementation/benchmarking/scenarios/memory-dedup.yaml).

## Developer: Verify Generated Code Before I See It

As a developer, I want generated code to pass the obvious checks before it is
shown as ready, so I do not become the first verifier.

Relevant mechanisms:

- Progressive gates.
- Code intelligence.
- DAG execution.
- Control-plane event stream.

Acceptance:

- Compile/lint/test failures appear as structured remediation.
- False blocks stay below the rollout threshold.
- Benchmark with [`gate-pipeline.yaml`](../../implementation/benchmarking/scenarios/gate-pipeline.yaml).

## Operator/Admin: Debug Why The Agent Stopped

As an operator, I want to inspect a stuck or stopped run from one place, so I can
see whether the cause was cancellation, provider health, gate failure, or budget.

Relevant mechanisms:

- EventBus replay.
- Control-plane projection.
- Conductor.
- Gate verdicts.

Acceptance:

- Dashboard and API agree on final state.
- Reconnect does not lose or duplicate terminal events.

## Extension Author: Ship A Tool Safely

As an extension author, I want a clear permission and feedback path, so my tool
can be installed, tested, and revoked safely.

Relevant mechanisms:

- Plugin manifest.
- Sandbox permission declaration.
- Reputation events.
- User approval.

Acceptance:

- Denied permissions fail closed.
- Revocation stops future tool calls.
- Feedback events can improve trust without bypassing sandboxing.
- Add a benchmark fixture before reputation affects production tool choice.

## Cost-Conscious User: Cap Background Work

As a cost-conscious user, I want background learning to stay inside an explicit
budget, so better memory does not create surprise spending.

Relevant mechanisms:

- Heartbeat budget.
- Dream consolidation.
- Cost guard.
- Rollout metrics.

Acceptance:

- Background spend is visible.
- Disabling the feature stops scheduled work.
- Low-confidence derived memories do not outrank user-authored facts.
- Benchmark with [`dream-consolidation.yaml`](../../implementation/benchmarking/scenarios/dream-consolidation.yaml).
