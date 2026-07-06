# Strategy

This folder turns captured agent-system ideas into IronClaw build decisions. It
is a planning layer, not an implementation spec. The deeper subsystem docs and
the current code remain authoritative when work begins.

The strategy is deliberately conservative:

- Build narrow, reversible improvements before architectural changes.
- Measure current IronClaw behavior before claiming improvement.
- Keep external context self-contained as captured concepts, not live source
  links.
- Prefer existing IronClaw modules, traits, and Reborn runtime boundaries over new crates unless a feature clearly needs one.

## Documents

| Document | Use it for |
|----------|------------|
| [integration-roadmap.md](integration-roadmap.md) | Phase order, rollout gates, risks, and implementation principles. |
| [ux-improvements.md](ux-improvements.md) | REPL, TUI, and web UX improvements distilled from captured interface patterns. |
| [priority-matrix/README.md](priority-matrix/README.md) | Scoring model, priority tiers, and recommended build order. |
| [priority-matrix/detailed-rankings.md](priority-matrix/detailed-rankings.md) | One-page rationale for each candidate capability. |
| [priority-matrix/quick-wins.md](priority-matrix/quick-wins.md) | First PRs that can be started with low design risk. |
| [priority-matrix/implementation-sketches.md](priority-matrix/implementation-sketches.md) | Integration contracts and test targets, not porting instructions. |
| [priority-matrix/benchmarking-plans.md](priority-matrix/benchmarking-plans.md) | Baselines, metrics, canary gates, and rollback thresholds. |
| [priority-matrix/synergy-analysis.md](priority-matrix/synergy-analysis.md) | Dependencies and combinations that change feature value. |
| [priority-matrix/references.md](priority-matrix/references.md) | Internal source map, captured context, and research anchors. |

## Decision Flow

1. Start with [priority-matrix/README.md](priority-matrix/README.md) to select the next candidate.
2. Check [synergy-analysis.md](priority-matrix/synergy-analysis.md) for hard prerequisites.
3. Use [benchmarking-plans.md](priority-matrix/benchmarking-plans.md) to define the baseline before writing code.
4. Follow [integration-roadmap.md](integration-roadmap.md) for rollout and rollback.
5. Before implementation, read the relevant subsystem spec listed in `AGENTS.md`.

## Current Recommendation

Do not start with the largest captured systems. The first useful sequence is:

1. Robust statistics for heavy-tailed cost and latency data.
2. Exact memory deduplication.
3. Memory decay with archival, not deletion.
4. Turn-level stuck-loop and cost-runaway monitoring.
5. A small reusable scoring interface.
6. Cancellation propagation through actual tool-call boundaries.

Only after those are measured should IronClaw consider adaptive routing, gate expansion, HDC search, or background consolidation.
