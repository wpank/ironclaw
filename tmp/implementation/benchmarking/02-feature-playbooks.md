# Feature Benchmark Playbooks

## HDC Similarity

Questions:

- How fast is fingerprint generation?
- How fast is brute-force scan?
- Does HDC recover useful related items that FTS misses?

Datasets:

- 1k, 10k, 100k synthetic memories.
- Real IronClaw memory export with sensitive fields redacted.
- Code-symbol corpus from the current workspace.

Metrics:

| Metric | Target |
|---|---|
| Fingerprint generation | under 50 microseconds per item |
| 100k scan | under 10ms unoptimized, under 2ms optimized |
| Memory overhead | 1280 bytes/vector plus index overhead |
| Retrieval lift | +5% relevant@10 over FTS-only on paraphrase queries |

Guardrails:

- No memory search regression when HDC is disabled.
- No persistent schema added without PostgreSQL/libSQL parity.

## Cascade Router

Questions:

- Does the router reduce spend?
- Does cheap-model routing reduce quality?
- How many observations are needed before stable decisions?

Scenarios:

| Scenario | Expected model |
|---|---|
| greeting/time/simple rewrite | cheap |
| multi-file refactor | primary |
| security-sensitive request | primary |
| tool-heavy uncertain task | primary until confidence improves |
| repeated simple project status request | cheap after confidence |

Metrics:

| Metric | Target |
|---|---|
| Cost/request | 20-50% reduction |
| Quality pass rate | no worse than -2 percentage points |
| Fallback rate | below 10% after warmup |
| Warmup | stable per class after 200-500 observations |

## Gate Pipeline

Questions:

- Does progressive verification catch defects earlier?
- Does it add acceptable latency?
- Does it produce useful remediation?

Metrics:

| Metric | Target |
|---|---|
| Compile/lint defect detection | +20% over current validation |
| False failure rate | under 3% on unchanged good artifacts |
| Rung 0-2 latency | under 30s for small generated changes |
| Remediation usefulness | 70% of failures include actionable next command |

Guardrails:

- Truncate stdout/stderr.
- Redact secrets in artifacts.
- Never run untrusted generated code outside the sandbox policy.

## Conductor

Questions:

- Does prediction fire before reactive failure thresholds?
- Does it avoid provider oscillation?
- Does it create noisy false alarms?

Metrics:

| Metric | Target |
|---|---|
| Lead time | warning at least one request before reactive breaker |
| False positive rate | under 5% per provider-hour |
| Oscillation cooldown | triggers after configured alternation threshold |
| Waste avoided | fewer failed retries during provider degradation |

## Dream Consolidation

Questions:

- Does background replay improve future task success?
- Does it create low-value or hallucinated memories?
- Is cost bounded?

Metrics:

| Metric | Target |
|---|---|
| Background cost | under configured daily/monthly cap |
| Promoted memory precision | 80% manually accepted sample |
| Future retrieval lift | +5% relevant@10 for repeated task classes |
| Retry reduction | fewer repeated failures in rehearsed categories |

Guardrails:

- Derived memories carry taint and confidence.
- No promotion without later use or verification.
- Heartbeat exits immediately when budget is exhausted.

## Control Plane And Events

Questions:

- Can TUI/web/SSE consume one event stream without custom wiring?
- Is event emission cheap enough to enable by default?

Metrics:

| Metric | Target |
|---|---|
| Event publish overhead | under 50 microseconds p95 |
| Subscriber lag loss | bounded and observable |
| SSE reconnect catch-up | recent events replayed without full log scan |
| Event schema compatibility | versioned and tested |

## Additional Feature Coverage

These rows cover the remaining numbered documents so every canonical file has a
benchmark target even when it is not one of the initial six playbooks.

| Doc | Feature | Primary metric | Guardrail |
|---|---|---|---|
| 03 | Affect hints | user correction rate non-increasing | approval/safety behavior unchanged |
| 04 | DAG runner | wall-clock reduction on independent nodes >= 15% | final state equals serial baseline |
| 08 | Local reputation | bad-tool selection rate down >= 20% | no permanent blacklist without evidence |
| 09 | Prompt composition | input tokens/request down >= 15% | quality no worse than -2pp |
| 10 | Signal records | duplicate storage down >= 30% | false merge rate < 2% |
| 11 | Robust stats | false alert rate down >= 50% on outlier fixtures | aggregation p95 < 1ms |
| 12 | Code intelligence | top-5 symbol recall +15pp | index time per KLOC within budget |
| 13 | Cognitive speeds | urgent task latency down >= 10% | background starvation count = 0 |
| 14 | Runtime bus/cancel | cancellation p95 < 100ms | no orphaned subprocesses |
| 15 | Wave scheduler | parallel throughput +20% | file-conflict retry rate down |
| 16 | Plugin hooks | install/activate/remove succeeds 100 cycles | permission escape count = 0 |
| 17 | Metacognition | runaway token spend down >= 30% | false intervention rate < 5% |
| 20 | Persistence parity | same contract tests pass on both DBs | migration rollback preserves reads |
| 21 | MCP/editor protocol | event loss count = 0 | permission denial enforced |
| 22 | Language support | symbol precision/recall reported per language | parser errors degrade gracefully |
| 23 | Control plane | route/event p95 within budget | auth/origin/rate checks unchanged |
| 24 | Contracts/local ledger | failed chain calls fall back locally | no mainnet dependency in tests |
| 28 | Plan runner | schema errors caught before execution | verification commands reproducible |

## Rollout And Rollback Gates

| Feature | Rollout gate | Rollback gate |
|---|---|---|
| HDC search | top-5 relevance improves with p95 search latency inside budget | false duplicate rate > 2% or latency > +10% |
| Cascade router | cost/request down >= 20%, quality no worse than -2pp | quality drops > 2pp or fallback rate rises > 5pp |
| Gates | escaped defects down and false blocks < 5% | secret in artifact or false blocks >= 5% |
| Conductor | degraded-provider spend down >= 50% | healthy-provider false positives > 3% |
| Dreams | useful-memory hit rate +10pp within budget | background cap exceeded or sensitive data leak |
| Control plane | reconnect fixture loses zero events | auth/origin/rate-limit regression |
