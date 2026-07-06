# Per-Feature Action And Measurement Matrix

This matrix maps concepts to implementation targets inside this codebase.
Concept notes are background, not source dependencies.

## Acceptance Pattern

Every implementation slice needs:

1. A runtime flag defaulting off.
2. A caller-level test that drives the side effect.
3. A scenario or benchmark with baseline and candidate metrics.
4. A rollback switch and persistence story.

## Canonical Matrix

| Feature | Owner target | First artifact | Caller-level test | Metric gate |
| --- | --- | --- | --- | --- |
| Signal records | `src/workspace/`, `src/db/` | content hash + signal row | memory write/search through tools on both DBs | duplicate storage down >= 30%, relevance loss <= 2pp |
| HDC memory search | `src/workspace/` | HDC candidate generator | workspace search facade | top-5 relevance up >= 10pp, p95 <= +10% |
| Cascade router | `crates/ironclaw_llm/` | shadow router | provider factory/wrapper | cost down >= 20%, quality >= -2pp |
| Progressive gates | `src/tools/` | compile/lint/unit/security rungs | generated-code caller blocks failed rung | escaped defects down >= 20%, false block <= 5% |
| Provider conductor | `crates/ironclaw_llm/` | observe-mode watcher | circuit breaker wrapper | degraded spend down >= 50%, false positive <= 3% |
| Dream consolidation | `src/agent/`, workspace | budgeted replay job | heartbeat -> memory facade | useful-memory hits up >= 10pp, no sensitive leak |
| DAG runner | product workflow/runtime | typed workflow executor | plan parse -> runner -> executor | final state equals serial, wall clock down >= 15% |
| Event replay | web gateway | bounded cursor replay | HTTP/SSE/WebSocket reconnect | event loss 0, auth/origin unchanged |
| Workspace code search | `src/workspace/` | symbol graph + RRF merge | workspace index/search facade | top-5 symbol recall up >= 15pp |
| Local reputation | trust/reputation + DB | off-chain score projection | outcome event path | bad-tool selection down, DB parity |
| Prompt composition | agent prompt builder | budget allocator | real agent request path | tokens/request down >= 15%, quality >= -2pp |
| Affect engine | `src/agent/` | PAD policy metadata | agent turn loop | safety unchanged, update p95 < 1ms |
| Swarm coordination | agent coordination | conflict-aware scheduler | multi-run plan fixture | throughput up, conflicting edits serialize |

## Benchmark Template

```yaml
feature: experimental.<feature>
flag_default: "off"
caller_boundary: module::function_or_handler
baseline: current behavior
candidate: flagged behavior
primary_metric: bounded target
guardrails:
  - quality
  - latency
  - safety
rollback: disable flag and verify baseline behavior
```

## Quantification Definitions

| Phrase | Meaning |
| --- | --- |
| cost/request down X% | `(baseline_median - candidate_median) / baseline_median >= X` |
| quality no worse than -2pp | `candidate_pass_rate + 0.02 >= baseline_pass_rate` |
| p95 latency no worse than +10% | `candidate_p95 <= baseline_p95 * 1.10` |
| false block <= 5% | valid fixture blocked / valid fixtures <= 0.05 |
| no safety regression | zero new policy/auth/secret/approval failures |
