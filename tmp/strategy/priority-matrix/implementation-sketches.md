# Implementation Sketches

These sketches define the smallest useful IronClaw-native slice for the top
candidates. They are contracts and review prompts, not porting instructions.
Before coding any item, read the relevant subsystem spec and verify the current
code path.

## 1. Robust Statistics

Goal: make cost and latency estimates less sensitive to outliers.

Minimal shape:

- Add tested helpers for `median`, `trimmed_mean`, and `mad`.
- Use them only where observations are known to be heavy-tailed, such as recent LLM cost ratios or tool latency.
- Preserve raw samples so legitimate drift can still be detected.

Caller-level test:

- Drive the existing estimator with a normal sequence, one extreme spike, then normal data.
- Assert the estimator does not swing sharply on the spike and still adapts when the shifted regime persists.

Avoid:

- Replacing every average in the codebase.
- Silently dropping observations.

## 2. BLAKE3 Memory Dedup

Goal: prevent exact duplicate memory entries from polluting search.

Minimal shape:

- Compute a BLAKE3 hash from normalized memory content at write time.
- Look up an existing entry with the same hash in the same scope.
- Merge by updating metadata such as `access_count`, `last_seen_at`, or source list.
- Return a tool-visible message when the write merges with an existing memory.

Caller-level test:

- Call the memory write tool twice with identical content.
- Assert only one durable entry is visible through the normal read/search path and the second response names the merge.

Avoid:

- Fuzzy dedup in the first PR.
- A schema change if existing metadata can safely carry the hash.

## 3. Ebbinghaus Memory Decay

Goal: make stale memory less prominent while preserving recoverability.

Minimal shape:

- Store decay state in document metadata unless a schema change is clearly necessary.
- Rank or filter by current strength during memory search.
- Archive stale entries with metadata, never delete content.
- Exempt identity, system, and agent instruction documents.

Caller-level test:

- Write ordinary memory and identity memory through normal tools.
- Advance time in a test clock.
- Assert ordinary memory loses rank or becomes archived, while identity memory remains active.

Avoid:

- Running archival during an active user turn.
- Treating decay as a destructive cleanup job.

## 4. Metacognitive Monitor

Goal: detect stuck turns and cost runaways earlier without adding another agent loop.

Minimal shape:

- Extend existing duplicate/stuck tracking in the agent loop.
- Track recent tool call signatures, outcomes, and projected cost.
- Emit a structured intervention decision: continue, ask user, change plan, or stop.
- Start with conservative thresholds and debug logging.

Caller-level test:

- Drive the real agent-loop boundary or a close harness with alternating failing tool calls.
- Assert the monitor intervenes after the configured threshold and does not intervene for diverse successful calls.

Avoid:

- Storing monitor reasoning in normal chat history unless it is intentionally user-visible.
- Duplicating job-level self-repair logic.

## 5. Composable Scorers

Goal: share quality scoring between evaluation and gate-style checks.

Minimal shape:

- Define a small trait that returns score, confidence, label, and optional diagnostic.
- Add weighted composition only after at least two real scorers exist.
- Keep the first caller narrow, such as a gate criterion or evaluation metric.

Caller-level test:

- Drive the real evaluation or gate caller with two scorer outputs.
- Assert the caller's side effect uses the composed result as intended.

Avoid:

- A generic scoring framework with no production caller.
- Hardcoding quality thresholds in scattered helpers after the trait exists.

## 6. Hierarchical Cancellation

Goal: ensure cancelling a session or turn stops in-flight tools and child processes.

Minimal shape:

- Thread cancellation tokens through session, turn, dispatcher, and tool execution.
- Give each tool call a child token.
- Make subprocess tools select on cancellation and terminate children deliberately.

Caller-level test:

- Start a tool call that runs a long subprocess, cancel the session or turn, and assert the process exits within a bounded time.

Avoid:

- Testing only token propagation.
- Leaving cancellation as advisory for shell/process tools.

## 7. Cascade Router

Goal: reduce model cost without lowering quality or bypassing safety routing.

Minimal shape:

- Keep current smart routing as the baseline and fallback.
- Add shadow-mode decision logging first.
- Train or evaluate a contextual bandit only from completed episodes with outcome labels.
- Place static safety overrides before learner decisions.

Caller-level test:

- Run representative requests through the provider selection boundary with shadow mode enabled.
- Assert selected provider is unchanged while candidate decisions are recorded.
- Assert security-sensitive requests never route below the required tier.

Avoid:

- Enabling active adaptive routing without a baseline.
- Letting the learner override auth, sandbox, secrets, or security routes.

## 8. Gate Verification Expansion

Goal: catch generated-code defects through progressive checks.

Minimal shape:

- Extend existing gate or builder validation paths before adding a new crate.
- Add rungs in order: compile, lint, tests, symbol checks.
- Use explicit command arguments, never shell-interpolated strings.
- Truncate diagnostics and redact secrets before returning output.

Caller-level test:

- Drive the builder or gate caller against a temporary project with a known compile error and a known failing test.
- Assert the right rung fails and diagnostics are bounded/redacted.

Avoid:

- Running arbitrary user commands as verification.
- Mixing policy approvals into normal chat history.

## 9. HDC Similarity Signal

Goal: test whether binary hypervector fingerprints improve compositional memory retrieval.

Minimal shape:

- Prototype an encoder and scorer behind a disabled flag.
- Store or compute fingerprints in a way that does not disturb existing FTS/vector search.
- Fuse HDC as an additional rank source only after offline results justify it.

Caller-level test:

- Run memory search over a fixed fixture corpus with and without HDC.
- Assert no regression on ordinary keyword/vector queries and measure relevant@10 on compositional queries.

Avoid:

- Creating a full HDC crate before a retrieval benchmark shows value.
- Replacing existing RRF with HDC-only ranking.

## 10. Cognitive Speed Labels

Goal: expose a small request-classification signal for routing, budgets, and UX.

Minimal shape:

- Define a compact enum such as `Reactive`, `Reflective`, and `Background`.
- Classify at the turn boundary from existing request context.
- Use it only where a caller needs it: routing features, budget defaults, or UI status.

Caller-level test:

- Drive the real request admission or dispatch boundary with representative inputs.
- Assert labels are attached and consumed by the selected caller.

Avoid:

- Building a taxonomy that does not change behavior.
- Coupling labels to specific model brand names.

## Shared Review Checklist

Every implementation PR should answer:

- What current IronClaw code path owns this behavior?
- What is the smallest reversible change?
- Which caller-level test proves the behavior?
- What metric will show whether it helped?
- How is rollback performed?
- Does this touch auth, secrets, sandboxing, listeners, outbound HTTP, approvals, or database schema?
