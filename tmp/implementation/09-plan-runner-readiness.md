# Plan Runner Readiness

This supplemental note converts the captured plan catalog into execution
readiness criteria for IronClaw-style automation.

## 1. Plan Validation Command

```text
ironclaw-plan validate tmp/implementation/benchmarking/scenarios/*.yaml
ironclaw-plan validate captured-plan tasks.toml
```

Required checks:

- TOML parses.
- every task has id, tier, description, target files, and verification commands.
- dependency ids resolve.
- no task writes outside allowed workspace.
- anti-patterns and acceptance contracts are present for non-mechanical tasks.

## 2. Dry Run Output

```json
{
  "plan_id": "P19-cascade-router-acp",
  "tasks": 8,
  "waves": 3,
  "blocked": [],
  "estimated_files": [
    "crates/ironclaw_llm/src/smart_routing.rs"
  ],
  "verification_commands": [
    "cargo test -p ironclaw_llm cascade_router"
  ]
}
```

## 3. Execution Evidence

Every executed plan should produce:

```text
plan-run/
  plan.toml
  dry-run.json
  execution-log.jsonl
  gate-verdicts.jsonl
  changed-files.txt
  summary.md
```

## 4. Readiness Levels

| Level | Meaning |
|---|---|
| cataloged | plan is documented only |
| schema-valid | plan parses and dependencies resolve |
| dry-run-valid | waves, files, and verification commands are known |
| executable | runner can execute tasks in a fixture workspace |
| production-ready | rollback, gates, and artifacts are audited |

The captured plan catalog is a strong source of implementation patterns, but it
should be treated as `cataloged` until an IronClaw-native runner validates and
dry-runs a plan.
