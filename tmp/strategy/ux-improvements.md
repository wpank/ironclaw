# IronClaw UX Improvements

This document distills captured interface patterns into IronClaw UX work. It is
not a request to copy external styling or structure. The goal is clearer
feedback across REPL, TUI, and web while preserving IronClaw's current channel
architecture.

Primary IronClaw areas:

- REPL: `src/channels/repl.rs`
- TUI: `src/channels/tui.rs`
- Web gateway: `src/channels/web/`
- Web SSE: `src/channels/web/platform/sse.rs`
- Setup: `src/setup/`

## Captured UX Lessons

| Captured pattern | Useful lesson for IronClaw |
|------------------|----------------------------|
| Shared CLI symbols | Use a small, consistent visual vocabulary for running, success, warning, error, tool, and gate states. |
| Phase labels while streaming | Replace generic "thinking" with concrete phases such as planning, calling tool, waiting, verifying, summarizing. |
| Collapsed tool calls | Show one concise line by default, with expandable details for arguments and output. |
| Progress tree | Use hierarchy only for real multi-step work; avoid decorative progress displays for simple turns. |
| Gate blocks | Show verification rung, result, duration, and bounded diagnostic preview. |
| Error blocks | Include failing operation, location, retry count, and next action. |
| Session summary | End a turn or session with cost, tool count, elapsed time, and final state when useful. |
| Status bar | Keep persistent state compact: branch/project, connection, active work, key shortcuts. |
| Toasts and approval modals | Separate transient notices from decisions that block execution. |
| Replay-aware SSE | Reconnect and replay are infrastructure concerns, not per-widget logic. |
| Numbered setup wizard | Setup should show step count, validation result, and recovery path. |

Captured context is summarized in
[priority-matrix/references.md](priority-matrix/references.md) for provenance.

## Priority 1: REPL Feedback

Start here because REPL changes are visible, narrow, and do not require a new UI framework.

Recommendations:

- Add consistent line prefixes for assistant phases, tool calls, warnings, and approvals.
- Show active phase labels during long turns.
- Render tool calls as collapsed one-liners by default: tool name, short summary, status, duration.
- Add bounded error blocks with retry context and next action.
- Add optional turn summary when tools, approvals, or notable cost were involved.

Do not:

- Print noisy internal diagnostics at `info!`.
- Add a permanent dashboard to the REPL.
- Expose raw tool arguments when they may contain secrets.

Measurement:

- Snapshot tests for rendered lines.
- Manual trace review for one simple turn, one tool turn, one approval turn, and one failure turn.
- No regression in non-interactive output readability.

## Priority 2: Approval And Error UX

Approvals and failures matter because they interrupt user flow.

Recommendations:

- Use a consistent approval block across REPL and TUI.
- Show what is being requested, why it is blocked, risk level, and available choices.
- Keep denial and timeout states explicit.
- For errors, distinguish user-actionable failures from internal failures.

Do not:

- Mix approval metadata into normal chat history unless it is already part of the user-visible transcript contract.
- Display secrets, full environment dumps, or unbounded command output.

Measurement:

- Caller-level approval tests that confirm the same request renders consistently across channels.
- Redaction tests for diagnostic text.

## Priority 3: Web Status And SSE

IronClaw already has web platform code and SSE support. Improve that path rather than inventing a parallel event bus.

Recommendations:

- Audit the existing SSE event taxonomy for gaps in turn lifecycle, tool lifecycle, approval, gate, and error events.
- Ensure reconnect behavior is handled in `src/channels/web/platform/sse.rs` or its owning platform layer.
- Keep events typed and stable; UI widgets should subscribe to lifecycle events instead of parsing text.
- Add compact status cards for active turn, running tools, approvals, and recent failures.

Do not:

- Add a second browser-facing event stream for the same lifecycle.
- Expose internal raw prompts, secrets, tool input, or backend error bodies.

Measurement:

- Web integration test for reconnect/replay if the behavior changes.
- UI fixture or screenshot test for active turn, failed tool, and approval states.

## Priority 4: TUI Polish

TUI work should follow REPL semantics so channels stay consistent.

Recommendations:

- Add a four-part status bar if the current TUI lacks compact persistent state.
- Add toasts for non-blocking events.
- Add an approval modal for blocking choices.
- Add scrollable task/tool progress only when there are multiple active items.

Do not:

- Port external palettes or theme names.
- Add nested panels for simple single-turn interactions.

Measurement:

- Snapshot/render tests where available.
- Manual check in narrow and wide terminal sizes.

## Priority 5: Setup UX

Setup should be predictable and recoverable.

Recommendations:

- Number setup steps and show validation results.
- Keep config source visible when showing a setting: env, profile, DB-backed setting, or default.
- Provide a final summary with next commands and diagnostics path.
- If onboarding behavior changes, update `src/setup/README.md` in the same branch.

Do not:

- Collapse bootstrap config, DB-backed settings, and secrets into one mental model.
- Hide skipped steps without explaining why they were skipped.

Measurement:

- Setup tests for rerun/merge behavior.
- Manual run through first-time setup and rerun.

## Minimal Event Vocabulary

Use a small shared state vocabulary before expanding event types:

| State | Meaning |
|-------|---------|
| `queued` | Work accepted but not running. |
| `planning` | Agent is deciding next action. |
| `running_tool` | A tool call is in progress. |
| `waiting_approval` | User decision required. |
| `verifying` | Checks or gates are running. |
| `summarizing` | Final response or summary is being produced. |
| `completed` | Work ended successfully. |
| `failed` | Work ended with an error. |
| `cancelled` | User or system cancelled work. |

Channels can render these differently, but they should not invent conflicting meanings.

## Example REPL Shape

Current low-information pattern:

```text
Thinking...
```

Target pattern:

```text
> planning
> tool shell: cargo test --lib  running  12s
> verifying: tests passed  18s
> completed: 2 tools, 31s
```

Current unbounded error pattern:

```text
Error: command failed
```

Target pattern:

```text
! tool failed: cargo test --lib
  exit: 101
  retry: not attempted
  next: inspect first compiler error
```

## Captured Context

Use:

- Consistent state symbols and labels.
- Collapsed tool and gate displays.
- Bounded diagnostics.
- Replay-aware web transport.
- Setup step numbering.

Do not carry forward:

- External branding, palettes, and theme names.
- Unsupported live source links.
- Full dashboard scope unless a current IronClaw workflow needs it.
- Large copied code examples.

## Implementation Order

1. Define shared status labels and rendering snapshots for REPL.
2. Normalize approval/error blocks across REPL and TUI.
3. Audit web SSE lifecycle events and reconnect behavior.
4. Add TUI status/toast/modal polish using the same labels.
5. Improve setup step summaries only if setup behavior is already being changed.

Each step should be independently useful and revertible.
