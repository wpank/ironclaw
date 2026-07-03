# Feature-Specific Threat Models

This file expands the generic risk register into concrete feature-level threats.

| Feature | Data classification | Main threat | Required test | Rollback validation |
|---|---|---|---|---|
| HDC memory search | memory metadata, derived fingerprints | false merge leaks or hides user fact | contradictory preference fixture | HDC ranking disabled, memory search baseline works |
| Signal records | memory body hashes, lineage, taints | sensitive memory loses taint or retention policy | sensitive memory fixture | taint filter still excludes sensitive records |
| Cascade router | prompts, provider choice, cost | cheap provider receives private/high-risk data | private prompt routing fixture | static provider selected after disable |
| Progressive gates | command output, artifacts, file paths | artifact leaks secret or bypasses approval | secret-shaped stderr fixture | gates disabled or report-only, artifacts retained safely |
| Provider conductor | provider health, routing bias | false degradation pushes traffic to unsafe provider | healthy-provider false positive fixture | conductor observe-only |
| Dream consolidation | transcripts, derived memories | hallucinated or sensitive derived memory | ambiguous transcript fixture | scheduled job disabled, low-confidence derived hidden |
| DAG runner | task graph, tool side effects | skipped branch misses required gate | missing predicate fixture | serial runner fallback |
| Event replay | session events | cross-user event replay | unauthorized cursor fixture | replay disabled, history fetch baseline |
| Code search | source snippets, paths | private path/source exposed in telemetry | path redaction fixture | index disabled, FTS/vector baseline |
| Plugin hooks | tool permissions, network access | plugin escapes sandbox | denied network fixture | extension disabled/revoked |
| Local reputation | actor scores, evidence hashes | unfair reputation penalty or collusion | collusion fixture | selection ignores reputation |
| Control-plane projection | dashboard/session state | user sees another user's state | cross-session projection fixture | projection route disabled |

## Prompt And Artifact Retention

Default:

- keep ids, hashes, metrics, statuses;
- redact command output;
- do not retain raw prompts in production telemetry;
- keep raw fixtures only in local benchmark artifacts.

Exception process:

```text
raw content needed:
  reason:
  retention:
  redaction:
  reviewer:
  deletion path:
```

