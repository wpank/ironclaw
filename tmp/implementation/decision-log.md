# Decision Log

This log records implementation decisions that affect the `tmp/implementation`
plan. It avoids assumptions about external code.

## ADR-001: Runtime Flags For All Experimental Features

Decision: every experimental behavior is behind `experimental.<feature>` and
defaults off.

Rationale:

- Rollback must be a config operation before it is a code revert.
- Shadow/canary stages need runtime control.
- Compile features are for dependency footprint, not rollout behavior.

Consequences:

- The flag inventory is mandatory.
- Caller-level tests must prove flag-off baseline behavior.
- Stale flags need removal criteria when a feature reaches default.

## ADR-002: Keep New Learning State In Structured Metadata Or New Tables

Decision: small per-record state may live in `metadata`; queryable or shared
state gets an explicit table with PostgreSQL/libSQL parity.

Rationale:

- Metadata keeps early experiments cheap.
- Tables are required when callers need indexes, joins, retention, or DB-level
  constraints.

Consequences:

- Derived memories keep `origin_ids`, confidence, and taint labels.
- Rollback hides or ignores experimental state instead of deleting user data.

## ADR-003: Use Robust Statistics Before Adaptive Routing

Decision: rollout and routing metrics use median, p95, MAD/trimmed summaries,
and bounded reward components before any adaptive router can change behavior.

Rationale:

- Provider latency/cost data is heavy-tailed.
- Adaptive routing without stable summaries amplifies noise.

Consequences:

- Cascade router starts in shadow mode.
- Mean-only benchmark reports are not promotion evidence.

## ADR-004: Clean-Room Implementation

Decision: implementation uses IronClaw-owned code and public design notes. Do
not import external research crates or copy inaccessible source paths into code
comments.

Rationale:

- The repo needs stable ownership, licensing clarity, and reviewable code.
- Captured notes are background context, not a dependency.

Consequences:

- New code lives in the owning IronClaw module.
- PRs cite the relevant local implementation doc, benchmark, and ADR.

## ADR-005: Metrics Exclude Raw Private Content

Decision: metrics and exposure events may store ids, hashes, bounded classes,
counts, cost, latency, token counts, and redaction status only.

Rationale:

- Rollout artifacts can be long-lived and widely inspected.
- Safety regressions should be detectable without persisting raw content.

Consequences:

- Raw prompts, secrets, file bodies, private paths, arbitrary URLs, and raw
  provider errors are disallowed in metrics.
- Redacted artifacts are referenced by id when debugging detail is needed.
