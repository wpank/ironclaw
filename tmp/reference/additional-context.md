# Additional Context

This file records cross-document caveats that apply to the captured reference
set.

## Provenance

- `roko-*`, `docs/v2-depth/*`, and `plans/*` names are captured-source labels.
- Do not assume those labels are accessible checkout paths.
- Do not add path dependencies to Roko crates from IronClaw.
- Count-heavy claims should be treated as catalog metadata unless regenerated
  from local evidence in the same pass.

## Terminology

Use [terminology-glossary.md](terminology-glossary.md) when moving between
captured names and IronClaw implementation terms. Prefer `Signal`, `Store`,
`Cell`, and `Graph` in design notes unless quoting captured types such as
`Engram` or `Substrate`.

## Cross-Reference Pattern

When adding or refining reference docs, point to local files:

- Captured source family map: [source-corpus-map.md](source-corpus-map.md)
- v2 depth catalog: [v2-depth-research.md](v2-depth-research.md)
- Plan catalog: [plans-catalog.md](plans-catalog.md)
- Research bibliography: [research-citations.md](research-citations.md)
- End-to-end summary: [end-to-end-synthesis.md](end-to-end-synthesis.md)

## Implementation Posture

Use captured Roko material as design input only. The implementation target is
IronClaw's existing module ownership, composition root, security model, DB
parity, tool-dispatch path, and caller-level test discipline.
