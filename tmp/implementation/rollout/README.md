# Rollout And Risk Playbooks

This directory describes how experimental features move from disabled code to
local fixtures, shadow runs, canaries, limited rollout, and default behavior.

| File | Purpose |
| --- | --- |
| `01-feature-rollout-runbooks.md` | Stage-by-stage runbooks for core features |
| `02-security-and-risk-register.md` | Shared rollback triggers and risk records |
| `03-feature-flag-inventory.md` | Flag keys, defaults, reload behavior, owners |
| `04-feature-threat-models.md` | Feature-specific abuse cases and validation fixtures |

## Non-Negotiables

- Runtime flags fail closed and default off.
- Shadow mode records metrics but baseline behavior remains authoritative.
- Canary/default promotion requires caller-level tests, benchmark evidence, and
  an exercised rollback switch.
- Metrics and artifacts must be redacted before persistence.
- DB changes ship for PostgreSQL and libSQL together.
