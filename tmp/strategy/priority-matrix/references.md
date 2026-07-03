# References

This file records where the strategy came from without requiring access to the original upstream source. Roko references below are captured source-path names from the analyzed corpus, included only to preserve the source trace.

## Companion Strategy Files

| File | Purpose |
|------|---------|
| [README.md](README.md) | Scoring model and priority tiers. |
| [detailed-rankings.md](detailed-rankings.md) | Rationale for each candidate. |
| [implementation-sketches.md](implementation-sketches.md) | Minimal IronClaw integration contracts. |
| [quick-wins.md](quick-wins.md) | First-PR scopes. |
| [benchmarking-plans.md](benchmarking-plans.md) | Measurement and rollout gates. |
| [synergy-analysis.md](synergy-analysis.md) | Dependencies and combined-value hypotheses. |
| [../integration-roadmap.md](../integration-roadmap.md) | Phase plan and risk controls. |

## Captured Roko Context

| Strategy area | Captured Roko concept or path name | IronClaw interpretation |
|---------------|------------------------------------|-------------------------|
| CLI visual language | `crates/roko-cli/src/inline/symbols.rs` | Use consistent status symbols and labels, not Roko branding. |
| Streaming phases | `crates/roko-cli/src/inline/primitives/streaming.rs` | Show phase-specific progress instead of generic "thinking". |
| Tool call rendering | `crates/roko-cli/src/inline/primitives/tool_call.rs` | Collapse tool calls by default with clear expansion affordance. |
| Progress tree | `crates/roko-cli/src/inline/primitives/progress_tree.rs` | Use hierarchical progress only for real multi-step plans. |
| Gate display | `crates/roko-cli/src/inline/primitives/gate_block.rs` | Surface verification rungs with bounded diagnostics. |
| Error block | `crates/roko-cli/src/inline/primitives/error_block.rs` | Include location, retry state, and next action in failures. |
| Session summary | `crates/roko-cli/src/inline/primitives/session_summary.rs` | Summarize cost, tool count, and outcome at turn/session end. |
| TUI status bar | `crates/roko-cli/src/tui/widgets/status_bar.rs` | Keep compact status sections for branch, state, counts, and shortcuts. |
| Toasts and approvals | `crates/roko-cli/src/tui/modals/notification.rs`, `approval.rs` | Make transient notifications and approval gates visually distinct. |
| SSE events | `crates/roko-serve/src/events.rs`, `routes/sse.rs` | Prefer typed lifecycle events and replay-aware SSE infrastructure. |
| Setup wizard | `crates/roko-cli/src/commands/setup.rs` | Use numbered setup steps with clear validation and recovery. |
| Memory decay/dedup | Captured "universal engram" notes | Archive stale memory and merge exact duplicates; do not delete. |
| Online routing | Captured LinUCB/router notes | Learn model routing only after shadow-mode episode collection. |
| HDC | Captured hyperdimensional-computing notes | Consider HDC as an optional retrieval signal after benchmarks. |
| Gates | Captured verification pipeline notes | Add verification rungs incrementally through existing boundaries. |

## IronClaw Areas To Check Before Coding

| Area | Paths to inspect |
|------|------------------|
| Agent loop and monitoring | `src/agent/agentic_loop.rs`, `src/agent/self_repair.rs`, `src/agent/cost_guard.rs`, `src/agent/CLAUDE.md` |
| Workspace memory | `src/workspace/`, `src/workspace/README.md`, `src/tools/builtin/memory.rs` |
| Database parity | `src/db/`, `src/db/CLAUDE.md` |
| Tool dispatch and sandboxing | `src/tools/dispatch.rs`, `src/tools/README.md`, `src/tools/wasm/` |
| Gate and Reborn runtime | `crates/ironclaw_engine/src/gate/`, `crates/ironclaw_engine/src/executor/`, `crates/ironclaw_engine/CLAUDE.md` |
| LLM routing | `crates/ironclaw_llm/src/smart_routing.rs`, `crates/ironclaw_llm/CLAUDE.md` |
| Web SSE and status | `src/channels/web/platform/sse.rs`, `src/channels/web/CLAUDE.md` |
| Setup/onboarding | `src/setup/README.md`, `src/setup/wizard.rs` |

## Research Anchors

These are conceptual anchors, not implementation instructions.

| Concept | Anchor |
|---------|--------|
| Forgetting curves | Ebbinghaus-style exponential retention and spaced repetition. |
| Contextual bandits | LinUCB-style exploration/exploitation for model routing. |
| HDC | Kanerva-style high-dimensional binary vectors and Hamming similarity. |
| Robust statistics | Median, trimmed mean, MAD, and robust z-scores for heavy-tailed data. |
| Cancellation | Hierarchical cancellation tokens and explicit subprocess cleanup. |
| Verification gates | Progressive compile/lint/test/symbol checks with safe command execution. |

## Stale Reference Policy

- Do not add live upstream URLs to these strategy files.
- Captured path names are acceptable when they explain source traceability.
- If a linked internal file moves, update this file in the same branch.
- If implementation changes feature status, check `FEATURE_PARITY.md` and relevant subsystem docs.
