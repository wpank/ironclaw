# Caller-Level Test Matrix

This matrix turns "test through the caller" into concrete test targets. It is
not enough to test helpers when a helper controls a side effect.

| Feature | Test target | Fixture | Command shape | Mocked dependency | Side effect protected | Required assertion |
|---|---|---|---|---|---|---|
| Cascade router | LLM routing facade/provider factory | high-risk private prompt | `cargo test -p ironclaw_llm cascade_router` | fixture providers | provider/model choice | static safety rule wins over bandit |
| HDC memory search | memory tool or workspace facade | paraphrased duplicate memories | `cargo test workspace_memory_hdc` | deterministic embeddings | memory write/search | near duplicate linked, no false hard merge |
| Signal records | DB memory repository | exact duplicate content | DB contract test both backends | Postgres/libSQL fixtures | persistent memory identity | same hash resolves to canonical record |
| Progressive gates | code-generation/tool-building caller | compile error patch | `cargo test generated_code_gate` | command runner fixture | submission/status | compile failure blocks through caller |
| Provider conductor | provider wrapper + circuit breaker | latency ramp | `cargo test -p ironclaw_llm provider_conductor` | fixture providers | routing bias/fallback | bias changes before reactive breaker |
| Dream consolidation | heartbeat/routine engine | rare failure session | `cargo test heartbeat_dream_consolidation` | deterministic LLM + memory store | background LLM spend and memory write | derived memory is tainted and budgeted |
| DAG runner | workflow runner | branch + required gate graph | `cargo test dag_runner_contract` | fixture cell registry | ordered side effects | required branch cannot silently skip |
| Event replay | web SSE/WebSocket handler | disconnect/reconnect cursor | `cargo test web_event_replay` | in-memory event bus | event delivery | terminal event delivered exactly once |
| Code search | workspace search facade | polyglot fixture repo | `cargo test workspace_code_search` | fixture parser | index/search output | expected symbol appears in top 5 |
| Plugin hooks | extension registry lifecycle | denied network plugin | `cargo test extension_permissions` | sandbox/network mock | network/filesystem action | denied request fails closed |
| Reputation | tool outcome event path | repeated failed tool | `cargo test local_reputation` | event store fixture | tool selection bias | bad-tool selection decreases |
| Control-plane projection | HTTP/SSE/WebSocket handler | unauthorized + reconnect | `cargo test web_projection_auth` | event bus fixture | user/session visibility | unauthorized projection is rejected |
| Feature flags | feature evaluation at caller | kill switch toggle | `cargo test feature_flags` | settings facade | all experimental behavior | disabled flag restores baseline |

## Test Artifact Template

```text
test name:
feature:
caller boundary:
fixture:
mocked dependency:
side effect:
assertions:
backend modes:
security checks:
```

## Minimum Rule

If a feature can trigger HTTP, DB writes, tool execution, provider calls, memory
writes, approvals, or background jobs, it must have a caller-level test in this
matrix before canary.

