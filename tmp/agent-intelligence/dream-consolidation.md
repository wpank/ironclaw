# Dream Consolidation

Dream consolidation is idle-time processing over existing IronClaw records. It should produce candidate memory notes, warnings, and routing advice. It must not retrain models, delete LLM data, or invent a parallel memory system.

## Goals

- Extract durable lessons from completed turns, jobs, tool calls, gates, and user feedback.
- Improve workspace memory quality by turning repeated evidence into concise notes.
- Rehearse known failure modes as warnings or playbook entries.
- Produce non-binding routing and recovery advice for later shadow evaluation.

## Current IronClaw Boundaries

| Concern | Current Home | Integration Rule |
| --- | --- | --- |
| Idle scheduling | `src/agent/heartbeat.rs`, `src/config/heartbeat.rs`, `HEARTBEAT.md` | Reuse heartbeat-style scheduling and quiet-hour controls. |
| Memory destination | `src/workspace/` | Write candidate notes to workspace paths; do not create a separate memory store. |
| Tool/action source | `src/context::ActionRecord`, `src/tools/dispatch.rs`, `src/tools/execute.rs` | Read audit records after they are persisted. Do not bypass tool safety or approval. |
| LLM calls | `crates/ironclaw_llm::LlmProvider`, `CostGuard` | Use normal provider chain and budget checks. |
| Persistence | `src/db/` trait and both backends | Add DB operations only when PostgreSQL and libSQL support are planned together. |
| Reborn | product-workflow/composition/runtime boundaries | Feed summaries through existing runtime events, not a separate runner. |

## Processing Phases

1. Select candidate episodes: completed turns/jobs, repeated tool errors, successful repairs, high-cost retries, user corrections, and accepted outputs.
2. Score utility: prefer recent, surprising, repeated, user-visible, or costly events. Avoid overfitting to one-off noise.
3. Distill: produce short candidate notes with evidence references.
4. Validate: check for contradictions against existing workspace memory and current code/docs when relevant.
5. Stage: keep candidates separate from curated memory until confirmed.
6. Promote: write to appropriate workspace paths after confidence or user approval criteria are met.

## Outputs

| Output | Destination | Enforcement |
| --- | --- | --- |
| Candidate insight | workspace staging path or DB-backed staging table | Never injected automatically as system prompt until promoted. |
| Warning/playbook | workspace project or daily path | Advisory first. |
| Routing hint | routing-advice record | Shadow only until online routing validates it. |
| Memory quality update | workspace metadata or staged rewrite | Preserve source records and version history. |

## Scheduling and Budget

Use explicit budgets:

- max cycles per day,
- max LLM calls per cycle,
- max tokens/cost per cycle,
- max records scanned per cycle,
- max concurrent heartbeat/consolidation workers,
- per-tenant isolation.

If budget is exhausted, stop cleanly and persist progress. A consolidation failure must not interrupt active user work.

## Feature Modes

| Mode | Behavior |
| --- | --- |
| `off` | No consolidation work. |
| `shadow` | Analyze records and emit metrics/logs only. |
| `stage` | Write candidate notes to a staging area. |
| `promote` | Promote notes that meet validation criteria. |

Default should be `off` or `shadow` for new installations. Promotion should require strong evidence and an easy rollback path.

## Validation Rules

- Every promoted note needs provenance: source record ids, timestamps, scope, and confidence.
- Do not store secrets or raw private payloads in distilled notes.
- Do not delete or truncate LLM records. Cleanup may evict caches only.
- Keep workspace semantics file-like: paths, versioning, hybrid search indexing, and identity/system-prompt rules remain owned by `src/workspace/`.
- If a note changes behavior through retrieval or prompt assembly, add caller-level tests at the retrieval/prompt boundary.

## Tests

- Unit-test scoring and staging transitions.
- Integration-test a heartbeat-triggered shadow cycle against a small fixture of action/job records.
- Workspace test: staged notes are searchable only in intended scope and do not alter identity files.
- DB parity test if new tables are added.
- Reborn contract test if consolidation output affects a user-visible Reborn workflow.

## Do Not Implement

- A new background loop independent of heartbeat/routine/runtime scheduling.
- A second tool execution path for consolidation actions.
- Hardcoded model slugs or provider-specific escalation strings.
- Long code listings taken from external experiments.
- Performance promises. Measure local cost, latency, retrieval quality, and false promotion rate after implementation.
