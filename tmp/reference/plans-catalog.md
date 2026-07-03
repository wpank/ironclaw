# Roko Implementation Plans Catalog

Reference for the captured implementation plans from the `plans/` source namespace.
Source paths are provenance labels, not links to an accessible checkout.

> **Cross-references**: [architecture-overview.md](architecture-overview.md) — crate map | [v2-depth-research.md](v2-depth-research.md) — depth docs | `../implementation/` — IronClaw-native build plans | `../implementation/09-plan-runner-readiness.md` — IronClaw plan-runner contract

---

## What These Plans Are

Captured machine-executable specifications consumed by Roko's plan runner to dispatch Claude agents that write, test, and verify code changes to the captured Roko codebase. Each plan is represented as a directory containing `tasks.toml` with a `[meta]` section and `[[task]]` entries.

| Task Field | Description |
|---|---|
| `id` | Unique identifier (T1, T2, ...) |
| `tier` | `mechanical`, `focused`, `integrative`, `architectural` |
| `model_hint` | LLM to execute this task (haiku/sonnet/opus) |
| `max_loc` | Hard cap on lines of code produced |
| `files` | Exact files the agent may modify |
| `role` | Safety contract role (gates which tools may be used) |
| `depends_on` | Ordered dependency list within the plan |
| `context.anti_patterns` | Explicit "Do NOT" instructions encoding past mistakes |
| `[[task.verify]]` | Multi-phase verification: structural grep, compile, test |

| Tier | Model | LoC Budget |
|---|---|---|
| mechanical | claude-haiku-4-5 | 5–20 |
| focused | claude-sonnet-4 | 20–60 |
| integrative | claude-sonnet or opus | 30–80 |
| architectural | claude-opus-4-6 | 50–150 |

---

## Queue Structure

### Completed (directories removed)

| Plan | Description |
|---|---|
| `W01-wire-system-prompts` | System prompt composition |
| `P06-process-management` | Process lifecycle management |
| `P07-autofix-retry` | Autofix retry (`97b15c085`) |

### Primary Queue (P08–P34, 27 plans)

Sequential execution order: P08 → P09 → P10 → P11 → P12 → P13 → P14 → P15 → P16 → P17 → P18 → P19 → P20 → P21 → P22 → P23 → P24 → P25 → P26 → P27 → P28 → P29 → P30 → P31 → P32 → P33 → P34

### Standalone Queues

| Queue | Tasks | Status |
|---|---|---|
| `e2e-smoke` | 2 | Ready |
| `architecture-defi-critical-path` | 3 | Ready |

### Superseded (do not run)

| Queue | Tasks | Superseded By |
|---|---|---|
| `self-dev-ux` | 55 | P08–P34 (feedback audit 2026-05-08) |
| `self-dev-extras` | 11 | P08–P34 |

---

## Master Summary Table

| Plan | Title | Source path | Tasks | Theme | IronClaw |
|---|---|---|---|---|---|
| P08 | Search Command Fix | `plans/P08-search-command-fix/tasks.toml` | 4 | Bug fixes | Medium |
| P09 | Tool Alias Fix | `plans/P09-tool-alias-fix/tasks.toml` | 3 | Bug fixes | **High** |
| P10 | Slash Command Flags | `plans/P10-slash-command-flags/tasks.toml` | 5 | Bug fixes | Medium |
| P11 | Runner V2 Default | `plans/P11-runner-v2-default/tasks.toml` | 5 | Runner infra | **High** |
| P12 | Runner Parallelism | `plans/P12-runner-parallelism/tasks.toml` | 5 | Runner infra | **High** |
| P13 | Rate Limit Retry | `plans/P13-rate-limit-retry/tasks.toml` | 4 | Resilience | **High** |
| P14 | Gate Rung Fix | `plans/P14-gate-rung-fix/tasks.toml` | 3 | Quality gates | Medium |
| P15 | Error Recovery Wiring | `plans/P15-error-recovery-wiring/tasks.toml` | 5 | Resilience | **High** |
| P16 | Safety Contracts | `plans/P16-safety-contracts/tasks.toml` | 5 | Safety | **High** |
| P17 | CLI Output Format | `plans/P17-cli-output-format/tasks.toml` | 6 | CLI/TUI | Low |
| P18 | TUI Agent Data | `plans/P18-tui-agent-data/tasks.toml` | 5 | CLI/TUI | Low |
| P19 | Cascade Router ACP | `plans/P19-cascade-router-acp/tasks.toml` | 6 | ACP/Editor | Medium |
| P20 | Zero Config | `plans/P20-zero-config/tasks.toml` | 5 | Provider UX | Medium |
| P21 | ACP Streaming | `plans/P21-acp-streaming/tasks.toml` | 5 | ACP/Editor | Medium |
| P22 | ACP Tool Permission | `plans/P22-acp-tool-permission/tasks.toml` | 5 | ACP/Editor | **High** |
| P23 | PRD Pipeline Fix | `plans/P23-prd-pipeline-fix/tasks.toml` | 6 | PRD/Workspace | Medium |
| P24 | Workspace Paths | `plans/P24-workspace-paths/tasks.toml` | 4 | PRD/Workspace | Low |
| P25 | MCP-ACP Passthrough | `plans/P25-mcp-acp-passthrough/tasks.toml` | 4 | ACP/Editor | **High** |
| P26 | HDC Similarity Lookup | `plans/P26-hdc-similarity-lookup/tasks.toml` | 4 | Learning | Medium |
| P27 | Provider Error UX | `plans/P27-provider-error-ux/tasks.toml` | 4 | Provider UX | Medium |
| P28 | Image Support | `plans/P28-image-support/tasks.toml` | 5 | Provider UX | Medium |
| P29 | Develop Command Wire | `plans/P29-develop-command-wire/tasks.toml` | 3 | ACP/Editor | Low |
| P30 | Onboarding Doctor | `plans/P30-onboarding-doctor/tasks.toml` | 4 | Onboarding | Medium |
| P31 | Note and Context | `plans/P31-note-and-context/tasks.toml` | 3 | Workflow | Low |
| P32 | CLI Polish | `plans/P32-cli-polish/tasks.toml` | 2 | CLI polish | Low |
| P33 | Model UX | `plans/P33-model-ux/tasks.toml` | 1 | Provider UX | Low |
| P34 | Verification Sweep | `plans/P34-verification-sweep/tasks.toml` | 4 | Verification | Low |
| e2e-smoke | E2E Smoke Tests | `plans/e2e-smoke/tasks.toml` | 2 | Side queue | Low |
| architecture-defi-critical-path | DeFi Critical Path | `plans/architecture-defi-critical-path/tasks.toml` | 3 | Architecture | Low |
| self-dev-ux | Self-Dev UX | `plans/self-dev-ux/tasks.toml` | 55 | Superseded | — |
| self-dev-extras | Self-Dev Extras | `plans/self-dev-extras/tasks.toml` | 11 | Superseded | — |

**Captured table totals**: 27 primary plans, 148 primary tasks, 153 tasks across the listed active queues (148 + 2 e2e + 3 defi).

---

## Plan-by-Plan Catalog

### Theme 1: Bug Fixes (P08–P10)

Highest-priority: fix concrete bugs that block already-built features.

---

#### P08 — Search Command Fix — `plans/P08-search-command-fix/tasks.toml`

**Root cause**: Captured analysis identifies 3 Perplexity client bugs: (1) wraps queries in `{"queries": [...]}` but the API expects `{"query": "..."}`, (2) dates formatted as ISO 8601 but API expects MM/DD/YYYY, (3) `SearchResult.content` missing `#[serde(alias = "snippet")]`. Search commands fail silently in that captured path.

| ID | Title | Tier | Crate |
|----|-------|------|-------|
| T1 | Rewrite PerplexitySearchClient to single-query API | focused | roko-agent |
| T2 | Replace date_range with native recency_filter | mechanical | roko-cli |
| T3 | Update search tests to match real API response format | focused | roko-agent |
| T4 | Add serde snippet alias to SearchResult.content | mechanical | roko-agent |

Anti-patterns: "Do NOT create a batch loop — the entire point is to eliminate the wrapper that caused the bug." "Do NOT use ISO 8601 date strings."

**IronClaw: Medium.** The `#[serde(alias = "snippet")]` pattern applies to any IronClaw external API integration where server field names differ from Rust struct field names.

---

#### P09 — Tool Alias Fix — `plans/P09-tool-alias-fix/tasks.toml`

**Root cause**: `parse_allowed_tools_csv()` stores tool names as-is. Claude CLI uses PascalCase (`Read`, `Write`, `Bash`); tool registry uses snake_case (`read_file`, `write_file`, `bash`). All non-Claude provider tool lookups fail.

| ID | Title | Tier | Crate |
|----|-------|------|-------|
| T1 | Resolve Claude aliases to canonical names in parse_allowed_tools_csv | focused | roko-agent |
| T2 | Add unit tests for alias resolution | mechanical | roko-agent |
| T3 | Audit other provider backends for same gap | focused | roko-agent |

T1: calls `canonical_of_claude(name)` from a 16-entry table in `roko_core::tool::aliases`. Return type changes from `Option<HashSet<&str>>` to `Option<HashSet<String>>`.

Anti-patterns: "Do NOT add new mappings in parse_allowed_tools_csv — use the existing aliases table." "Do NOT avoid the HashSet<String> change."

**IronClaw: High.** IronClaw's `src/tools/registry.rs` faces the same cross-channel naming problem. WASM channels, MCP servers, and the web gateway may reference tools by different naming conventions. A `canonical_tool_name()` with a known alias table is directly applicable.

---

#### P10 — Slash Command Flags — `plans/P10-slash-command-flags/tasks.toml`

**Root cause**: 3 ACP slash command bugs: `/plan-resume` passes `--resume` but CLI expects `--resume-plan`; `/plan-run` omits `--model`; `/develop` is not registered as ACP slash command.

| ID | Title | Tier | Crate |
|----|-------|------|-------|
| T1 | Fix /plan-resume to use --resume-plan | mechanical | roko-acp |
| T2 | Add --model passthrough to /plan-run | mechanical | roko-acp |
| T3 | Register /develop as ACP slash command | mechanical | roko-acp |
| T4 | Add /develop dispatch handler | mechanical | roko-acp |
| T5 | Add flag correctness tests | mechanical | roko-acp |

**IronClaw: Medium.** IronClaw's current web gateway and command-routing paths build argument vectors similarly. The flag-correctness test pattern (assert exact CLI args vector per command) is portable.

---

### Theme 2: Plan Runner Infrastructure (P11–P12)

---

#### P11 — Runner V2 Default — `plans/P11-runner-v2-default/tasks.toml`

**Captured status**: T1 and T2 were recorded as completed outside the queue (commit `9423998a7`); T3–T5 were listed as remaining in the captured plan notes.

**Root cause**: RunnerV2 (the working engine) was gated behind `--features legacy-runner-v2`. Without it, every `roko plan run` does nothing.

| ID | Title | Tier | Crate |
|----|-------|------|-------|
| T1 | Add legacy-runner-v2 to default features | mechanical | roko-cli |
| T2 | Make RunnerV2 the default PlanEngine variant | mechanical | roko-cli |
| T3 | Remove cfg gates from plan.rs and util.rs | focused | roko-cli |
| T4 | Remove cfg gates from test files | mechanical | roko-cli |
| T5 | Add TOML validation after plan generate | focused | roko-cli |

T5: validates required fields, unique IDs, and `depends_on` references exist within the same plan.

Anti-patterns: "Do NOT remove legacy-runner-v2 feature entirely yet." "Do NOT add TOML validator to the plan run path — only for generated plans."

**IronClaw: High.** The feature-flag-gated engine migration pattern applies to IronClaw's progressive tool disclosure (commit `ee838183d`). Post-generation TOML validation maps to IronClaw's skill manifest validation.

---

#### P12 — Runner Parallelism — `plans/P12-runner-parallelism/tasks.toml`

**Root cause**: `active_agent_tasks` and `agent_handles` are both `HashMap<String, T>` keyed by `plan_id`, so only one task per plan is tracked at a time even when `max_parallel = 5`.

| ID | Title | Tier | Crate |
|----|-------|------|-------|
| T1 | Read max_parallel from plan meta | focused | roko-cli |
| T2 | Change active_agent_tasks to track multiple per plan | integrative | roko-cli |
| T3 | Change agent_handles to track multiple per plan | integrative | roko-cli |
| T4 | Add per-task output tracking to RunState | focused | roko-cli |
| T5 | Dispatch multiple ready tasks per tick | integrative | roko-cli |

T2: `HashMap<String, String>` → `HashMap<String, HashSet<String>>`. T3: `HashMap<String, AgentHandle>` → `HashMap<String, HashMap<String, AgentHandle>>`. T4: adds `task_outputs: HashMap<String, String>` keyed by `"plan_id:task_id"`. T5: collects all ready tasks, sorts topologically, dispatches up to `max_concurrent_tasks`.

Anti-patterns: "Do NOT use a single counter for active tasks — use the HashSet." "Do NOT dispatch tasks whose depends_on is not complete."

**IronClaw: High.** IronClaw's `src/agent/` dispatcher handles concurrent job execution. Per-task output tracking keyed by compound string maps to `src/context/`. The topology-aware bounded dispatch algorithm (T5) is the most instructive reference for IronClaw's parallel agent orchestration.

---

### Theme 3: Resilience (P13–P15)

---

#### P13 — Rate Limit Retry — `plans/P13-rate-limit-retry/tasks.toml`

**Root cause**: Captured `OpenAiCompatLlmBackend` send paths map non-2xx responses to a generic network error, discarding the status code. The retry loop only retries provider-class errors, so HTTP 429 and 5xx are not retried in that path.

| ID | Title | Tier | Crate |
|----|-------|------|-------|
| T1 | Classify HTTP errors in send_turn (non-streaming) | mechanical | roko-agent |
| T2 | Classify HTTP errors in stream_turn (streaming) | mechanical | roko-agent |
| T3 | Classify HTTP errors in send_turn_streaming | mechanical | roko-agent |
| T4 | Add unit tests for classify_http_error | focused | roko-agent |

`classify_http_error(status, body)` returns: 429 → rate-limit provider error with optional `retry_after_ms`; 500–599 → server provider error with status; else → generic network/error class. `retry_after_ms` is extracted from JSON field `retry_after` when present.

Anti-patterns: "Do NOT create separate classify functions per send path — one helper." "Do NOT extract retry_after if status is not 429."

**IronClaw: High.** `crates/ironclaw_llm/` should be audited for the same status-code classification issue across provider send paths. The `classify_http_error` helper shape is directly portable.

---

#### P14 — Gate Rung Fix — `plans/P14-gate-rung-fix/tasks.toml`

**Root cause**: `selected_gate_steps()` increments `skipped_count` in *both* branches of the `enable_advanced_rungs` conditional — the `true` branch skips the rung it should activate.

```rust
// Broken:            if enable_advanced_rungs { skipped_count += 1; } else { skipped_count += 1; }
// Fixed:             if enable_advanced_rungs { gates.push(gate_for_rung(rung)); } else { skipped_count += 1; }
```

| ID | Title | Tier | Crate |
|----|-------|------|-------|
| T1 | Push concrete gate steps for advanced rungs when enabled | focused | roko-cli |
| T2 | Upgrade advanced rung activation log to info level | mechanical | roko-cli |
| T3 | Test verifying Complex pipeline includes all 7 gates | focused | roko-gate |

**IronClaw: Medium.** The "increment skipped in both branches" bug is a common copy-paste error in any boolean-gated pipeline, worth watching for in `src/evaluation/`.

---

#### P15 — Error Recovery Wiring — `plans/P15-error-recovery-wiring/tasks.toml`

**Root cause**: `classify_agent_crash()` and `recovery_hint()` exist but are never called from any failure handler. `RokoConfig::from_toml().unwrap_or_default()` silently discards parse errors.

| ID | Title | Tier | Crate |
|----|-------|------|-------|
| T1 | Wire classify_agent_crash into runner-v2 failure handler | mechanical | roko-cli |
| T2 | Wire classify_agent_crash into do_cmd.rs failure paths | mechanical | roko-cli |
| T3 | Warn on silent roko.toml parse fallback | mechanical | roko-cli |
| T4 | Add crash_class field to failure ledger entries | mechanical | roko-cli |
| T5 | Enrich agent exit messages with crash diagnosis | mechanical | roko-cli |

`AgentCrashClass` variants: `AuthFailure`, `RateLimitExhausted`, `ContextOverflow`, `ToolExecutionFailure`, `ProcessKilled`, `Unknown`. Each has `recovery_hint()`. T3 changes `unwrap_or_default()` to log a warning before falling back.

Anti-patterns: "Do NOT create new crash classification logic — call classify_agent_crash()." "Do NOT add crash_class to success ledger entries."

**IronClaw: High.** IronClaw's error handling would benefit from structured crash classification with recovery hints. Crash class aggregation in the run ledger aligns with IronClaw's LLM data retention principle.

---

### Theme 4: Safety (P16)

---

#### P16 — Safety Contracts — `plans/P16-safety-contracts/tasks.toml`

**Root cause**: `AgentContract` safety enforcement exists, but the captured dispatch path never loads or consults contracts. Contract-defined forbidden tool lists are ignored in that path.

| ID | Title | Tier | Crate |
|----|-------|------|-------|
| T1 | Add disallowed_tools to CliDispatchRequest | mechanical | roko-cli |
| T2 | Add disallowed_tools to AgentSpawnConfig | mechanical | roko-cli |
| T3 | Add forbidden_tool_names() helper to AgentContract | mechanical | roko-agent |
| T4 | Load contract for task role and set disallowed_tools | focused | roko-cli |
| T5 | Wire contract forbidden tools into bridge dispatch | mechanical | roko-cli |

Uses `ContractLoadMode::RestrictedFallback` (unknown roles get deny-all, not permissive). T3: `forbidden_tool_names()` iterates `governance_rules`, collects `GovernanceRule::ForbiddenTools` variants, deduplicates, returns sorted `Vec<String>`. Known gap: bridge `AgentDispatchRequest.tools` is an allowed-tools field, not denied; contract enforcement there is logged only.

Anti-patterns: "Do NOT use PermissiveFallback." "Do NOT add allowed_tools logic — only disallowed_tools."

**IronClaw: High.** Maps directly to IronClaw's tool attenuation in `src/skills/mod.rs` (`attenuate_tools()`). The disallowed_tools propagation through dispatch configs is the same pattern as IronClaw's skill trust model. The known bridge gap (allowed vs denied tool lists) applies to IronClaw's MCP and WASM channel dispatch paths.

---

### Theme 5: CLI Output and TUI (P17–P18)

Roko-specific UI polish; architectural patterns worth noting.

---

#### P17 — CLI Output Format — `plans/P17-cli-output-format/tasks.toml`

Creates a `CliOutput` wrapper struct that routes all progress/error messages through `output_format` functions with `quiet` suppression. Quiet flag is a single control point — not per-call-site guards. 6 tasks migrating ~25 `eprintln!` call sites.

**IronClaw: Low.** IronClaw's CLAUDE.md already has an equivalent (log level discipline for REPL/TUI). The `CliOutput` wrapper pattern applies if IronClaw adds a quiet mode.

---

#### P18 — TUI Agent Data — `plans/P18-tui-agent-data/tasks.toml`

Fixes 4 bugs preventing dashboard from showing live agent data: agent_id prefix match uses `:` but runner uses `/`; efficiency events not published; `MessageDelta` not forwarded; gate failure diagnoses not published. The `DashboardEvent` enum is the clean separation point between runner and TUI.

**IronClaw: Low.** TUI-specific. The structured event pattern (runner emits typed events, TUI consumes) mirrors IronClaw's `src/observability/` observer pattern.

---

### Theme 6: ACP / Editor Integration (P19–P22, P25, P29)

ACP (Agent Client Protocol) is Roko's equivalent of IronClaw's web gateway channel — the primary interactive interface for developers (Zed editor).

---

#### P19 — Cascade Router ACP — `plans/P19-cascade-router-acp/tasks.toml`

**Root cause**: `route_with_cfactor()` is never called in the ACP dispatch path. Model selection falls back to a hardcoded default instead of the router's recommendation even when the cascade router is loaded and initialized.

6 tasks: add `selected_model` field to `AcpSessionState`, call `route_with_cfactor()` during session setup, use selected model in dispatch, add fallback handling, add observation record, add tests.

**IronClaw: Medium.** IronClaw's `crates/ironclaw_llm/` would benefit from an adaptive model router. This plan demonstrates how to wire an existing adaptive router into a dispatch path.

---

#### P20 — Zero Config — `plans/P20-zero-config/tasks.toml`

**Root cause**: `preflight_provider_for_model()` only checks TOML entries. If `roko.toml` is absent, all model lookups fail even when API keys are in env vars.

5 tasks (max_parallel=2): add builtin model registry (static map of slugs to providers), fall back to registry when TOML has no entry, add env-var detection for known provider keys, wire detection into preflight, add tests.

**IronClaw: Medium.** The builtin model registry pattern (static slug → provider map) is directly applicable when IronClaw auto-detects the provider for a given model string. IronClaw's env var detection in `.env` already addresses this partially.

---

#### P21 — ACP Streaming — `plans/P21-acp-streaming/tasks.toml`

**Root cause**: `run_slash_command()` accumulates all subprocess stdout into a buffer, emitting a single `TokenChunk` at exit. Users see no output until the command finishes.

5 tasks: read stdout line-by-line, send each line as a `TokenChunk` immediately, handle final newline-stripped line, preserve progress handling, test that multiple chunks are emitted.

**IronClaw: Medium.** IronClaw's web gateway uses WebSocket streaming (`src/channels/web/`). Converting buffered subprocess output to streaming events applies to IronClaw's shell tool and MCP subprocess output streaming.

---

#### P22 — ACP Tool Permission — `plans/P22-acp-tool-permission/tasks.toml`

**Root cause**: 3 ACP code paths use `ToolContext::testing()` in production, which sets `network: false` and bypasses all safety checks.

5 tasks: audit all `ToolContext::testing()` call sites, replace each with explicit `ToolContext::new(capabilities)` construction, add tests verifying production call sites use non-testing contexts.

Anti-pattern: "Do NOT create a `ToolContext::for_acp()` convenience constructor — each call site has different capability requirements."

**IronClaw: High.** IronClaw's `src/tools/dispatch.rs` warrants the same audit. If any production path uses a test `ToolContext`, network behavior and safety checks can diverge from intent. Audit web gateway and WASM channel dispatch paths. Explicit capability construction should be enforced via code review.

---

#### P25 — MCP-ACP Passthrough — `plans/P25-mcp-acp-passthrough/tasks.toml`

**Root cause**: `roko_core::AgentConfig` lacks `mcp_config`. MCP servers configured in `roko.toml` are never passed to agents launched via ACP.

4 tasks: add `mcp_config: Option<McpConfig>` to `AgentConfig`, serialize through ACP wire protocol, deserialize on receiving end, pass to Claude CLI `--mcp-config` flag.

**IronClaw: High.** IronClaw's MCP integration in `src/tools/mcp/` has the same config passthrough need. Verify that `McpConfig` reaches all channel dispatch paths — web gateway handlers and WASM channel invocations. This is a concrete blueprint for that audit.

---

#### P29 — Develop Command Wire — `plans/P29-develop-command-wire/tasks.toml`

Registers `/develop` as an ACP slash command. **Overlap**: P10-T3/T4 covers the same registration; if P10 runs first (it does), P29 becomes a verification step.

**IronClaw: Low.** Roko-specific. The general concept (ensure CLI subcommands are reachable from interactive channels) is relevant for IronClaw's web gateway, but this plan is not portable.

---

### Theme 7: PRD and Workspace Tooling (P23–P24)

---

#### P23 — PRD Pipeline Fix — `plans/P23-prd-pipeline-fix/tasks.toml`

**Root cause**: PRD draft agent has `allowed_tools: Some("none")`. PRDs are generated without reading the codebase.

6 tasks: change allowed tools to `Some("Read,Grep,Glob")`, add system prompt instruction to read key files, add codebase summary step, add validation that PRD references actual file paths, add tests.

Key pattern: **stage-gated tool access** — different pipeline stages get different tool access (draft: read-only, implementation: write).

**IronClaw: Medium.** IronClaw's skill attenuation (`attenuate_tools()`) implements least-privilege at the skill level. This plan applies the same principle at the workflow stage level. IronClaw's job state machine could carry per-stage tool access policies.

---

#### P24 — Workspace Paths — `plans/P24-workspace-paths/tasks.toml`

**Root cause**: `plan.rs` prefers `./plans/`; `main.rs` prefers `.roko/plans/`. Plan runner fails to locate plans when invoked from `main.rs`.

4 tasks: align `main.rs` to use `./plans/`, add fallback to `.roko/plans/` for backward compatibility, document resolution order, add test.

**IronClaw: Low.** Path resolution consistency is universal (IronClaw's `src/bootstrap.rs`), but this specific fix is Roko-internal.

---

### Theme 8: Learning (P26)

---

#### P26 — HDC Similarity Lookup — `plans/P26-hdc-similarity-lookup/tasks.toml`

**Root cause**: `EpisodeLogger` stores HDC fingerprints for each agent session but `query_similar_episodes()` was never implemented. Stored fingerprints cannot be queried.

| ID | Title | Tier | Crate |
|----|-------|------|-------|
| T1 | Add query_similar_episodes() to EpisodeLogger | integrative | roko-learn |
| T2 | Decode stored HDC fingerprints from binary | focused | roko-learn |
| T3 | Rank episodes by Hamming distance to query | focused | roko-learn |
| T4 | Add tests with synthetic fingerprints | focused | roko-learn |

T1: `query_similar_episodes(query_fp: &HdcFingerprint, limit: usize) -> Vec<(EpisodeId, f32)>`. T3: similarity = `1.0 - (hamming_distance / dimension)`.

**IronClaw: Medium.** IronClaw's workspace uses hybrid search (FTS + vector via RRF). HDC is faster (bit ops vs float dot products), lower memory (binary vs float32), and requires no embedding model call. An HDC-based session similarity index in `src/workspace/` could complement vector search. See [`../core-concepts/hyperdimensional-computing/`](../core-concepts/hyperdimensional-computing/) for the full HDC design.

---

### Theme 9: Provider and Model UX (P27–P28, P33)

---

#### P27 — Provider Error UX — `plans/P27-provider-error-ux/tasks.toml`

**Root cause**: `roko doctor` always runs `check_anthropic_api_key()` even when the user only configured OpenAI or Gemini.

4 tasks: read configured providers list, make `check_anthropic_api_key()` conditional on Anthropic being configured, add equivalent conditional checks for OpenAI and Gemini, add tests.

**IronClaw: Medium.** IronClaw's `src/cli/doctor.rs` should validate API keys only for configured providers: NEAR AI, OpenAI, Anthropic, Bedrock (AWS credentials), Tinfoil. A doctor that warns about Anthropic keys for a NEAR AI-only deployment is noise.

---

#### P28 — Image Support — `plans/P28-image-support/tasks.toml`

**Root cause**: ACP session `image` capability is hardcoded to `false`. The `supports_vision` flag exists in the model registry but is never consulted.

5 tasks: add `supports_vision: bool` to `ModelProfile`, set it for known vision models in builtin registry, read it during ACP session setup, set `capabilities.image = model_profile.supports_vision`, add tests.

**IronClaw: Medium.** IronClaw's `src/channels/channel.rs` could expose/hide image upload based on active provider model capabilities via `crates/ironclaw_llm/` model metadata.

---

#### P33 — Model UX — `plans/P33-model-ux/tasks.toml`

1 task: `max_tokens` auto-recovery — detect context overflow errors and retry with reduced `max_tokens`. Already implemented: Jaro-Winkler fuzzy model name suggestions, setup wizard, `/models` ACP command.

**IronClaw: Low.** The max_tokens auto-recovery pattern is a minor optimization for `crates/ironclaw_llm/`.

---

### Theme 10: Onboarding (P30)

---

#### P30 — Onboarding Doctor — `plans/P30-onboarding-doctor/tasks.toml`

Adds OpenAI and Gemini API key checks to `roko doctor` in the captured state where only Anthropic was checked. Uses `Warn` severity (not `Fail`) since no single provider is mandatory. 4 tasks, max_parallel=2.

**IronClaw: Medium.** IronClaw's `src/cli/doctor.rs` should validate keys for every configured provider. Severity ladder: Info / Warn / Fail, with Warn for optional providers.

---

### Theme 11: Note Workflow and CLI Polish (P31–P32)

**P31 — Note and Context** — `plans/P31-note-and-context/tasks.toml`: Adds `roko plan new --prompt "..."` and plan generation from accumulated `roko note` entries. 3 tasks. **IronClaw: Low.**

**P32 — CLI Polish** — `plans/P32-cli-polish/tasks.toml`: Verbose flag guard for debug config output; symbol consistency in `ContentBlock` enum. 2 independent tasks. **IronClaw: Low.**

---

### Theme 12: Verification Sweep (P34)

---

#### P34 — Verification Sweep — `plans/P34-verification-sweep/tasks.toml`

Final verification pass after P08–P33: T1 `cargo check`, T2 `cargo clippy` (zero warnings), T3 `cargo test`, T4 `cargo build --release`. Each task has a custom `fail_msg` explaining the regression type.

**IronClaw: Low.** IronClaw already implements this pattern in CI. The value is making it the explicit last step in the plan queue, gating any release.

---

### Standalone Plans

---

#### e2e-smoke — E2E Smoke Tests — `plans/e2e-smoke/tasks.toml`

2 trivial tasks designed to validate the plan runner itself (not the implementation). Uses `skip_enrichment = true` (the only plan in the corpus to do so). S01: add `#[must_use]` to `generate_share_token()`. S02: add unit test for it.

**IronClaw: Low.** IronClaw's plan runner (see `../implementation/09-plan-runner-readiness.md`) should have an equivalent smoke test plan for initial validation.

---

#### architecture-defi-critical-path — DeFi Critical Path — `plans/architecture-defi-critical-path/tasks.toml`

3 tasks building chain-side primitives (chain registry client, registry query routes, integration verification). Requires `architecture-core-queue Q14` (not in captured plan set).

Notable: uses `acceptance_contract` blocks with `parity_ledger` linking formal requirement IDs to source specification references and evidence test file locations — the most sophisticated verification structure in the entire corpus.

**IronClaw: Low.** Blockchain-specific. However, the `parity_ledger` format is worth adopting for high-stakes IronClaw architectural changes.

---

### Superseded Plans

**self-dev-ux** (55 tasks) — `plans/self-dev-ux/tasks.toml`: Original monolithic self-development UX plan. Originally `max_parallel = 20`; revised downward in P08–P34 after the feedback audit found high parallelism caused dependency conflicts and harder-to-diagnose failures. Tasks completed before supersession: H07–H09, L05, L09–L10, M01, M10–M11, V01.

**self-dev-extras** (11 tasks) — `plans/self-dev-extras/tasks.toml`: Additional tasks consolidated into P08–P34.

---

## Cross-Plan Patterns

### High-Value Transferable Patterns

| Pattern | Plans | IronClaw Target |
|---------|-------|----------------|
| HTTP error classification by status code | P13 | `crates/ironclaw_llm/` provider send paths |
| Tool name alias canonicalization | P09 | `src/tools/registry.rs` |
| Per-role forbidden tool enforcement | P16 | `src/skills/mod.rs` attenuate_tools |
| MCP config passthrough to all dispatch paths | P25 | `src/tools/mcp/` |
| Production ToolContext audit (not testing) | P22 | `src/tools/dispatch.rs` |
| Stage-gated tool access per pipeline stage | P23 | job state machine tool policies |
| Structured crash classification + recovery hints | P15 | `src/agent/` error handling |
| Feature-flag-gated engine migration | P11 | progressive tool disclosure |

### Anti-Pattern Taxonomy

The plans encode past mistakes explicitly. Common anti-patterns across the corpus:

1. **Batch API assumption**: wrapping single-item calls as batch calls before verifying the API supports batches (P08)
2. **Naming convention mismatch**: different layers using different name formats without a canonical resolver (P09)
3. **Test context in production**: using `::testing()` constructors in production paths (P22)
4. **Convenience constructors bypassing requirements**: single constructors that silently satisfy all capability fields (P22)
5. **Config fallback that swallows errors**: `unwrap_or_default()` on parse results without logging (P15)
6. **Both-branch skip bug**: incrementing skipped count in both branches of a boolean-gated pipeline (P14)
7. **Config not flowing to all dispatch paths**: struct fields added to core types but not plumbed through all layers (P25)

### Plan Format Evolution

From self-dev-ux to P08–P34:
- `max_parallel` reduced from 20 → 1 (mostly), some plans 2
- `anti_patterns` field added per task (explicit "Do NOT" encoding)
- `model_hint` made tier-driven rather than manually specified
- `acceptance_contract` blocks added for architecture-tier plans
- Explicit `depends_on` chains replacing implicit ordering
