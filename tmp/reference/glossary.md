# Glossary

Concise definitions for terms used across the reference docs. Captured Roko
terms are normalized through [terminology-glossary.md](terminology-glossary.md).

## Core Architecture

| Term | Definition |
|---|---|
| Signal | Captured v2 name for the universal record: an observation, memory, verdict, tool trace, or derived lesson with identity and provenance. |
| Engram | Captured v1/source term for Signal. Use only when quoting captured types or docs. |
| Store | Persistence abstraction for Signals and indexes; maps to IronClaw DB/workspace-owned storage. |
| Substrate | Captured older name for Store. |
| Cell | Typed computation boundary. In IronClaw, translate to an owned tool, workflow step, handler, service, or gate. |
| Graph | Dependency-aware plan of Cells. In IronClaw, use the existing runner/workflow path. |
| Bus | Event stream for transient progress, tool, provider, gate, or projection updates. |
| Trigger | External event source such as webhook, schedule, file watch, or channel input. |
| Compose | Prompt/context assembly from identity, task, memory, tools, and budget. |
| Route | Model/provider/tool selection under policy and cost constraints. |
| Verify / Gate | Structured acceptance check for an artifact, model output, or side effect. |
| Verdict | Structured result of a gate, including pass/fail, evidence, and remediation. |

## Memory And Search

| Term | Definition |
|---|---|
| HDC | Hyperdimensional computing; represents items as high-dimensional vectors and compares them with cheap vector operations. |
| VSA | Vector Symbolic Architecture; broader family that includes HDC-style bind, bundle, and permute operations. |
| Hypervector | Fixed-width vector used for HDC similarity or symbolic composition. |
| Bind | HDC operation that combines role and value into a reversible-like association. |
| Bundle | HDC operation that superimposes several vectors into one approximate set representation. |
| Permute | HDC operation that encodes order or position. |
| AntiKnowledge | Captured term for explicit "known false" or disproved knowledge that should prevent repeated mistakes. |
| Taint | Trust/provenance marker that limits how derived, unverified, or unsafe data can be used. |
| Lineage | Source chain explaining where a memory, verdict, or derived lesson came from. |
| Demurrage | Decay-inspired weighting: stale or unused knowledge can lose retrieval weight without being physically deleted. |
| Dream consolidation | Background or offline learning that summarizes, replays, or distills past episodes into derived memory. |
| Derived memory | Memory produced by the system rather than directly authored by the user; should carry source links and lower initial confidence. |

## Agent Runtime

| Term | Definition |
|---|---|
| Agent loop | The turn loop that alternates model calls, tool dispatch, observation, and final response. |
| ToolDispatcher | IronClaw action boundary for tool execution. Captured tool ideas must preserve this path. |
| Approval | User or policy gate before sensitive tool execution. |
| Routine | Scheduled or background work owned by the runtime. |
| Heartbeat | Runtime tick that may drive maintenance or background tasks inside budget and policy limits. |
| Checkpoint | Durable progress state used to resume or explain a run. |
| Cancellation | Explicit interruption path that should propagate through the runtime and tool boundaries. |
| Conductor | Captured supervisory concept for provider health, stuck-loop detection, and intervention advice. |
| Circuit breaker | Hard runtime guard that stops repeated failing provider or service calls. |
| Cost guard | Budget enforcement for foreground or background model/tool work. |

## Verification And Quality

| Term | Definition |
|---|---|
| Progressive gate | A staged verification approach where cheaper checks run before more expensive checks. |
| Caller-level test | Test that drives the production boundary where a side effect happens, not only the helper that classifies it. |
| Remediation | Specific repair instruction derived from a failed gate. |
| Artifact | File, diff, log, tool output, or report retained as bounded evidence. |
| Rollout guardrail | Metric or condition that controls observe, canary, and enforce modes. |
| Shadow mode | Advisory mode where a new classifier/router records decisions but does not control behavior. |
| Canary | Limited enforcement on safe traffic before broader rollout. |
| False block | Verification failure that incorrectly stops valid work. |

## Routing And Learning

| Term | Definition |
|---|---|
| Cascade router | Captured model-routing pattern that combines static rules, confidence checks, and learned selection. |
| Static safety rule | Non-learned rule that blocks or forces a safer path for private, high-risk, or policy-sensitive requests. |
| Bandit | Online-learning method that balances exploration and exploitation across choices such as providers. |
| LinUCB | Contextual bandit algorithm commonly used for explainable exploration with uncertainty bonuses. |
| Reward signal | Measured outcome used to update routing, memory, or policy choices. |
| Provider health | Latency, error, retry, and cost observations used to avoid degraded providers. |
| Drift | Change in data, provider behavior, or task mix that invalidates old assumptions. |
| Calibration | Alignment between predicted confidence and observed success. |

## Security And Integration

| Term | Definition |
|---|---|
| MCP | Model Context Protocol; external tool/resource/prompt server integration. |
| ACP | Captured editor/agent coordination protocol concept; translate through IronClaw channel and web boundaries. |
| WASM tool | Sandboxed extension capability executed with host-controlled permissions. |
| Extension lifecycle | Install, authenticate/configure, activate, use, revoke, and remove. |
| Sandbox | Runtime restriction around filesystem, network, credentials, CPU, memory, and tool authority. |
| Credential name | Backend credential storage identity; do not conflate with user-facing extension names. |
| Bearer auth | Token-based access control that must remain intact on web/API/SSE/WebSocket paths. |
| CORS/origin check | Browser-facing origin policy that must not be weakened. |
| Body limit | Request-size control for listener safety. |
| Secret handling | Rules for loading, storing, redacting, and injecting credentials. |

## Ecosystem Terms

| Term | Definition |
|---|---|
| Pheromone | Captured coordination marker: a decaying hint left for future tasks or agents. |
| Reputation | Trust score based on observed outcomes; advisory unless paired with hard approval and sandbox checks. |
| Soulbound token | Non-transferable identity-token concept used in captured chain-reputation designs. |
| x402 | HTTP payment protocol concept referenced by captured marketplace/economics material. |
| Chain witness | On-chain or signed evidence of an event; future-facing for IronClaw unless an integration is implemented. |
| Control plane | Operator-facing state projection for runs, tools, events, and health. |
| Event projection | View built from event streams for UI/API consumers. |
| Reconnect cursor | Client marker used to resume SSE/WebSocket streams without gaps or duplicate terminal events. |

## Research Terms

| Term | Definition |
|---|---|
| Active inference | Theory of action as expected-surprise reduction; use as inspiration for context/routing scoring, not as a guarantee. |
| VCG | Mechanism-design family for incentive-compatible allocation under strong assumptions; avoid claiming truthfulness without matching those assumptions. |
| Ebbinghaus curve | Forgetting-curve model used as an analogy for memory weighting. |
| Somatic marker | Affect-linked memory of prior decisions; advisory signal for prioritization or routing. |
| PAD | Pleasure/arousal/dominance affect vector used in captured affect-engine designs. |
| CUSUM / EWMA | Process-control statistics for detecting drift or trend. |
| HNSW | Approximate nearest-neighbor graph index for vector search. |
| RRF | Reciprocal rank fusion, a way to combine ranked search result lists. |
| TDA | Topological data analysis, used in captured material as a metric-shape and anomaly-analysis idea. |

For source-family mappings, see [source-corpus-map.md](source-corpus-map.md).
