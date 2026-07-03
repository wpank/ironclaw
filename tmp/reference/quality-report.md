# Quality Report

**Generated**: 2026-07-03
**Scope**: `/Users/will/dev/near/ironclaw/tmp`

This report is a current, concise health check for the documentation tree. It is
not a historical audit log; stale findings should be removed rather than
preserved.

## Inventory

| Metric | Current value |
|---|---:|
| Top-level directories | 8 |
| Markdown files | 116 |
| YAML benchmark fixtures | 6 |
| Total files | 122 |
| Directories including root | 19 |
| Approximate disk size | 3.2 MB |

Top-level directories:

```text
agent-intelligence/
context-memory/
core-concepts/
ecosystem/
execution-verification/
implementation/
reference/
strategy/
```

## Validation Status

| Check | Status | Notes |
|---|---|---|
| Markdown local links | Passing | Use a code-fence-aware checker so inline syntax examples are not treated as Markdown links |
| YAML fixtures | Passing | Six files under `implementation/benchmarking/scenarios/` |
| Direct Roko checkout dependency | Not present | Source references are captured-source identifiers, not required checkout paths |
| Folder organization | Passing | Topic folders are clear; volatile counts should stay in this report and root README only |

## Editorial Standard

Use these rules when editing the corpus:

- Prefer implementation contracts, owner paths, caller tests, and rollout gates over long speculative code blocks.
- Mark proposed crates/modules as proposed until they exist in the IronClaw workspace.
- Avoid exact route/event/test counts unless generated from the current checkout.
- Treat Roko paths as captured-source identifiers; do not require external Roko source access.
- Do not claim VCG truthfulness, crash safety, tamper evidence, or cost savings unless the shown implementation and benchmark evidence support the claim.
- Keep indexes short. Detailed theory belongs in child docs or appendices.

## Current Cleanup Priorities

| Priority | Area | Action |
|---|---|---|
| P0 | Pseudo-compilable snippets | Remove or rewrite snippets that use stale IronClaw APIs |
| P0 | External/source wording | Keep captured-source labels; avoid direct repo dependency language |
| P1 | Overengineered plans | Scope MVPs to existing IronClaw systems before proposing new crates or loops |
| P1 | Cross-reference quality | Use specific document titles instead of terse labels like `schemas/04` |
| P2 | Volatile stats | Keep counts in this report and README; avoid repeating them across folder indexes |

## Files That Need Continued Attention

| File | Reason |
|---|---|
| [../context-memory/budget-composition.md](../context-memory/budget-composition.md) | VCG-inspired allocation must not overclaim full VCG guarantees |
| [../execution-verification/gate-verification.md](../execution-verification/gate-verification.md) | Adaptive gate learning should use per-rung verdicts or be deferred |
| [../ecosystem/mcp-editor-integration.md](../ecosystem/mcp-editor-integration.md) | MCP/ACP capability claims should distinguish implemented tool-client support from future surfaces |
| [../agent-intelligence/online-learning.md](../agent-intelligence/online-learning.md) | Router implementation guidance should match current `ironclaw_llm` APIs |
| [../reference/glossary.md](glossary.md) | Alias entries and volatile count claims should be consolidated |

## Final Gate

Before sharing or implementing from these docs, run:

```text
markdown local-link check
YAML fixture parse for implementation/benchmarking/scenarios/*.yaml
forbidden source-dependency phrase scan
git diff --check -- tmp
```
