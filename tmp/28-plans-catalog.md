# Roko Implementation Plans -- Comprehensive Catalog

## Overview

This document catalogs every implementation plan found in the roko project's
`plans/` directory at `/Users/will/dev/nunchi/roko/roko/plans/`. Roko is a
Rust-based AI coding assistant (~177K LOC, 18 crates) with a CLI, ACP (Agent
Communication Protocol) integration for the Zed editor, plan-driven development
workflows, agent orchestration, and an adaptive learning subsystem.

These plans are not ordinary task lists. They are **machine-executable
specifications** that roko uses to develop *itself*. Each plan is a TOML file
(`tasks.toml`) containing a `[meta]` header and one or more `[[task]]` entries
with:

- **Tier** -- mechanical (trivial, haiku-class), focused (single-concern,
  sonnet-class), integrative (cross-module, opus-class), or architectural
  (system-level design).
- **Model hint** -- which LLM should execute the task (e.g.,
  `claude-haiku-4-5`, `claude-sonnet-4-20250514`, `claude-opus-4-6`).
- **Max LoC budget** -- hard cap on lines of code the implementing agent may
  produce.
- **File targets** -- exact files to modify.
- **Dependency chains** -- `depends_on` for ordering within a plan.
- **Read-context** -- files + line ranges + rationale the agent must read before
  implementing.
- **Symbols** -- function/struct names the agent needs to understand.
- **Anti-patterns** -- explicit "do NOT" instructions to prevent common
  mistakes.
- **Multi-phase verification** -- structural grep checks, compile checks, test
  runs, each with custom `fail_msg`.
- **Acceptance contracts** (in some plans) -- formal parity ledgers linking
  requirements to evidence files.

The plans are organized into three queues: a primary self-development queue
(P08--P34), standalone/side queues, and superseded source plans that were
consolidated into the primary queue.

**Cross-reference**: Related IronClaw documents that provide additional context
on roko's architecture include documents on the orchestrator/swarm system, agent
coordination, the learning subsystem, and individual crate deep-dives in the
`/Users/will/dev/near/ironclaw/tmp/` directory.

---

## Table of Contents

1. [Execution Order and Queue Structure](#1-execution-order-and-queue-structure)
2. [Summary Table](#2-summary-table)
3. [Plan-by-Plan Catalog](#3-plan-by-plan-catalog)
   - [Theme 1: Bug Fixes and Correctness (P08--P10)](#theme-1-bug-fixes-and-correctness-p08--p10)
   - [Theme 2: Plan Runner Infrastructure (P11--P12)](#theme-2-plan-runner-infrastructure-p11--p12)
   - [Theme 3: Resilience and Error Handling (P13--P15)](#theme-3-resilience-and-error-handling-p13--p15)
   - [Theme 4: Safety and Quality Gates (P16)](#theme-4-safety-and-quality-gates-p16)
   - [Theme 5: CLI Output and TUI (P17--P18)](#theme-5-cli-output-and-tui-p17--p18)
   - [Theme 6: ACP / Editor Integration (P19--P22, P25, P29)](#theme-6-acp--editor-integration-p19--p22-p25-p29)
   - [Theme 7: PRD and Workspace Tooling (P23--P24)](#theme-7-prd-and-workspace-tooling-p23--p24)
   - [Theme 8: Learning and Intelligence (P26)](#theme-8-learning-and-intelligence-p26)
   - [Theme 9: Provider and Model UX (P20, P27--P28, P33)](#theme-9-provider-and-model-ux-p20-p27--p28-p33)
   - [Theme 10: Onboarding and Doctor (P30)](#theme-10-onboarding-and-doctor-p30)
   - [Theme 11: Note Workflow and CLI Polish (P31--P32)](#theme-11-note-workflow-and-cli-polish-p31--p32)
   - [Theme 12: Verification Sweep (P34)](#theme-12-verification-sweep-p34)
   - [Standalone Plans](#standalone-plans)
   - [Superseded Source Plans](#superseded-source-plans)
4. [Cross-Plan Patterns and Architecture](#4-cross-plan-patterns-and-architecture)
5. [IronClaw Relevance Assessment](#5-ironclaw-relevance-assessment)
6. [Lessons Learned](#6-lessons-learned)

---

## 1. Execution Order and Queue Structure

**Source file:** `plans/_meta/IMPLEMENTATION_ORDER.md`

The plans are organized into distinct queues with strict execution ordering.

### Already Complete (not in filesystem)

These plans were completed and their directories subsequently removed from
tracking (per git commit `f6938565b`):

1. `W01-wire-system-prompts` -- system prompt composition (completed)
2. `P06-process-management` -- process lifecycle management (completed)
3. `P07-autofix-retry` -- autofix retry mechanism (completed; commit
   `97b15c085`)

### Primary Queue (P08--P34)

Run sequentially, one plan at a time. All 27 plans currently have
`status = "ready"` in their TOML -- none have been executed from the queue
runner yet, though some tasks from the superseded `self-dev-ux` plan (H07--H09,
L05, L09--L10, M01, M10--M11, V01) were completed individually before the
consolidation. The changes from `self-dev-ux` that overlap with P08--P34 tasks
are noted where relevant. Additionally, commit `9423998a7` ("fix: make
runner-v2 the default plan engine") implements the core of P11 outside the plan
runner.

Execution order: P08, P09, P10, P11, P12, P13, P14, P15, P16, P17, P18, P19,
P20, P21, P22, P23, P24, P25, P26, P27, P28, P29, P30, P31, P32, P33, P34.

### Separate Queues

- `architecture-core-queue` -- the larger architecture queue (referenced but
  directory not present in filesystem; may have been removed after the plans/
  gitignore commit).
- `architecture-defi-critical-path` -- depends on architecture-core Q14; 3
  tasks present in filesystem.
- `e2e-smoke` -- standalone side queue; 2 mechanical tasks for basic validation.
- `dry-run-flag`, `live-demo-phase1`, `live-demo-phase2` -- referenced in
  IMPLEMENTATION_ORDER but directories not present in filesystem.

### Superseded (do not run)

- `self-dev-ux` -- 55 tasks, status `superseded`, superseded by "P08-P34 plans
  (consolidated from feedback audit 2026-05-08)".
- `self-dev-extras` -- 11 tasks, status `superseded`, same supersession note.

---

## 2. Summary Table

| Plan | Title | Tasks | Status | Theme | Max Parallel | IronClaw Relevance |
|------|-------|-------|--------|-------|--------------|-------------------|
| P08 | Search Command Fix | 4 | Ready (not started) | Bug fixes | 1 | Medium |
| P09 | Tool Alias Fix | 3 | Ready (not started) | Bug fixes | 1 | High |
| P10 | Slash Command Flags | 5 | Ready (not started) | Bug fixes | 1 | Medium |
| P11 | Runner V2 Default | 5 | Ready (partially implemented outside queue) | Runner infra | 1 | High |
| P12 | Runner Parallelism | 5 | Ready (not started) | Runner infra | 1 | High |
| P13 | Rate Limit Retry | 4 | Ready (not started) | Resilience | 1 | High |
| P14 | Gate Rung Fix | 3 | Ready (not started) | Quality gates | 1 | Medium |
| P15 | Error Recovery Wiring | 5 | Ready (not started) | Resilience | 1 | High |
| P16 | Safety Contracts | 5 | Ready (not started) | Safety | 1 | High |
| P17 | CLI Output Format | 6 | Ready (not started) | CLI/TUI | 1 | Low |
| P18 | TUI Agent Data | 5 | Ready (not started) | CLI/TUI | 1 | Low |
| P19 | Cascade Router ACP | 6 | Ready (not started) | ACP/Editor | 1 | Medium |
| P20 | Zero Config | 5 | Ready (not started) | Provider UX | 2 | Medium |
| P21 | ACP Streaming | 5 | Ready (not started) | ACP/Editor | 1 | Medium |
| P22 | ACP Tool Permission | 5 | Ready (not started) | ACP/Editor | 1 | High |
| P23 | PRD Pipeline Fix | 6 | Ready (not started) | PRD/Workspace | 1 | Medium |
| P24 | Workspace Paths | 4 | Ready (not started) | PRD/Workspace | 1 | Low |
| P25 | MCP-ACP Passthrough | 4 | Ready (not started) | ACP/Editor | 1 | High |
| P26 | HDC Similarity Lookup | 4 | Ready (not started) | Learning | 1 | Medium |
| P27 | Provider Error UX | 4 | Ready (not started) | Provider UX | 1 | Medium |
| P28 | Image Support | 5 | Ready (not started) | Provider UX | 1 | Medium |
| P29 | Develop Command Wire | 3 | Ready (not started) | ACP/Editor | 1 | Low |
| P30 | Onboarding Doctor | 4 | Ready (not started) | Onboarding | 2 | Medium |
| P31 | Note and Context | 3 | Ready (not started) | Workflow | 1 | Low |
| P32 | CLI Polish | 2 | Ready (not started) | CLI polish | 2 | Low |
| P33 | Model UX | 1 | Ready (not started) | Provider UX | 1 | Low |
| P34 | Verification Sweep | 4 | Ready (not started) | Verification | 1 | Low |
| e2e-smoke | E2E Smoke Tests | 2 | Ready | Side queue | 1 | Low |
| architecture-defi-critical-path | DeFi Critical Path | 3 | Ready | Architecture | 1 | Low |
| self-dev-ux | Self-Dev UX | 55 | Superseded | (consolidated) | 20 | -- |
| self-dev-extras | Self-Dev Extras | 11 | Superseded | (consolidated) | 20 | -- |

**Aggregate statistics:**
- Total plans in filesystem: 31 (27 primary + 2 standalone + 2 superseded)
- Total tasks across all plans: 148 (primary queue) + 2 (e2e) + 3 (defi) + 66
  (superseded) = 219
- Estimated total minutes (where specified): e2e-smoke (5), P31 (90), P32 (20),
  P33 (30), P34 (120), self-dev-extras (180), self-dev-ux (720),
  architecture-defi (360)

---

## 3. Plan-by-Plan Catalog

### Theme 1: Bug Fixes and Correctness (P08--P10)

These plans fix concrete bugs that block basic functionality. They are the
highest-priority items in the queue because they prevent existing features from
working correctly.

---

#### P08 -- Search Command Fix

**File:** `plans/P08-search-command-fix/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 4 | **Max parallel:** 1

**What it fixes:** The Perplexity search client wraps queries in
`{"queries": [...]}` but Perplexity has no batch endpoint -- it expects flat
`{"query": "..."}`. Every search command fails silently. Additionally, dates
are formatted as ISO 8601 but Perplexity expects `MM/DD/YYYY`, and
`SearchResult.content` does not alias the `snippet` field name.

**Tasks:**

| ID | Title | Tier | LoC | Crate |
|----|-------|------|-----|-------|
| T1 | Rewrite PerplexitySearchClient to single-query API format | focused | 60 | roko-agent |
| T2 | Replace date_range with native recency_filter in search command | mechanical | 20 | roko-cli |
| T3 | Update search tests to match real Perplexity API response format | focused | 80 | roko-agent |
| T4 | Add serde snippet alias to SearchResult.content field | mechanical | 15 | roko-agent |

**Key details:**
- T1 adds `recency_filter: Option<String>` to `SearchQuery` (serde-skipped),
  rewrites `search()` to single-query POST, rewrites `search_batch()` as
  sequential loop. Removes `MAX_BATCH_SIZE` and `TooManyQueries`.
- T3 rewrites all mock responses from nested batch format to flat; removes
  `canned_batch` helper.
- T4 adds `#[serde(alias = "snippet")]` to `SearchResult.content`.

**Dependency graph:** T1 (root) -> T2, T3; T4 is independent.

**IronClaw relevance:** Medium. The serde alias trick and batch-vs-single API
adaptation pattern are universally applicable.

---

#### P09 -- Tool Alias Fix

**File:** `plans/P09-tool-alias-fix/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 3 | **Max parallel:** 1

**What it fixes:** `parse_allowed_tools_csv()` collects tool names as-is.
Callers pass Claude CLI PascalCase (`Read`, `Write`, `Edit`) but the tool
registry uses snake_case (`read_file`, `write_file`, `edit_file`). Zero tools
match on non-Claude providers, breaking research/analyze commands.

**Tasks:**

| ID | Title | Tier | LoC | Crate |
|----|-------|------|-----|-------|
| T1 | Resolve Claude aliases to canonical names in parse_allowed_tools_csv | focused | 25 | roko-agent |
| T2 | Add unit tests for alias resolution in tool CSV parser | mechanical | 50 | roko-agent |
| T3 | Audit other provider backends for same alias resolution gap | focused | 20 | roko-agent |

**Key details:**
- Uses `canonical_of_claude()` from `roko_core::tool::aliases` (a 16-entry
  alias table mapping PascalCase to snake_case).
- Return type changes from `Option<HashSet<&str>>` to `Option<HashSet<String>>`.
- T3 verifies gemini.rs/anthropic_api.rs delegate to `tool_registry_for_options`.

**IronClaw relevance:** High. IronClaw's `ToolRegistry` faces the same
cross-channel naming canonicalization problem.

---

#### P10 -- Slash Command Flags

**File:** `plans/P10-slash-command-flags/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 5 | **Max parallel:** 1

**What it fixes:** Three ACP slash command issues:
1. `/plan-resume` passes `--resume` but CLI expects `--resume-plan`.
2. `/plan-run` omits `--model`, ignoring user's model selection.
3. `/develop` is not registered as a slash command.

**Tasks:**

| ID | Title | Tier | LoC | Crate |
|----|-------|------|-----|-------|
| T1 | Fix /plan-resume to use --resume-plan flag | mechanical | 5 | roko-acp |
| T2 | Add --model flag passthrough to /plan-run | mechanical | 10 | roko-acp |
| T3 | Register /develop as ACP slash command in session.rs | mechanical | 15 | roko-acp |
| T4 | Add /develop dispatch handler to bridge_events.rs | mechanical | 20 | roko-acp |
| T5 | Add test for flag correctness | mechanical | 30 | roko-acp |

**Note:** T3/T4 overlap with P29 (`develop-command-wire`). P10 covers both
registration and dispatch; P29 covers the same registration plus wiring.

**IronClaw relevance:** Medium. The slash command dispatch pattern (match block
building CLI arg vectors) is relevant to IronClaw's web gateway command routing.

---

### Theme 2: Plan Runner Infrastructure (P11--P12)

These plans make the plan execution engine production-ready by enabling the
working runner by default and adding parallel task dispatch.

---

#### P11 -- Runner V2 Default

**File:** `plans/P11-runner-v2-default/tasks.toml`
**Status:** Ready (partially implemented) | **Tasks:** 5 | **Max parallel:** 1

**Implementation status:** Commit `9423998a7` ("fix: make runner-v2 the default
plan engine") implements the core intent of this plan (T1--T2) outside the plan
runner. T3--T5 (removing cfg gates from production code and tests, adding TOML
validation) remain unimplemented.

**What it does:** Activates the `legacy-runner-v2` feature by default. Without
this, `roko plan run` routes to the Graph Engine stub which does nothing.

**Tasks:**

| ID | Title | Tier | LoC | Crate |
|----|-------|------|-----|-------|
| T1 | Add legacy-runner-v2 to default features | mechanical | 5 | roko-cli |
| T2 | Make RunnerV2 the default PlanEngine variant | mechanical | 15 | roko-cli |
| T3 | Remove cfg gates from plan.rs and util.rs | focused | 30 | roko-cli |
| T4 | Remove cfg gates from test files | mechanical | 20 | roko-cli |
| T5 | Add TOML validation after plan generate | focused | 35 | roko-cli |

**IronClaw relevance:** High. The feature-flag-gated engine migration pattern
and post-generation TOML validation are directly applicable to IronClaw's
progressive tool disclosure (flag-gated, default off).

---

#### P12 -- Runner Parallelism

**File:** `plans/P12-runner-parallelism/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 5 | **Max parallel:** 1

**What it does:** Enables parallel task dispatch within the plan runner. Currently
limited to one agent per plan even when `max_parallel > 1` in tasks.toml.

**Tasks:**

| ID | Title | Tier | LoC | Crate |
|----|-------|------|-----|-------|
| T1 | Read max_parallel from plan meta into max_concurrent_tasks | focused | 20 | roko-cli |
| T2 | Change active_agent_tasks to track multiple per plan | integrative | 40 | roko-cli |
| T3 | Change agent_handles to track multiple per plan | integrative | 50 | roko-cli |
| T4 | Add per-task output tracking to RunState | focused | 30 | roko-cli |
| T5 | Dispatch multiple ready tasks per tick | integrative | 40 | roko-cli |

**Key details:**
- T2 changes `active_agent_tasks` from `HashMap<String, String>` to
  `HashMap<String, HashSet<String>>` (plan_id -> set of task_ids).
- T3 changes `agent_handles` from `HashMap<String, AgentHandle>` to
  `HashMap<String, HashMap<String, AgentHandle>>` (plan_id -> task_id -> handle).
- T4 adds `task_outputs: HashMap<String, String>` keyed by `"plan_id:task_id"`.
- T5 synthesizes additional `SpawnAgent` actions within a single tick, bounded
  by `max_concurrent_tasks`.

**IronClaw relevance:** High. IronClaw's agent/dispatcher system handles
concurrent tool execution and job management. The per-task output tracking and
concurrency-limited dispatch patterns are directly transferable.

---

### Theme 3: Resilience and Error Handling (P13--P15)

These plans improve how the system handles transient failures, rate limits,
and agent crashes.

---

#### P13 -- Rate Limit Retry

**File:** `plans/P13-rate-limit-retry/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 4 | **Max parallel:** 1

**What it fixes:** All three HTTP send paths (`send_turn`, `stream_turn`,
`send_turn_streaming`) in `OpenAiCompatLlmBackend` map non-2xx responses to
`LlmError::Network`, discarding the status code. The retry loop in `ToolLoop`
only retries on `LlmError::Provider` -- so 429/5xx errors are never retried.

**Tasks:**

| ID | Title | Tier | LoC | Crate |
|----|-------|------|-----|-------|
| T1 | Classify HTTP errors in send_turn (non-streaming) | mechanical | 25 | roko-agent |
| T2 | Classify HTTP errors in stream_turn (streaming) | mechanical | 25 | roko-agent |
| T3 | Classify HTTP errors in send_turn_streaming | mechanical | 15 | roko-agent |
| T4 | Add unit test for classify_http_error | focused | 40 | roko-agent |

**Key details:**
- Introduces `classify_http_error(HttpPostError) -> LlmError` helper: 429 ->
  `ProviderError::RateLimit`, 500-599 -> `ProviderError::ServerError`, else ->
  `LlmError::Network`.
- Extracts `retry_after` from 429 response JSON when present.

**IronClaw relevance:** High. IronClaw's `ironclaw_llm` crate has the same
multi-provider HTTP error classification need. The `classify_http_error`
pattern is directly portable.

---

#### P14 -- Gate Rung Fix

**File:** `plans/P14-gate-rung-fix/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 3 | **Max parallel:** 1

**What it fixes:** In `selected_gate_steps()`, the match arm for
Symbol/PropertyTest/Integration rungs increments `skipped_count` in *both*
branches -- the `enable_advanced_rungs = true` branch skips instead of
constructing gate instances.

**Tasks:**

| ID | Title | Tier | LoC | Crate |
|----|-------|------|-----|-------|
| T1 | Push concrete gate steps for advanced rungs when enabled | focused | 40 | roko-cli |
| T2 | Upgrade advanced rung activation log to info level | mechanical | 10 | roko-cli |
| T3 | Add test verifying Complex pipeline includes all 7 rung gates | focused | 30 | roko-gate |

**IronClaw relevance:** Medium. The gate pipeline pattern (ordered verification
steps with skip/activate logic) has parallels in IronClaw's evaluation system.

---

#### P15 -- Error Recovery Wiring

**File:** `plans/P15-error-recovery-wiring/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 5 | **Max parallel:** 1

**What it does:** Wires `classify_agent_crash()` and `recovery_hint()` into
agent failure paths so users get actionable recovery guidance instead of
generic "agent process exited unsuccessfully: exit_code=N" messages.

**Tasks:**

| ID | Title | Tier | LoC | Crate |
|----|-------|------|-----|-------|
| T1 | Wire classify_agent_crash into runner-v2 failure handler | mechanical | 25 | roko-cli |
| T2 | Wire classify_agent_crash into do_cmd.rs failure paths | mechanical | 20 | roko-cli |
| T3 | Warn on silent roko.toml parse fallback | mechanical | 20 | roko-cli |
| T4 | Add crash_class field to failure ledger entries | mechanical | 15 | roko-cli |
| T5 | Enrich agent exit and output sink messages with crash diagnosis | mechanical | 15 | roko-cli |

**Key details:**
- `AgentCrashClass` has variants for auth failures, rate limits, context
  overflow, etc., each with a human-readable recovery hint.
- T3 addresses a separate issue: `RokoConfig::from_toml().unwrap_or_default()`
  silently discards parse errors.
- T4 adds `crash_class` to the run ledger JSON for learning system aggregation.

**IronClaw relevance:** High. IronClaw's error handling would benefit from
structured crash classification with recovery hints. The ledger-based crash
class aggregation enables systematic failure pattern analysis.

---

### Theme 4: Safety and Quality Gates (P16)

---

#### P16 -- Safety Contracts

**File:** `plans/P16-safety-contracts/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 5 | **Max parallel:** 1

**What it does:** Wires `AgentContract` safety enforcement into the runner
dispatch path. Currently contracts exist but fall back to permissive defaults.
This plan loads per-role contracts and propagates forbidden tool lists to CLI
`--disallowed-tools` args.

**Tasks:**

| ID | Title | Tier | LoC | Crate |
|----|-------|------|-----|-------|
| T1 | Add disallowed_tools to CliDispatchRequest | mechanical | 25 | roko-cli |
| T2 | Add disallowed_tools to AgentSpawnConfig | mechanical | 15 | roko-cli |
| T3 | Add forbidden_tool_names() helper to AgentContract | mechanical | 20 | roko-agent |
| T4 | Load contract for task role and set disallowed_tools | focused | 25 | roko-cli |
| T5 | Wire contract forbidden tools into bridge dispatch path | mechanical | 20 | roko-cli |

**Key details:**
- Uses `ContractLoadMode::RestrictedFallback` -- unknown roles get deny-all,
  not permissive.
- T5 notes a known gap: bridge `AgentDispatchRequest.tools` is for allowed
  tools, not denied. Contract enforcement is logged but not enforceable on the
  bridge path.
- `forbidden_tool_names()` deduplicates and sorts across multiple
  `GovernanceRule::ForbiddenTools` entries.

**IronClaw relevance:** High. IronClaw's safety layer (`ironclaw_safety`) and
tool dispatch pipeline have the same per-role contract enforcement pattern.
The `disallowed_tools` propagation through dispatch configs maps directly to
IronClaw's tool attenuation system.

---

### Theme 5: CLI Output and TUI (P17--P18)

These plans improve the user-facing output quality of the CLI and TUI.

---

#### P17 -- CLI Output Format

**File:** `plans/P17-cli-output-format/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 6 | **Max parallel:** 1

**What it does:** Creates a `CliOutput` wrapper that routes all progress/error
messages through `output_format` functions (with `quiet` suppression) instead of
raw `eprintln!` calls. Also downgrades noisy config warnings from `warn!` to
`debug!`.

**Tasks:** T1 creates the wrapper struct; T2 downgrades config warnings; T3--T6
migrate ~25 `eprintln!` call sites across `do_cmd.rs`.

**IronClaw relevance:** Low. Roko-specific CLI formatting.

---

#### P18 -- TUI Agent Data

**File:** `plans/P18-tui-agent-data/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 5 | **Max parallel:** 1

**What it does:** Fixes data flow from the runner-v2 to the TUI dashboard:
agent_id prefix matching uses `:` but runner uses `/`; efficiency events
(tokens, cost) are not published; MessageDelta is not forwarded to TUI;
gate failure diagnoses are not published.

**Tasks:**

| ID | Title | Tier | LoC | Crate |
|----|-------|------|-----|-------|
| T1 | Fix agent_id prefix match (`:` vs `/`) | mechanical | 10 | roko-core |
| T2 | Publish EfficiencyEvent on turn completion | focused | 30 | roko-cli |
| T3 | Forward MessageDelta to TUI as agent_output | mechanical | 15 | roko-cli |
| T4 | Publish Diagnosis event on gate failure | focused | 30 | roko-cli |
| T5 | Add diagnosis method to TuiBridge | mechanical | 15 | roko-cli |

**Dependency note:** T4 depends on T5 (needs the `diagnosis()` method).

**IronClaw relevance:** Low. TUI-specific data plumbing. The structured
dashboard event pattern (`DashboardEvent` enum) is interesting architecturally.

---

### Theme 6: ACP / Editor Integration (P19--P22, P25, P29)

These plans improve the ACP protocol layer that connects roko to the Zed editor.

---

#### P19 -- Cascade Router ACP

**File:** `plans/P19-cascade-router-acp/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 6 | **Max parallel:** 1

**What it does:** Wires the cascade router's model selection into ACP dispatch.
The router is loaded for observation recording but its `route_with_cfactor()`
select path is never called. When the user has not explicitly overridden the
model, the router's selected primary slug should drive dispatch.

**IronClaw relevance:** Medium. IronClaw's multi-provider LLM routing would
benefit from an adaptive model router.

---

#### P20 -- Zero Config

**File:** `plans/P20-zero-config/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 5 | **Max parallel:** 2

**What it does:** Enables roko to work without explicit `roko.toml` configuration
by consulting the builtin model registry and auto-detecting available API keys.
`preflight_provider_for_model()` currently only checks explicit TOML entries.

**IronClaw relevance:** Medium. IronClaw's configuration system (env vars, `.env`
files, `settings.json`) has a similar zero-config aspiration.

---

#### P21 -- ACP Streaming

**File:** `plans/P21-acp-streaming/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 5 | **Max parallel:** 1

**What it does:** Fixes buffered output in `run_slash_command()` -- non-progress
stdout lines are accumulated into a string and only sent as a single TokenChunk
at the end. Each line should be streamed immediately.

**IronClaw relevance:** Medium. IronClaw's web gateway WebSocket streaming has
similar real-time output forwarding requirements.

---

#### P22 -- ACP Tool Permission

**File:** `plans/P22-acp-tool-permission/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 5 | **Max parallel:** 1

**What it fixes:** Three ACP code paths use `ToolContext::testing()` which
bypasses safety checks and sets `network: false`. Replaces with explicit
`ToolContext` construction using full capabilities and real cancel tokens.

**IronClaw relevance:** High. IronClaw's tool dispatch has the same concern --
`ToolContext` must carry correct capabilities. The anti-pattern of using test
constructors in production is something to audit in IronClaw.

---

#### P25 -- MCP-ACP Passthrough

**File:** `plans/P25-mcp-acp-passthrough/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 4 | **Max parallel:** 1

**What it does:** Adds an `mcp_config` field to `roko-core`'s `AgentConfig` so
MCP server configurations can flow through ACP dispatch. Currently the CLI
config layer supports it but the canonical agent struct does not.

**IronClaw relevance:** High. IronClaw's MCP integration (tools/mcp/) has the
same config passthrough need across channels.

---

#### P29 -- Develop Command Wire

**File:** `plans/P29-develop-command-wire/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 3 | **Max parallel:** 1

**What it does:** Registers `/develop` as an ACP slash command. The CLI command
exists but Zed users cannot invoke it. Overlaps with P10-T3/T4.

**IronClaw relevance:** Low. Roko-specific ACP command registration.

---

### Theme 7: PRD and Workspace Tooling (P23--P24)

---

#### P23 -- PRD Pipeline Fix

**File:** `plans/P23-prd-pipeline-fix/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 6 | **Max parallel:** 1

**What it fixes:** The PRD draft agent has `allowed_tools: Some("none")`,
generating PRDs without reading the codebase. Changes to `Some("Read,Grep,Glob")`
so the agent can inspect existing code.

**IronClaw relevance:** Medium. The pattern of gating agent tool access by
workflow stage is relevant to IronClaw's skill attenuation system.

---

#### P24 -- Workspace Paths

**File:** `plans/P24-workspace-paths/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 4 | **Max parallel:** 1

**What it fixes:** Two functions resolve the plans directory with opposite
preferences: `plan.rs` prefers `./plans/`, `main.rs` prefers `.roko/plans/`.
Aligns to `./plans/` (where actual plan directories live).

**IronClaw relevance:** Low. Path resolution consistency is a universal concern
but the specific fix is roko-internal.

---

### Theme 8: Learning and Intelligence (P26)

---

#### P26 -- HDC Similarity Lookup

**File:** `plans/P26-hdc-similarity-lookup/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 4 | **Max parallel:** 1

**What it does:** Adds `query_similar_episodes()` to `EpisodeLogger` -- decodes
stored HDC (hyperdimensional computing) fingerprints and ranks episodes by
Hamming similarity. Enables the system to find past similar agent sessions for
context enrichment.

**IronClaw relevance:** Medium. IronClaw's workspace memory system uses hybrid
search (FTS + vector via RRF). The HDC fingerprint similarity approach is a
different technique worth studying as a complement to embedding-based similarity.

---

### Theme 9: Provider and Model UX (P20, P27--P28, P33)

---

#### P27 -- Provider Error UX

**File:** `plans/P27-provider-error-ux/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 4 | **Max parallel:** 1

**What it does:** Makes `roko doctor` API key checks conditional on configured
providers. Currently `check_anthropic_api_key()` always warns, even for
OpenAI-only setups.

**IronClaw relevance:** Medium. IronClaw's `doctor` subcommand could use the
same conditional API key validation.

---

#### P28 -- Image Support

**File:** `plans/P28-image-support/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 5 | **Max parallel:** 1

**What it does:** Sets the `image` capability dynamically based on model's
`supports_vision` flag. Currently hardcoded to `false`, preventing Zed from
showing the image upload button.

**IronClaw relevance:** Medium. IronClaw's channel system negotiates
capabilities; dynamic capability detection from model profiles is applicable.

---

#### P33 -- Model UX

**File:** `plans/P33-model-ux/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 1 | **Max parallel:** 1 |
**Est:** 30 min

**What it does:** Implements `max_tokens` auto-recovery. Already-implemented
items noted in the plan: fuzzy model suggestions (jaro_winkler), setup wizard,
/models ACP command, config reload notification.

**IronClaw relevance:** Low. Model-specific UX.

---

### Theme 10: Onboarding and Doctor (P30)

---

#### P30 -- Onboarding Doctor

**File:** `plans/P30-onboarding-doctor/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 4 | **Max parallel:** 2

**What it does:** Adds OpenAI and Gemini API key checks to `roko doctor`.
Currently only `ANTHROPIC_API_KEY` is checked. Uses `Warn` severity (not `Fail`)
since no single provider is mandatory.

**IronClaw relevance:** Medium. IronClaw's doctor/status system has the same
multi-provider key validation need.

---

### Theme 11: Note Workflow and CLI Polish (P31--P32)

---

#### P31 -- Note and Context

**File:** `plans/P31-note-and-context/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 3 | **Max parallel:** 1 |
**Est:** 90 min

**What it does:** Adds plan-from-prompt shorthand and plan-from-notes clustering.
Already-implemented items noted in the plan: `roko note` command, `--context`
flag, `/note` ACP command, pinned context.

**IronClaw relevance:** Low. Roko-specific workflow enhancements.

---

#### P32 -- CLI Polish

**File:** `plans/P32-cli-polish/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 2 | **Max parallel:** 2 |
**Est:** 20 min

**What it does:** Config noise reduction and symbol consistency. Already-
implemented items: verbose debug guard, bare mode auto-detect, ContentBlock
Unknown variant, deprecation hints.

**IronClaw relevance:** Low.

---

### Theme 12: Verification Sweep (P34)

---

#### P34 -- Verification Sweep

**File:** `plans/P34-verification-sweep/tasks.toml`
**Status:** Ready (not started) | **Tasks:** 4 | **Max parallel:** 1 |
**Est:** 120 min

**What it does:** Final verification pass: compile, clippy, test, release build.
Verifies cumulative changes from P31--P33 (and prior) introduce no regressions.
Already-implemented items: do slug lookup, AcpProgressSink.

**IronClaw relevance:** Low. The concept of a terminal verification plan that
gates release quality is sound practice.

---

### Standalone Plans

---

#### e2e-smoke -- E2E Smoke Tests

**File:** `plans/e2e-smoke/tasks.toml`
**Status:** Ready | **Tasks:** 2 | **Max parallel:** 1 | **Est:** 5 min

Two trivial tasks designed to validate the plan runner itself:
- S01: Add `#[must_use]` to `generate_share_token()` (1 line, haiku-tier)
- S02: Add a unit test for `generate_share_token()` format (20 lines,
  haiku-tier)

Uses `skip_enrichment = true` to bypass prompt enrichment.

---

#### architecture-defi-critical-path -- DeFi Critical Path

**File:** `plans/architecture-defi-critical-path/tasks.toml`
**Status:** Ready | **Tasks:** 3 | **Max parallel:** 1 | **Est:** 360 min |
**Queue kind:** `architecture_implementation`

Creates chain-side primitives for the DeFi architecture path:
- D01: Chain registry client and event indexer foundation (`roko-chain` crate)
- D02: Registry query routes and passport lifecycle hooks (`roko-serve`)
- D03: Verification of the integration surface

Depends on `architecture-core-queue` Q14 (chain registries DeFi foundation),
which is referenced but not present in filesystem.

Notable features: each task has an `acceptance_contract` with version,
compile/test gates, and a `parity_ledger` linking requirement IDs to source
references and evidence files.

**IronClaw relevance:** Low. Blockchain-specific architecture.

---

### Superseded Source Plans

---

#### self-dev-ux (55 tasks, superseded)

**File:** `plans/self-dev-ux/tasks.toml` | **Status:** superseded
**Superseded by:** "P08-P34 plans (consolidated from feedback audit 2026-05-08)"
**Est:** 720 min | **Max parallel:** 20

This was the original monolithic self-development UX plan. Tasks ranged from
critical ACP usability fixes (C01--C03) through model selection improvements
(M01--M15), diagnostics (D01--D12), hardening (H01--H12), and lower-priority
enhancements (L01--L10, V01). Git history confirms many were completed
individually (H07--H09, L05, L09--L10, M01, M10--M11, V01) before the plan was
superseded.

The remaining ready tasks were redistributed into the focused P08--P34 queue for
better dependency ordering and granular progress tracking.

---

#### self-dev-extras (11 tasks, superseded)

**File:** `plans/self-dev-extras/tasks.toml` | **Status:** superseded
**Superseded by:** "P08-P34 plans (consolidated from feedback audit 2026-05-08)"
**Est:** 180 min | **Max parallel:** 20

Gap items from self-development feedback docs not covered by self-dev-ux:
zero-knowledge onboarding gaps (D01), workspace path inconsistencies, PRD
pipeline issues, and provider error handling. All redistributed into P20, P23,
P24, P27, and P30.

---

## 4. Cross-Plan Patterns and Architecture

### Task Specification Pattern

Every task follows a consistent schema that embodies executable documentation:

```toml
[[task]]
id = "T1"
title = "Human-readable title"
status = "ready"
tier = "focused"                          # complexity tier
model_hint = "claude-sonnet-4-20250514"   # which LLM should execute
max_loc = 25                              # hard LOC budget
files = ["path/to/file.rs"]              # exact modification targets
role = "implementer"                      # safety contract role
depends_on = ["T0"]                       # dependency chain

[task.context]
read_files = [
    { path = "file.rs", lines = "10-50", why = "reason" },
]
symbols = ["FnName -- description (line N)"]
anti_patterns = ["Do NOT ..."]

[[task.verify]]
phase = "structural"
command = "grep -q 'pattern' file.rs"
fail_msg = "Human-readable failure explanation"
```

This is not just documentation -- it is the input format consumed by the plan
runner's agent dispatch system.

### Recurring Architectural Themes

1. **Wire, don't build.** Most plans connect existing but unconnected code.
   The `classify_agent_crash` function exists; P15 wires it into failure paths.
   The cascade router exists; P19 wires it into ACP dispatch. This reflects
   roko's "built but never connected" pattern documented in its CLAUDE.md.

2. **Mechanical + focused layering.** Complex features are decomposed into
   mechanical tasks (haiku-class, <20 LoC, one-string changes) and focused
   tasks (sonnet-class, 20-60 LoC, single-concern changes). The mechanical
   tasks establish scaffolding; focused tasks wire the logic.

3. **Test-as-verification, not test-as-afterthought.** Every plan has structural
   grep checks, compile checks, and test run verifications. The verification
   commands are explicit shell commands with custom failure messages, not
   abstract quality gates.

4. **Anti-pattern lists as guardrails.** Each task carries 2-5 explicit "do NOT"
   instructions. These encode hard-won lessons: "Do NOT wrap the query in
   `{\"queries\": [...]}` -- Perplexity has no batch endpoint." They prevent
   the implementing agent from repeating known mistakes.

5. **Progressive hardening.** The queue moves from bug fixes (P08--P10) through
   infrastructure (P11--P12), resilience (P13--P15), safety (P16), UX
   (P17--P18), integration (P19--P22), intelligence (P26), polish (P31--P33),
   and ends with a verification sweep (P34). This is intentional: fix the
   foundation first, then build upward.

6. **Acceptance contracts** (in architecture plans) provide formal traceability
   from requirements to evidence files, with parity ledger rows linking
   `requirement_id` -> `source_ref` -> `evidence_ref`.

### Cross-Plan Dependencies

Plans depend on each other implicitly through file targets. Key dependency
chains:

- P08 T1 (search client) -> P08 T2 (date format) -> P08 T3 (tests): all
  modify `perplexity/search.rs`
- P11 (runner default) -> P12 (parallelism): P12 assumes runner-v2 is active
- P13 (rate limit) uses the same `openai_compat_backend.rs` modified by P12
- P16 T4/T5 (safety contracts in dispatch) depends on P16 T1--T3 (field/helper
  additions)
- P18 T4 (diagnosis events) depends on P18 T5 (TuiBridge method)
- P10 T3/T4 overlap with P29 (both register /develop)

---

## 5. IronClaw Relevance Assessment

### High Relevance (directly portable patterns)

| Plan | Pattern | IronClaw Mapping |
|------|---------|-----------------|
| P09 | Tool name canonicalization across providers | `ToolRegistry` cross-channel naming |
| P11 | Feature-flag-gated engine migration | Progressive tool disclosure (`flag-gated, default off`) |
| P12 | Concurrent agent dispatch with per-task tracking | `src/agent/` dispatcher, `src/context/` job management |
| P13 | HTTP error classification (429/5xx -> retryable) | `crates/ironclaw_llm/` provider error handling |
| P15 | Structured crash classification with recovery hints | `src/error.rs` error types, `src/evaluation/` |
| P16 | Per-role safety contracts propagated through dispatch | `crates/ironclaw_safety/`, tool attenuation in `src/skills/` |
| P22 | Audit for test context in production dispatch | `src/tools/dispatch.rs` ToolContext construction |
| P25 | MCP config passthrough across dispatch layers | `src/tools/mcp/` config propagation |

### Medium Relevance (adaptable approaches)

| Plan | Pattern | IronClaw Mapping |
|------|---------|-----------------|
| P08 | Serde alias for API field name mismatches | Any external API integration |
| P14 | Gate pipeline skip/activate bug pattern | `src/evaluation/` success evaluator pipeline |
| P19 | Adaptive model routing in dispatch | `crates/ironclaw_llm/` provider selection |
| P20 | Zero-config with builtin model registry | `src/config/` env-based configuration |
| P23 | Stage-gated tool access for agents | Skill trust model (trusted vs installed) |
| P26 | HDC fingerprint similarity for episode retrieval | `src/workspace/` hybrid search complement |
| P27/P30 | Conditional API key validation | `src/cli/doctor.rs` |
| P28 | Dynamic capability detection from model profiles | `src/channels/` capability negotiation |

### Implementation Blueprints for IronClaw

**1. Error Classification Pipeline (from P13 + P15)**

```rust
// In crates/ironclaw_llm/src/error.rs:
fn classify_http_error(status: u16, body: &str) -> LlmError {
    match status {
        429 => LlmError::RateLimit {
            retry_after_ms: parse_retry_after(body),
        },
        500..=599 => LlmError::ServerError(status),
        _ => LlmError::Network(format!("HTTP {status}")),
    }
}

// In src/error.rs or evaluation/:
enum CrashClass { Auth, RateLimit, ContextOverflow, ToolFailure, Unknown }
impl CrashClass {
    fn recovery_hint(&self) -> &str { /* actionable guidance */ }
}
```

**2. Tool Name Canonicalization (from P09)**

```rust
// In src/tools/registry.rs:
fn canonical_tool_name(name: &str) -> &str {
    ALIASES.iter()
        .find(|a| a.alias.eq_ignore_ascii_case(name))
        .map(|a| a.canonical)
        .unwrap_or(name)
}
```

**3. Safety Contract Propagation (from P16)**

```rust
// In src/tools/dispatch.rs:
let contract = SafetyLayer::load_for_role(role, FallbackMode::Restricted)?;
let denied_tools = contract.forbidden_tool_names();
dispatch_config.disallowed_tools = denied_tools;
```

---

## 6. Lessons Learned

### What These Plans Reveal About Roko's Development Approach

**1. Self-development as a first-class workflow.**

Roko's plans are not human-written task lists managed in Jira -- they are
TOML specifications consumed by roko's own plan runner to dispatch Claude agents
that implement the changes. The plans include model hints (which LLM tier should
execute each task), anti-patterns (things the implementing agent must not do),
and multi-phase verification commands. This is meta-programming in the truest
sense: the tool developing itself using itself.

**2. The "built but never connected" anti-pattern is real.**

A striking pattern emerges from these plans: most work is *wiring*, not
*building*. Functions like `classify_agent_crash()`, `recovery_hint()`, the
cascade router's `route_with_cfactor()`, and `AgentContract` enforcement all
*exist* in the codebase but are not called from the right places. This suggests
a development mode where features were built speculatively (or by agents) and
then a human audit identified the missing connections. IronClaw should watch for
the same pattern.

**3. Feedback-driven consolidation works.**

The superseded `self-dev-ux` (55 tasks) and `self-dev-extras` (11 tasks) were
too large and unfocused. The feedback audit on 2026-05-08 consolidated their
ready tasks into 27 focused plans (P08--P34) with explicit dependency ordering,
per-task verification, and sensible LOC budgets. The self-dev plans had
`max_parallel = 20`, which the focused plans mostly reduce to 1 or 2. This
reflects a shift from "spray and pray" parallelism to disciplined sequential
execution.

**4. LOC budgets enforce discipline.**

Every task has a `max_loc` budget, typically 5-60 lines. This prevents scope
creep: an agent implementing P10-T1 (one string change) cannot restructure the
slash command system. The budgets are calibrated to the tier: mechanical tasks
are 5-20 lines, focused tasks are 20-60, integrative tasks are 30-50. This is a
practical application of the "small change, verify, repeat" discipline.

**5. Anti-patterns are more valuable than requirements.**

The anti-pattern lists in each task are often longer and more specific than the
positive requirements. "Do NOT wrap the query in `{\"queries\": [...]}` --
Perplexity has no batch endpoint" tells the implementing agent exactly what
mistake to avoid. These anti-patterns are the accumulated wisdom of previous
failed attempts -- each one represents a bug that was encountered, diagnosed,
and encoded as a guardrail.

**6. Verification is structural, not aspirational.**

Each task has concrete verification commands: `grep -q 'pattern' file.rs`,
`cargo check -p crate`, `cargo test -p crate -- test_name`. The verification
runs automatically after the agent completes. This is fundamentally different
from "please add tests" -- it is machine-checkable, deterministic, and cannot
be skipped.

**7. Model-aware task routing is a practical optimization.**

Mechanical tasks (one-string changes, attribute additions) are routed to
`claude-haiku-4-5` (cheapest, fastest). Focused tasks go to `claude-sonnet`.
Integrative tasks requiring cross-module reasoning go to `claude-sonnet` or
`claude-opus`. This tiered routing reduces cost without sacrificing quality on
complex tasks.

**8. The queue structure prevents cascading failures.**

The primary queue runs one plan at a time, P08 through P34. Each plan's tasks
have internal dependency chains. This means a failure in P13-T2 does not block
P14 (different plan) but does block P13-T3 (same plan, depends_on). The
verification sweep at P34 catches cross-plan regressions. This is a practical
solution to the dependency management problem in large codebase changes.

**9. Acceptance contracts provide formal traceability.**

The architecture plans (architecture-defi-critical-path) include
`acceptance_contract` blocks with version numbers, compile/test gates, and
parity ledger rows that link requirement IDs to source references and evidence
files. This is a lightweight requirements-tracing system embedded in the plan
format itself -- no external tool needed.

**10. Superseded plans are archived, not deleted.**

The self-dev-ux and self-dev-extras plans are marked `status = "superseded"`
with a `superseded_by` field explaining where their tasks went. They remain in
the filesystem as historical documentation. This audit trail is valuable for
understanding why certain decisions were made.
