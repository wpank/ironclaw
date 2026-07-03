# Online Learning

Online learning for IronClaw means adaptive model routing that learns from observed outcomes while preserving the existing `crates/ironclaw_llm` provider chain.

The first version should run in shadow mode: compute the model it would have selected, log the decision, and compare it with the model actually used. Enforcement comes only after local evidence shows that the learner improves the configured objective.

## Current Baseline

IronClaw already has:

- `LlmProvider` as the stable provider trait.
- `SmartRoutingProvider` with a 13-dimension complexity scorer and `Tier` values: `Flash`, `Standard`, `Pro`, `Frontier`.
- `RetryProvider`, `CircuitBreakerProvider`, `FailoverProvider`, and `CachedProvider` decorators.
- model cost lookup in `crates/ironclaw_llm/src/costs.rs`.
- provider-chain construction through the LLM factory/resolution code.

An adaptive router must compose with those pieces. It should not replace retry, circuit breaker, failover, or cache behavior.

## Problem

Static routing is useful but cannot learn from local outcomes:

- some users' "simple" requests regularly require stronger models,
- some project tasks become cheap after context and tools are stable,
- provider quality, latency, and availability change,
- budget pressure changes the preferred quality/cost tradeoff.

Online learning turns each completed call into an observation and gradually biases future choices for similar contexts.

## Candidate Design

Add an adaptive routing decorator around model candidates. It implements `LlmProvider` and chooses one configured candidate provider per request.

Decision pipeline:

1. Extract context from the current provider request or tool-provider request type.
2. Ask the existing `SmartRoutingProvider` scorer for baseline tier and score.
3. Filter unavailable candidates using existing provider health/circuit state.
4. In shadow mode, compute the learned choice but dispatch to the configured baseline.
5. In enforce mode, dispatch to the learned choice when confidence and policy allow it.
6. Record outcome: success/failure, latency, token cost, user-visible gate result, retry count, and fallback path.

## Context Features

Keep the first feature set small and explainable:

| Feature | Source |
| --- | --- |
| baseline tier | `SmartRoutingProvider::Tier` |
| complexity score | smart routing scorer |
| estimated input tokens | request messages |
| tool-call allowed | request type and tool schema count |
| coding/security indicators | existing scorer evidence |
| conversation/job path | chat, background job, routine, Reborn mission |
| budget pressure | `CostGuard` or budget gate state |
| provider health | circuit/failover observations |
| recent retries | provider chain outcome |

Do not encode user ids, tenant ids, raw prompts, or secrets as model features. Scope observations by tenant/project where needed, but keep the feature vector privacy-preserving.

## Reward Signal

Use a weighted reward configured per deployment:

- task success or gate success,
- no retry/fallback needed,
- lower cost,
- lower latency,
- user accepted result or did not correct it,
- no policy/safety regression.

Store the raw components as well as the scalar reward. Future tuning requires seeing which component drove a decision.

## Rollout Modes

| Mode | Behavior |
| --- | --- |
| `off` | Existing provider chain only. |
| `shadow` | Compute learned selection and log it; dispatch baseline. |
| `guarded` | Enforce only when confidence is high and the learned choice is no more expensive than baseline. |
| `enforce` | Use learned routing within configured policy caps. |

Suggested policy caps:

- never exceed project/user budget gates,
- never route safety-sensitive requests below the configured minimum tier,
- fall back through existing failover on transient errors,
- allow per-provider and global kill switches.

## Persistence

Observation storage needs:

- request context hash or compact feature vector,
- selected baseline model,
- shadow/adaptive model,
- final dispatched model,
- reward components,
- latency and cost,
- error class if any,
- scope and timestamp.

If this is stored in the DB, add operations to the shared trait first and implement PostgreSQL and libSQL together. A file/workspace prototype is acceptable for shadow experiments only if it cannot affect production behavior.

## Tests

- Unit-test bandit math and reward scalarization.
- Unit-test feature extraction from representative requests without storing raw prompt text.
- Provider-chain test through the factory/resolution path proving `off` mode is identical to today.
- Caller-level test proving `shadow` logs adaptive choice while dispatching baseline.
- Caller-level test proving `enforce` obeys circuit breaker, failover, and budget gates.
- Reborn QA fixture when routing affects model tool choice, request shape, or end state.

## Operational Metrics

Track local measurements instead of promising global savings:

- regret versus baseline in shadow mode,
- cost per successful turn/job,
- retries and fallbacks per provider,
- latency distribution by tier,
- rate of adaptive choices vetoed by policy,
- user correction rate after adaptive routing.

## Implementation Cautions

- Use configurable model roles or provider metadata; do not hardcode model slugs.
- Keep the existing `SMART_ROUTING_CASCADE` behavior separate from learned routing unless deliberately migrated.
- Do not let the learner override auth, safety, context-length, or approval failures.
- Make export/debug output redacted by default.
