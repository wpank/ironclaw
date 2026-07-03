# Feature-Specific Threat Models

| Feature | Assets | Main abuse case | Fixture | Expected fail-closed behavior |
| --- | --- | --- | --- | --- |
| Signal records / HDC memory | memory metadata, fingerprints, derived links | false merge hides or leaks a user fact | contradictory preference fixture | dedupe/ranking disabled, baseline memory search works |
| Cascade router | prompt classes, provider choice, cost data | cheap provider used for high-risk request | high-risk routing fixture | static safety router wins |
| Progressive gates | command output, file paths, diagnostics | unredacted secret in gate artifact | secret-like compiler output fixture | artifact rejected and feature disabled |
| Provider conductor | provider health and fallback decisions | oscillation or false degradation alert | alternating healthy/degraded provider fixture | observe/off mode restores reactive breaker |
| Dream consolidation | transcript-derived memories | sensitive detail promoted into memory | private channel fixture | derived memory redacted, hidden, or not written |
| Event replay | session events and cursors | cross-session replay | unauthorized cursor fixture | replay disabled, snapshot/history fetch only |
| Workspace code search | symbols, snippets, paths | private path or source body exposed in telemetry | path redaction fixture | index disabled, FTS/vector baseline works |
| Local reputation | evidence, scores, subject ids | forged or stale evidence changes selection | bad evidence fixture | local ledger ignores event and keeps previous score |

## Artifact Retention

Keep ids, hashes, statuses, bounded metrics, redaction status, and artifact refs.
Drop or redact prompts, file bodies, secrets, private paths, raw stack traces,
and provider raw errors before persistence.
