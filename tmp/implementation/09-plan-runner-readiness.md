# Plan Runner Readiness

Plan-runner automation is useful only after the plan schema is strict and dry
runs are reproducible.

## Plan Validation

Required fields per task:

- `id`
- `tier`
- `description`
- `target_files`
- `caller_boundary`
- `feature_flag`
- `verification_commands`
- `rollback`

Reject plans with unknown fields, missing rollback, or target files outside the
declared ownership scope.

## Dry Run Output

```text
plan_id
task_count
owned_files
feature_flags
verification_commands
estimated_runtime
blocked_reasons
```

Dry run must not edit files, call live providers, or create DB rows.

## Execution Evidence

Each executed task records:

- run id and task id;
- files changed;
- commands run and exit codes;
- feature exposure and metric ids if applicable;
- rollback command or flag value.

## Readiness Levels

| Level | Meaning |
| --- | --- |
| 0 | plan parses only |
| 1 | dry run resolves files and commands |
| 2 | hermetic execution works with fixtures |
| 3 | caller-level tests and rollback evidence captured |

Do not use plan-runner output as implementation evidence until level 3.
