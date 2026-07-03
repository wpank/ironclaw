# References: Cross-References to Source Documents

> **Navigation**:
> [README](README.md) |
> [Detailed Rankings](detailed-rankings.md) |
> [Implementation Sketches](implementation-sketches.md) |
> [Quick Wins](quick-wins.md) |
> [Synergy Analysis](synergy-analysis.md) |
> [Benchmarking Plans](benchmarking-plans.md) |
> [References (you are here)](references.md)

---

## Source Corpus

All 25 items derive from the Roko source corpus: [https://github.com/wpank/roko](https://github.com/wpank/roko)

The corpus spans 18+ crates, 200K+ lines of Rust, and 155+ research documents covering
hyperdimensional computing, online learning, dream consolidation, DAG execution engines,
gate verification, conductor anomaly detection, and more.

---

## Per-Item Source Cross-References

| Rank | Concept | Primary Source Doc | IronClaw Integration Doc |
|------|---------|-------------------|--------------------------|
| 1 | Metacognitive Monitor | [../../agent-intelligence/agent-patterns.md](../../agent-intelligence/agent-patterns.md) | [../../implementation/03-ironclaw-integration-recipes.md](../../implementation/03-ironclaw-integration-recipes.md) |
| 2 | Ebbinghaus Decay | [../../core-concepts/universal-engram.md](../../core-concepts/universal-engram.md) | [../../implementation/schemas/02-storage-and-migrations.md](../../implementation/schemas/02-storage-and-migrations.md) |
| 3 | BLAKE3 Content Dedup | [../../core-concepts/universal-engram.md](../../core-concepts/universal-engram.md) | [../../implementation/schemas/02-storage-and-migrations.md](../../implementation/schemas/02-storage-and-migrations.md) |
| 4 | Robust Statistics | [../../core-concepts/mathematical-primitives.md](../../core-concepts/mathematical-primitives.md) | [../../implementation/benchmarking/01-measurement-framework.md](../../implementation/benchmarking/01-measurement-framework.md) |
| 5 | Composable Scorers | [../../agent-intelligence/agent-patterns.md](../../agent-intelligence/agent-patterns.md) | [../../implementation/03-ironclaw-integration-recipes.md](../../implementation/03-ironclaw-integration-recipes.md) |
| 6 | Hierarchical Cancellation | [../../execution-verification/runtime-infrastructure.md](../../execution-verification/runtime-infrastructure.md) | [../../implementation/03-ironclaw-integration-recipes.md](../../implementation/03-ironclaw-integration-recipes.md) |
| 7 | Cascade Router | [../../agent-intelligence/online-learning.md](../../agent-intelligence/online-learning.md) | [../../implementation/benchmarking/02-feature-playbooks.md](../../implementation/benchmarking/02-feature-playbooks.md) |
| 8 | Cognitive Speed Labels | [../../core-concepts/cognitive-architecture.md](../../core-concepts/cognitive-architecture.md) | [../../implementation/03-ironclaw-integration-recipes.md](../../implementation/03-ironclaw-integration-recipes.md) |
| 9 | HDC Similarity Engine | [../../core-concepts/hyperdimensional-computing/README.md](../../core-concepts/hyperdimensional-computing/README.md) | [../../implementation/benchmarking/02-feature-playbooks.md](../../implementation/benchmarking/02-feature-playbooks.md) |
| 10 | Gate Pipeline (Rungs 1–4) | [../../execution-verification/gate-verification.md](../../execution-verification/gate-verification.md) | [../../implementation/benchmarking/02-feature-playbooks.md](../../implementation/benchmarking/02-feature-playbooks.md) |
| 11 | Enhanced Heartbeat | [../../agent-intelligence/dream-consolidation.md](../../agent-intelligence/dream-consolidation.md) | [../../implementation/03-ironclaw-integration-recipes.md](../../implementation/03-ironclaw-integration-recipes.md) |
| 12 | DAG Execution Engine | [../../execution-verification/dag-execution.md](../../execution-verification/dag-execution.md) | [../../implementation/03-ironclaw-integration-recipes.md](../../implementation/03-ironclaw-integration-recipes.md) |
| 13 | EventBus with Replay Ring | [../../execution-verification/runtime-infrastructure.md](../../execution-verification/runtime-infrastructure.md) | [../../implementation/03-ironclaw-integration-recipes.md](../../implementation/03-ironclaw-integration-recipes.md) |
| 14 | Conductor | [../../execution-verification/conductor-anomaly.md](../../execution-verification/conductor-anomaly.md) | [../../implementation/benchmarking/02-feature-playbooks.md](../../implementation/benchmarking/02-feature-playbooks.md) |
| 15 | Resumable Checkpoints | [../../agent-intelligence/agent-patterns.md](../../agent-intelligence/agent-patterns.md) | [../../implementation/schemas/02-storage-and-migrations.md](../../implementation/schemas/02-storage-and-migrations.md) |
| 16 | Declarative TOML Tools | [../../ecosystem/plugin-extension.md](../../ecosystem/plugin-extension.md) | [../../implementation/03-ironclaw-integration-recipes.md](../../implementation/03-ironclaw-integration-recipes.md) |
| 17 | User Engagement PAD | [../../agent-intelligence/affect-engine.md](../../agent-intelligence/affect-engine.md) | [../../implementation/03-ironclaw-integration-recipes.md](../../implementation/03-ironclaw-integration-recipes.md) |
| 18 | Full Dream Consolidation | [../../agent-intelligence/dream-consolidation.md](../../agent-intelligence/dream-consolidation.md) | — |
| 19 | Budget Composition (VCG) | [../../context-memory/budget-composition.md](../../context-memory/budget-composition.md) | — |
| 20 | Code Intelligence | [../../context-memory/code-intelligence.md](../../context-memory/code-intelligence.md) | — |
| 21 | NEAR On-Chain Reputation | [../../ecosystem/chain-reputation/README.md](../../ecosystem/chain-reputation/README.md) | — |
| 22 | Pheromone System | [../../core-concepts/cognitive-architecture.md](../../core-concepts/cognitive-architecture.md) | — |
| 23 | Pure SM Extraction | [../../execution-verification/runtime-infrastructure.md](../../execution-verification/runtime-infrastructure.md) | — |
| 24 | Full Affect Engine | [../../agent-intelligence/affect-engine.md](../../agent-intelligence/affect-engine.md) | — |
| 25 | TDA / Sheaves | [../../core-concepts/mathematical-primitives.md](../../core-concepts/mathematical-primitives.md) | — |

---

## IronClaw Source Files Referenced

Files that implementations in this matrix touch directly:

| File | Features That Touch It |
|------|----------------------|
| `src/agent/agentic_loop.rs` | Rank 1 (Metacognitive Monitor integration), Rank 6 (Hierarchical Cancellation) |
| `src/agent/cost_guard.rs` | Rank 1 (EWMA cost tracking extension) |
| `src/agent/self_repair.rs` | Rank 1 (existing job-level detection; Monitor adds turn-level) |
| `src/agent/heartbeat.rs` | Rank 11 (Enhanced Heartbeat extends this file) |
| `src/agent/metacognitive.rs` | Rank 1 (new file) |
| `src/agent/cancel.rs` | Rank 6 (new file) |
| `src/agent/cognitive_speed.rs` | Rank 8 (new file) |
| `src/workspace/decay.rs` | Rank 2 (new file) |
| `src/workspace/dedup.rs` | Rank 3 (new file) |
| `src/workspace/repository.rs` | Rank 2 (search query modification), Rank 3 (dedup methods) |
| `src/workspace/search.rs` | Rank 9 (HDC as third RRF signal) |
| `src/tools/builtin/memory.rs` | Rank 2 (on_access calls), Rank 3 (dedup check in memory_write) |
| `src/tools/builder/gate.rs` | Rank 10 (new file) |
| `src/tools/builder/validation.rs` | Rank 10 (existing file; gate.rs extends) |
| `src/util.rs` | Rank 4 (append robust statistics functions) |
| `src/evaluation/scorer.rs` | Rank 5 (new file) |
| `src/evaluation/mod.rs` | Rank 5 (add `pub mod scorer;`) |
| `src/config/agent.rs` | Rank 1 (`metacognitive_monitor_enabled` flag) |
| `src/estimation/learner.rs` | Rank 4 (replace arithmetic mean with `trimmed_mean`) |
| `crates/ironclaw_llm/src/smart_routing.rs` | Rank 7 (Cascade Router replaces static scorer) |
| `crates/ironclaw_llm/src/router/` | Rank 7 (new module: bandit.rs, cascade.rs) |
| `crates/ironclaw_hdc/` | Rank 9 (new crate) |
| `Cargo.toml` | Rank 9 (add `crates/ironclaw_hdc`); Rank 7 (add `ndarray`) |

---

## Database Migration Files

All migrations must be created for both backends per the dual-backend rule (`src/db/CLAUDE.md`):

| Migration | PostgreSQL File | libSQL File | Features |
|-----------|----------------|------------|----------|
| Memory decay columns | `migrations/20260101000001_memory_decay.sql` | `migrations/libsql/20260101000001_memory_decay.sql` | Rank 2 |
| Content hash column | `migrations/20260101000002_memory_content_hash.sql` | `migrations/libsql/20260101000002_memory_content_hash.sql` | Rank 3 |
| Metacognitive events | `migrations/20260101000003_metacognitive_events.sql` | `migrations/libsql/20260101000003_metacognitive_events.sql` | Rank 1 (optional, for metrics) |
| Routing events | `migrations/20260101000004_routing_events.sql` | `migrations/libsql/20260101000004_routing_events.sql` | Rank 7 |
| Gate pipeline events | `migrations/20260101000005_gate_pipeline_events.sql` | `migrations/libsql/20260101000005_gate_pipeline_events.sql` | Rank 10 |
| HDC fingerprints | `migrations/20260101000006_hdc_fingerprints.sql` | `migrations/libsql/20260101000006_hdc_fingerprints.sql` | Rank 9 |

---

## Academic References

### Ebbinghaus Decay

- Ebbinghaus, H. (1885). *Über das Gedächtnis: Untersuchungen zur experimentellen Psychologie*.
  Duncker & Humblot. The original forgetting curve: `R = e^{-t/S}`.
- Wozniak, P. & Gorzelanczyk, E. (1994). Optimization of repetition spacing in the practice of
  learning. *Acta Neurobiologiae Experimentalis*, 54(1), 59–62. Foundation of SuperMemo/SM-2.

### LinUCB (Cascade Router)

- Li, L., Chu, W., Langford, J., & Schapire, R. (2010). A contextual-bandit approach to
  personalized news article recommendation. *Proceedings of WWW 2010*.
  arXiv:1003.0146. The LinUCB algorithm used in the Cascade Router.

### Hyperdimensional Computing (HDC Similarity)

- Kanerva, P. (2009). Hyperdimensional computing: An introduction to computing in distributed
  representation with high-dimensional random vectors. *Cognitive Computation*, 1(2), 139–159.
  Foundation of all HDC operations (bind, bundle, rotate, Hamming similarity).
- Imani, M., et al. (2019). Hierarchical hyperdimensional computing for energy efficient
  classification. *Design Automation Conference (DAC) 2019*.

### Robust Statistics

- Hampel, F. R., et al. (1986). *Robust Statistics: The Approach Based on Influence Functions*.
  Wiley. MAD and robust z-score.
- Hodges, J. L. & Lehmann, E. L. (1963). Estimates of location based on rank tests.
  *Annals of Mathematical Statistics*, 34(2), 598–611. Hodges-Lehmann estimator.
- Rousseeuw, P. J. & Croux, C. (1993). Alternatives to the median absolute deviation.
  *Journal of the American Statistical Association*, 88(424), 1273–1283.

### Cancellation Tokens

- `tokio_util::sync::CancellationToken` documentation:
  [https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html)

---

## Companion Strategy Documents

| Document | Path | Relationship |
|----------|------|-------------|
| Master index | [../../README.md](../../README.md) | Knowledge base root and navigation |
| Integration roadmap | [../integration-roadmap.md](../integration-roadmap.md) | 4-phase plan with success criteria |
| Architecture overview | [../../reference/architecture-overview.md](../../reference/architecture-overview.md) | Roko system design |
| Benchmarking framework | [../../implementation/benchmarking/01-measurement-framework.md](../../implementation/benchmarking/01-measurement-framework.md) | Full measurement methodology |
| Feature playbooks | [../../implementation/benchmarking/02-feature-playbooks.md](../../implementation/benchmarking/02-feature-playbooks.md) | Per-feature measurement plans |
| Implementation recipes | [../../implementation/03-ironclaw-integration-recipes.md](../../implementation/03-ironclaw-integration-recipes.md) | IronClaw-native build plans |
