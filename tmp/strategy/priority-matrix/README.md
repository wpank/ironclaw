# Priority Matrix

Navigation: [strategy index](../README.md) | [integration roadmap](../integration-roadmap.md)

## Purpose

This matrix ranks captured ideas by near-term IronClaw value. It is not a
mandate to port external systems wholesale. A candidate moves from this folder
into implementation only after:

1. The current IronClaw subsystem has been checked.
2. A baseline metric has been captured.
3. The rollout has a kill switch or a narrow revert path.
4. Caller-level tests cover the behavior that changes.

## Scoring Model

Each candidate is scored from 1 to 5 on five axes. Higher is better.

| Axis | Weight | Meaning |
|------|--------|---------|
| User impact | 30% | Observable improvement for users: fewer failed turns, lower cost, better retrieval, clearer UX. |
| System impact | 20% | Reliability, maintainability, observability, or future extensibility. |
| Ease | 25% | Small surface area, few ownership boundaries, low migration burden. |
| Safety | 15% | Reversible, flaggable, low chance of corrupting state or weakening security. |
| Independence | 10% | Can ship without waiting for other new capabilities. |

```
score = user*0.30 + system*0.20 + ease*0.25 + safety*0.15 + independence*0.10
```

Scores are directional planning inputs. A high score does not override subsystem
ownership, security review, database parity, or caller-level testing.

## Build Order

The highest composite score is not always the first implementation task. Build order also accounts for risk, prerequisites, and measurement readiness.

| Order | Candidate | Planning score | Tier | Why now |
|-------|-----------|-------|------|---------|
| 1 | Robust statistics | 4.20 | Quick win | Pure functions plus narrow estimator integration; improves measurement quality for later work. |
| 2 | BLAKE3 memory dedup | 4.30 | Quick win | Exact-match dedup is easy to reason about and reversible if stored as metadata. |
| 3 | Ebbinghaus memory decay | 4.50 | Quick win | Builds on dedup; must archive, never delete. |
| 4 | Metacognitive monitor | 4.55 | Quick win | Extends existing stuck-loop handling with turn-level patterns and spend projection. |
| 5 | Composable scorers | 3.90 | Quick win | Creates a small shared evaluation contract for gates and quality checks. |
| 6 | Hierarchical cancellation | 3.50 | Quick win | Important reliability work, but test through actual tool/process boundaries. |
| 7 | Cascade router | 4.15 | Big bet | High value only after shadow-mode data proves safe routing. |
| 8 | Gate verification expansion | 3.85 | Big bet | Valuable, but must extend existing validation/gate code without unsafe command execution. |
| 9 | HDC similarity signal | 3.85 | Big bet | Promising retrieval signal; validate against current FTS/vector RRF before building a crate. |
| 10 | Cognitive speed labels | 3.50 | Big bet | Useful mainly as routing/context metadata; do not build as a standalone taxonomy project. |

## Tiers

| Tier | Candidates | Build stance |
|------|------------|--------------|
| Quick wins | Robust stats, dedup, decay, monitor, scorers, cancellation | Start here. Keep each PR small and reversible. |
| Big bets | Cascade router, gate expansion, HDC, cognitive labels | Require baselines, design review, and staged rollout. |
| Conditional | Enhanced heartbeat, EventBus replay, conductor, checkpoints, DAG runner, declarative tools, engagement tracker | Build only when a product need or dependency makes the value concrete. |
| Research | Full dreams, budget auctions, broad code intelligence, NEAR reputation, pheromones, state-machine extraction, full affect engine, TDA/sheaves | Keep as research notes until feasibility and user value are demonstrated. |

## Calibration Against IronClaw

The recommendations assume these current IronClaw capabilities:

| Existing capability | Current area | Planning implication |
|---------------------|--------------|----------------------|
| Agent loop and duplicate tool-call tracking | `src/agent/agentic_loop.rs` | Monitoring should extend existing turn logic, not add a second loop. |
| Job self-repair | `src/agent/self_repair.rs` | New monitor should avoid duplicate escalation paths. |
| Cost guard | `src/agent/cost_guard.rs` | Spend projection should read existing budget state when available. |
| Workspace memory and RRF search | `src/workspace/` | Dedup, decay, and HDC must preserve file-like memory semantics. |
| Built-in memory tool | `src/tools/builtin/memory.rs` | Memory side effects should be tested through the tool call path. |
| Tool dispatcher | `src/tools/dispatch.rs` | Workflow and cancellation work must preserve dispatch auditing. |
| Smart routing provider | `crates/ironclaw_llm/src/smart_routing.rs` | Adaptive routing should wrap or shadow existing routing before replacing it. |
| Existing gate infrastructure | `crates/ironclaw_engine/src/gate/`, `src/tools/builder/validation.rs` | Gate work should extend current boundaries before introducing new crates. |
| Dual database backends | `src/db/` | New persistence needs PostgreSQL and libSQL support unless stored in existing metadata. |

## Decision Rules

- If a feature touches auth, secrets, sandboxing, listeners, or outbound network paths, require security review before implementation.
- If a feature changes DB shape, implement both backends and add contract tests first.
- If a feature gates a side effect, test through the caller that performs the side effect, not only through helper functions.
- If the expected lift cannot be measured, run it in observe-only mode before enabling behavior changes.
- If the captured design used a large generalized subsystem, prefer the
  smallest IronClaw-native slice that solves the current problem.

## What Not To Build Yet

- Do not build full dream consolidation before decay, dedup, cost caps, and background-run isolation are validated.
- Do not build an on-chain reputation bridge before an off-chain trust signal has users and measurable decisions.
- Do not replace the model router with a learner until shadow mode shows equal quality and no safety override violations.
- Do not add a new workflow engine unless existing Reborn runner/driver/executor boundaries cannot express the needed behavior.
- Do not transplant external UI aesthetics; keep the interaction lessons, not
  the brand.
