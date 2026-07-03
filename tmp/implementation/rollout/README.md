# Rollout And Risk Playbooks

This folder turns implementation ideas into operational steps. It should be read
before enabling any feature from the numbered analysis documents outside a local
development environment.

| File | Contents |
|---|---|
| `01-feature-rollout-runbooks.md` | Shadow/canary/default rollout steps for the highest-value features |
| `02-security-and-risk-register.md` | Security review checklist, risk register, rollback triggers, and DB parity rules |
| `03-feature-flag-inventory.md` | Fail-closed feature flag inventory, ownership, stages, default states, and rollback semantics |
| `04-feature-threat-models.md` | Threat models for routing, gates, memory, dreams, code intelligence, control plane, extensions, reputation, and configuration |

## Non-Negotiables

- Feature flags must fail closed.
- Security-sensitive behavior must keep existing approval and auth paths.
- Database changes must pass both PostgreSQL and libSQL contract tests.
- Rollback must preserve readable existing data.
- Every canary needs an exposure event, metric event, guardrail verdict, and
  rollback trigger that can be joined by `run_id` and `turn_id`.
