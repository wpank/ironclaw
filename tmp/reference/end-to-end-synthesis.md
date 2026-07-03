# End-to-End Synthesis

This synthesis summarizes the local end-to-end flow without requiring access to
captured-source paths. For the full walkthrough, use [end-to-end-flow.md](end-to-end-flow.md).

## Captured Flow Shape

1. Channel or trigger input is normalized into an inbound request.
2. Session, thread, and turn state determine whether the input is a command,
   approval/auth continuation, background routine, or normal chat turn.
3. Prompt/context composition selects identity files, skills, workspace memory,
   tool descriptions, and task state.
4. The agent loop alternates LLM calls and tool dispatch through the approved
   execution path.
5. Verification, persistence, event emission, and learning hooks record the
   result before the final channel response.

## IronClaw Anchors

| Concern | Local anchor |
|---|---|
| Channel normalization | `src/channels/`, especially `src/channels/channel.rs` |
| Session/thread/turn handling | `src/agent/`, `crates/ironclaw_threads/`, `crates/ironclaw_turns/` |
| Tool execution | `src/tools/dispatch.rs` via `ToolDispatcher` |
| Persistence | `src/db/` with PostgreSQL/libSQL parity |
| Memory/search | `src/workspace/` and workspace memory tools |
| Web/SSE/WebSocket | `src/channels/web/` |

## Implementation Rule

When translating captured Roko flow concepts, do not add `roko-*` dependencies
or bypass IronClaw composition roots. Extend the owning IronClaw module and add
caller-level tests at the boundary where a side effect occurs.
