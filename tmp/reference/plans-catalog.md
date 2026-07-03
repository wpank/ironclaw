# Captured Plans Catalog

This catalog summarizes the captured Roko plan set as reusable planning
patterns. Plan labels such as `P08` or `P34` are source-corpus identifiers, not
commands to run in IronClaw.

## What Transfers

| Pattern | Why it is useful in IronClaw | Guardrail |
|---|---|---|
| Caller-level regression tests | Bugs often live between helper output and the side effect. | Drive the handler, factory, manager, or route that owns the effect. |
| Exact argument capture | CLI/tool wrappers regress when flags move or quoting changes. | Mocks should capture every production argument. |
| Observe -> canary -> enforce rollout | Learned routing and dedupe need measured confidence. | Preserve existing defaults until fixture evidence supports enforcement. |
| Structured remediation | Agents repair faster when failures name the artifact and rule. | Keep artifacts bounded, redacted, and source-linked. |
| Recovery classification | Crash, timeout, provider, gate, and cancellation failures need different handling. | Do not swallow parse/config errors into silent defaults. |
| Plan readiness checks | Plans should declare owners, dependencies, risk, verification, and rollback. | Avoid broad refactors without a caller-visible behavior change. |

## Captured Plan Themes

| Theme | Captured examples | IronClaw adaptation |
|---|---|---|
| Concrete bug fixes | CLI flags, config parsing, model detection, capability flags. | Add targeted tests at the command or provider boundary. |
| Plan runner infrastructure | Plan discovery, task tiers, dependency ordering, verification commands. | Reuse existing task/workflow owners; avoid a parallel agent loop. |
| Resilience | Crash classification, recovery hints, retry behavior. | Keep failures typed and observable through agent/session state. |
| Safety | Contract checks, forbidden tools, artifact bounds. | Enforce through dispatcher, sandbox, auth, and caller tests. |
| Editor/web integration | ACP/MCP/session routing, streaming, model capability metadata. | Preserve web auth, CORS/origin, body limits, and event replay behavior. |
| Learning | Episode logging, similar-session search, route feedback. | Start in observe mode with workspace-memory provenance. |
| Onboarding and model UX | Provider discovery, setup diagnostics, model suggestions. | Preserve config precedence and post-secrets LLM re-resolution. |

## Anti-Patterns To Avoid

- Helper-only tests for behavior that is enforced by a caller.
- Hardcoded provider or model defaults that bypass config resolution.
- Silent fallback on malformed config.
- Plan tasks that mix unrelated ownership areas.
- Generated snippets that imply unverified IronClaw APIs.
- Exact count, timing, or cost claims without current local measurement.

## Use In Implementation Work

1. Identify the IronClaw owner module before copying a captured pattern.
2. Read the owner spec or README.
3. Define the caller-level regression first.
4. Keep rollout modes explicit for learned or advisory behavior.
5. Update docs, parity notes, and changelog files only when behavior changes
   require it.

## Related Reference

- [v2-implementation-summary.md](v2-implementation-summary.md)
- [cross-reference-map.md](cross-reference-map.md)
- [examples/operator-debugging-runbooks.md](examples/operator-debugging-runbooks.md)
