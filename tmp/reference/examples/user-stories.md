# User Stories

These stories describe the user-visible reason to build the proposed features.

## Individual User: Stop Repeating My Preferences

As an individual user, I want the assistant to remember my preferences without
writing the same fact repeatedly, so memory search stays clean.

Relevant mechanisms:

- Signal content identity.
- HDC near-duplicate detection.
- decay and reinforcement.

Acceptance:

- Repeated preference writes produce one canonical memory or a linked duplicate.
- The assistant can explain which memory it used.

## Developer: Verify Generated Code Before I See It

As a developer, I want generated code to pass the obvious checks before it is
shown as ready, so I do not become the first verifier.

Relevant mechanisms:

- progressive gates.
- code intelligence.
- DAG execution.
- control-plane event stream.

Acceptance:

- Compile/lint/test failures appear as structured remediation.
- False blocks stay below the rollout threshold.

## Operator/Admin: Debug Why The Agent Stopped

As an operator, I want to inspect a stuck or stopped run from one place, so I can
see whether the cause was cancellation, provider health, gate failure, or budget.

Relevant mechanisms:

- EventBus replay.
- control-plane projection.
- conductor.
- gate verdicts.

Acceptance:

- Dashboard and API agree on final state.
- Reconnect does not lose or duplicate terminal events.

## Extension Author: Ship A Tool Safely

As an extension author, I want a clear permission and feedback path, so my tool
can be installed, tested, and revoked safely.

Relevant mechanisms:

- plugin manifest.
- sandbox permission declaration.
- reputation events.
- user approval.

Acceptance:

- denied permissions fail closed.
- revocation stops future tool calls.
- feedback events can improve trust without bypassing sandboxing.

## Cost-Conscious User: Cap Background Work

As a cost-conscious user, I want background learning to stay inside an explicit
budget, so better memory does not create surprise spending.

Relevant mechanisms:

- heartbeat budget.
- dream consolidation.
- cost guard.
- rollout metrics.

Acceptance:

- background spend is visible.
- disabling the feature stops scheduled work.
- low-confidence derived memories do not outrank user-authored facts.

