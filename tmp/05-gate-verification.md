# Gate Verification Pipeline

**Source crate**: `roko-gate` (`crates/roko-gate/`)
**Priority**: HIGH -- direct enhancement to tool builder validation and code generation QA

---

## Table of Contents

1. [Introduction: What Is a Gate?](#introduction-what-is-a-gate)
2. [The Problem: Why Fixed Pass/Fail Is Insufficient](#the-problem-why-fixed-passfail-is-insufficient)
3. [Progressive Verification: The Core Idea](#progressive-verification-the-core-idea)
4. [Architecture Overview](#architecture-overview)
5. [The Verify Trait and Verdict](#the-verify-trait-and-verdict)
6. [The 7-Rung Pipeline](#the-7-rung-pipeline)
7. [What Each Rung Catches](#what-each-rung-catches)
8. [Complexity-Driven Rung Selection](#complexity-driven-rung-selection)
9. [Rung Dispatch: From Enum to Concrete Gates](#rung-dispatch-from-enum-to-concrete-gates)
10. [The GatePipeline Orchestrator](#the-gatepipeline-orchestrator)
11. [Gate Composition: Parallel, Voting, Fallback](#gate-composition-parallel-voting-fallback)
12. [Adaptive Thresholds](#adaptive-thresholds)
13. [Statistical Process Control (SPC)](#statistical-process-control-spc)
14. [Process Reward Model](#process-reward-model)
15. [Gate Ratchet: Preventing Rung Regression](#gate-ratchet-preventing-rung-regression)
16. [Forensic Causal Chain Reconstruction](#forensic-causal-chain-reconstruction)
17. [Acceptance Contracts](#acceptance-contracts)
18. [Agent Feedback Filtering](#agent-feedback-filtering)
19. [Hotelling's T-Squared: Joint Anomaly Detection](#hotellings-t-squared-joint-anomaly-detection)
20. [PELT: Offline Change Point Detection](#pelt-offline-change-point-detection)
21. [IronClaw Integration Plan](#ironclaw-integration-plan)
22. [Implementation Architecture](#implementation-architecture)
23. [Complexity Assessment](#complexity-assessment)
24. [References](#references)

---

## Introduction: What Is a Gate?

A **gate** is a verification checkpoint that an AI agent's output must pass before it is accepted. In the roko codebase, gates are the bridge between the agent's reasoning and external reality: they compile code, run tests, check linting rules, verify symbol resolution, and judge output quality. A gate takes an agent's work product as input and produces a **verdict** -- a structured result that says whether the output is correct, and if not, why.

The term comes from quality engineering, where a "quality gate" is a decision point in a manufacturing or software process where defined criteria must be met before work proceeds to the next stage. In the context of AI agents, gates serve a critical safety function: they prevent errors from propagating through a multi-step agentic workflow. An agent that generates code with a type error should discover that error at the compile gate, not after deploying the code.

**Why progressive verification matters for AI agents**: LLM agents generate output that can be subtly wrong -- code that compiles but fails tests, tests that pass but do not actually cover the intended behavior, or changes that fix one problem while introducing another. A single-pass binary check (compile or not) catches only the most superficial errors. Progressive verification applies increasingly rigorous checks in sequence, where each rung catches a different class of error. This "defense in depth" approach ensures that errors are caught at the earliest, cheapest point in the pipeline, and that expensive verification (LLM-generated tests, integration tests) is only invoked when cheaper checks have already passed.

**How this prevents error propagation**: Without gates, an agent that generates broken code in step 3 of a 10-step plan will propagate that error through steps 4-10, wasting compute and context tokens on work that builds on a broken foundation. With the gate pipeline, step 3's output is verified before step 4 begins. If the gate fails, the agent receives structured feedback about what went wrong and can retry -- before the error compounds.

---

## The Problem: Why Fixed Pass/Fail Is Insufficient

Traditional CI/CD systems use a binary gate model: a build either passes or fails. This approach has three fundamental shortcomings when applied to AI agent verification:

**1. Cost blindness.** Running all verification checks on every change is wasteful. A single-line typo fix does not need property-based fuzzing or integration tests, yet a binary gate system either runs everything (wasting budget) or runs nothing beyond compile (missing real problems).

**2. Static thresholds.** A fixed pass rate of 0.85 may be perfectly reasonable for a well-tested codebase but absurdly lenient for a security-critical path. Conversely, a new codebase with evolving APIs may legitimately have a lower pass rate during early development. Static thresholds cannot adapt to these realities.

**3. No early termination signal.** When a complex verification pipeline is running, there is no signal to decide "this trajectory is clearly failing -- stop wasting resources." A binary gate system runs everything and only then reports failure, burning expensive LLM-generated test budget on changes that clearly will not pass.

The roko-gate crate solves these problems with **progressive verification**: a pipeline of increasing rigor that adapts to the complexity of the change, learns from historical pass rates, and provides continuous promise/progress signals for early termination decisions.

---

## Progressive Verification: The Core Idea

Progressive verification is built on three principles:

**Proportional effort.** The verification rigor should be proportional to the risk of the change. A trivial change (rename, typo fix) gets only a compile check. A complex architecture change gets every verification rung including LLM-generated tests, property-based fuzzing, and integration tests.

**Adaptive baselines.** Pass/fail thresholds are not static numbers -- they are exponential moving averages that adapt to the empirical pass rate of each gate. If a codebase's compile gate passes 99% of the time, a sudden run of failures is detected and escalated. If a property-test gate historically passes 60% of the time, that is its learned baseline, not a sign of failure.

**Cybernetic feedback.** Each gate produces a `Verdict` with a numeric score, duration, and structured error data. These verdicts feed into a Process Reward Model that computes two continuous signals -- **Promise** (probability of eventual success) and **Progress** (trajectory delta between turns) -- enabling the orchestrator to terminate early or escalate before wasting budget.

**Why progressive beats single-pass**: A single-pass verification system has two failure modes. It can be too lenient (running only compile, missing logic errors that tests would catch) or too expensive (running everything including integration tests on a one-line config change). Progressive verification avoids both: it starts cheap and escalates only when warranted by the complexity of the change or by repeated failures. This makes it both more thorough and more economical than either extreme.

---

## Architecture Overview

The roko-gate crate ships a two-tier gate system:

```
                              +--------------------------+
                              |    GatePipeline /         |
                              |  ComposedGatePipeline     |
                              |  (orchestration layer)    |
                              +---------+----------------+
                                        |
                    +-------------------+-------------------+
                    |                                       |
          +---------v----------+              +-------------v-----------+
          | Rung-Dispatched     |              | Standalone Gates        |
          | (7 rungs, ordered)  |              | (invoked ad-hoc)        |
          +--------------------+              +-------------------------+
          | 0: CompileGate      |              | DiffGate                |
          | 1: ClippyGate       |              | CodeExecutionGate       |
          | 2: TestGate         |              | ShellGate               |
          | 3: SymbolGate       |              | BenchmarkRegressionGate |
          | 4: GeneratedTestGate|              | FormatCheckGate         |
          |    + VerifyChainGate|              | SecurityScanGate        |
          | 5: PropertyTestGate |              | GateGenerator (dynamic) |
          |    + FactCheckGate  |              +-------------------------+
          | 6: LlmJudgeGate    |
          |    + IntegrationGate|
          +--------------------+

                    +-------------------------------------------+
                    |         Composition Wrappers               |
                    +-------------------------------------------+
                    | ParallelGate  -- all must pass             |
                    | VotingGate    -- N-of-M must pass          |
                    | FallbackGate  -- try primary, then backup  |
                    +-------------------------------------------+

                    +-------------------------------------------+
                    |         Cross-Cutting Systems              |
                    +-------------------------------------------+
                    | AdaptiveThresholds -- EMA + CUSUM per rung |
                    | SpcDetector -- CUSUM + EWMA + BOCPD        |
                    | HotellingDetector -- joint anomaly          |
                    | ProcessRewardModel -- promise + progress    |
                    | GateRatchet -- anti-regression              |
                    | ForensicReplayBuilder -- causal audit       |
                    | AcceptanceContract -- done-gate spec        |
                    | GateFeedback -- filtered agent output       |
                    +-------------------------------------------+
```

> **Source**: Architecture overview is documented in `crates/roko-gate/src/lib.rs` lines 1-50 (module doc comment), which defines the two-tier system and lists all gates.

---

## The Verify Trait and Verdict

Every gate in the system implements a single trait from `roko-core`:

```rust
// Source: crates/roko-core/src/traits.rs, line 213-226
#[async_trait]
pub trait Verify: Send + Sync {
    /// Verify the engram and return a verdict.
    async fn verify(&self, engram: &Engram, ctx: &Context) -> Verdict;

    /// Verify a batch of ephemeral pulses by promoting them to a synthetic engram.
    async fn verify_stream(&self, pulses: &[Pulse], ctx: &Context) -> Verdict {
        let synthetic = Engram::from_pulses(pulses);
        self.verify(&synthetic, ctx).await
    }

    /// Human-readable name (appears in verdicts).
    fn name(&self) -> &str;
}
```

The `Verify` trait is the fundamental abstraction. Every gate -- whether it shells out to `cargo check`, runs an LLM judge, or performs multivariate anomaly detection -- implements this trait and returns a `Verdict`:

```rust
// Source: crates/roko-core/src/verdict.rs, line 50-70
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Verdict {
    /// Did the signal pass the gate?
    pub passed: bool,
    /// Human-readable reason (used for logs, error messages).
    pub reason: String,
    /// Identifier of the gate that rendered this verdict.
    pub gate: String,
    /// Numeric score in [0..1] -- useful for thresholding (e.g. judge gates).
    pub score: f32,
    /// Optional detail string (stdout, error output, diagnostic).
    pub detail: Option<String>,
    /// Structured test counts (populated by test gates).
    pub test_count: Option<TestCount>,
    /// Structured error digest for feeding back to agents.
    pub error_digest: Option<String>,
    /// Wall-clock duration the gate took, in milliseconds.
    pub duration_ms: u64,
}
```

The `Verdict` struct is the common currency of the entire verification system. It carries not just a boolean pass/fail, but a continuous score (0.0 to 1.0), structured test counts, error digests for agent feedback, and wall-clock timing. This richness is what enables the Process Reward Model, adaptive thresholds, and forensic replay.

The `Verdict` also provides helper constructors and a `is_mostly_passing()` method that classifies test verdicts where the pass rate exceeds 90% but some tests still fail -- useful for policies that want to distinguish "mostly working" from "completely broken."

> **Source**: `crates/roko-core/src/traits.rs` line 213 for the trait; `crates/roko-core/src/verdict.rs` line 50 for the struct.

---

## The 7-Rung Pipeline

The canonical pipeline consists of 7 rungs of increasing rigor and cost. Each rung maps to one or more concrete gates:

| Rung | Index | Name | Concrete Gates | What It Does | Cost |
|------|-------|------|----------------|-------------|------|
| 0 | `Compile` | Compile | `CompileGate` | Syntax/type checking (`cargo check`, `npm run build`, `go build`, `python -m py_compile`, `forge build`, `make`) | Cheap |
| 1 | `Lint` | Lint | `ClippyGate` | Static analysis (`cargo clippy -- -D warnings`) | Cheap |
| 2 | `Test` | Test | `TestGate` | Run existing test suite (`cargo test`, `npm test`, `go test`, `pytest`, `forge test`) | Medium |
| 3 | `Symbol` | Symbol | `SymbolGate` | Verify symbol resolution against a manifest -- all references resolve | Cheap |
| 4 | `GeneratedTest` | Generated Test | `GeneratedTestGate` + `VerifyChainGate` | LLM-generate tests for the change, then run them and verify the chain | Expensive |
| 5 | `PropertyTest` | Property Test | `PropertyTestGate` + `FactCheckGate` | Property-based tests (fuzzing) + fact-checking claims | Expensive |
| 6 | `Integration` | Integration | `LlmJudgeGate` + `IntegrationGate` | LLM-judged quality assessment + full integration/E2E test run | Very Expensive |

The rung enum is defined with a numeric representation that enforces canonical ordering:

```rust
// Source: crates/roko-gate/src/rung_selector.rs, lines 93-117
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
#[repr(u8)]
pub enum Rung {
    Compile = 0,
    Lint = 1,
    Test = 2,
    Symbol = 3,
    GeneratedTest = 4,
    PropertyTest = 5,
    Integration = 6,
}

// Source: crates/roko-gate/src/rung_selector.rs, lines 119-128
pub const CANONICAL_ORDER: [Rung; 7] = [
    Rung::Compile,
    Rung::Lint,
    Rung::Test,
    Rung::Symbol,
    Rung::GeneratedTest,
    Rung::PropertyTest,
    Rung::Integration,
];
```

The `#[repr(u8)]` and derived `Ord` guarantee that rungs execute in the correct order. The `#[non_exhaustive]` attribute allows future rung additions without breaking downstream matches.

Each rung also has a short display label for TUI rendering:

```rust
// Source: crates/roko-gate/src/rung_selector.rs, lines 160-173
impl Rung {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Compile => "compile",
            Self::Lint => "lint",
            Self::Test => "test",
            Self::Symbol => "symbol",
            Self::GeneratedTest => "gen-test",
            Self::PropertyTest => "prop-test",
            Self::Integration => "integration",
        }
    }
}
```

> **Source**: `crates/roko-gate/src/rung_selector.rs` lines 93-173 for the `Rung` enum, `CANONICAL_ORDER`, and `label()`.

### Standalone Gates (Beyond the 7 Rungs)

Six additional gates exist outside the rung pipeline for scenario-specific checks:

| Gate | Purpose | When Used |
|------|---------|-----------|
| `DiffGate` | Analyze git diff characteristics (lines added/removed, binary files) | Post-task review, cheap pre-filter |
| `CodeExecutionGate` | Sandboxed code execution | Verifying generated scripts |
| `ShellGate` | Arbitrary shell command, passes on exit code 0 | Custom verification commands |
| `BenchmarkRegressionGate` | Performance benchmark comparison | Detecting perf regressions |
| `FormatCheckGate` | Code formatting checks | Style enforcement |
| `SecurityScanGate` | Security vulnerability scanning | Security-critical changes |

Additionally, `GateGenerator` / `GeneratedCheck` support dynamically generated verification checks -- the system can synthesize verification steps at runtime.

> **Source**: `crates/roko-gate/src/lib.rs` lines 27-41 for standalone gate documentation.

---

## What Each Rung Catches

The progressive approach is better than single-pass verification because each rung catches a distinct class of error. Here are concrete examples of what goes wrong without each rung:

### Rung 0: Compile

**What it catches**: Syntax errors, type mismatches, missing imports, undefined variables.

**Example**: An agent renames a struct field from `name` to `label` but forgets to update one call site. The compile gate catches `error[E0609]: no field "name" on type "Config"` immediately, before any further verification is attempted.

**Without this rung**: All downstream rungs would fail with cascading compile errors, wasting time and producing noise instead of the single actionable diagnostic.

### Rung 1: Lint

**What it catches**: Code style violations, potential bugs flagged by static analysis, unsafe patterns, unnecessary clones, dead code.

**Example**: An agent writes `if x == true` instead of `if x`, or uses `.unwrap()` in production code, or creates an unused variable. Clippy catches these patterns with targeted warnings like `clippy::bool_comparison` or `clippy::disallowed_methods`.

**Without this rung**: Subtle code quality issues accumulate silently. The code works but becomes harder to maintain, and potential bugs (like unchecked `.unwrap()` calls) lurk until they cause runtime panics.

### Rung 2: Test

**What it catches**: Behavioral regressions, incorrect logic, violated invariants, broken contracts.

**Example**: An agent modifies a sorting function to fix a performance issue but accidentally breaks stability (equal elements no longer maintain their original order). The existing test suite catches `assertion failed: sorted[2] == sorted_stable[2]`.

**Without this rung**: The agent would consider its work done. The regression would only surface when a downstream consumer of the sorting function produces incorrect output -- possibly much later and much harder to diagnose.

### Rung 3: Symbol

**What it catches**: Dangling references, missing API implementations, incomplete interface contracts.

**Example**: An agent adds a new method to a trait but only implements it for 2 of 4 concrete types. The symbol gate detects that the remaining 2 types have unresolved method references.

**Without this rung**: In languages with dynamic dispatch or duck typing, missing implementations might not cause compile errors but would cause runtime crashes.

### Rung 4: Generated Test

**What it catches**: Edge cases the original test suite did not cover, behavioral gaps in new code, verification chain inconsistencies.

**Example**: An agent implements a new HTTP endpoint but the existing tests do not cover error responses. The LLM generates tests for 400/401/404/500 status codes and discovers the endpoint returns 500 instead of 404 for missing resources.

**Without this rung**: The original test suite passes because it only tests the happy path. The bug surfaces in production.

### Rung 5: Property Test

**What it catches**: Invariant violations that are statistically unlikely to appear in hand-written tests, fuzz-discovered crashes, factual inaccuracies in claims.

**Example**: An agent implements a custom serializer. Property-based testing generates random inputs and checks the round-trip property (`deserialize(serialize(x)) == x`). After 500 random inputs, it finds that Unicode surrogate pairs cause a panic.

**Without this rung**: Hand-written tests would use ASCII strings and miss the Unicode edge case entirely.

### Rung 6: Integration

**What it catches**: Cross-system interaction failures, environment-specific bugs, holistic quality issues that individual unit tests miss.

**Example**: An agent modifies a database migration and all unit tests pass, but the integration test discovers that the migration breaks when applied to a database with existing data because a NOT NULL column was added without a default value.

**Without this rung**: The migration would deploy successfully to an empty test database but fail catastrophically against production data.

---

## Complexity-Driven Rung Selection

Not every change needs all 7 rungs. The `select_rungs` function determines which rungs to execute based on three inputs:

### Plan Complexity

```rust
// Source: crates/roko-gate/src/rung_selector.rs, lines 23-34
pub enum PlanComplexity {
    /// Single-line / derive-only change. Compile only.
    Trivial,
    /// Small feature, few files. Adds linting.
    Simple,
    /// Normal plan. Adds the project's tests plus symbol checking.
    Standard,
    /// Large cross-crate work. Full rung suite.
    Complex,
}
```

### The Decision Table

| Complexity | Compile | Lint | Test | Symbol | GenTest | PropTest | Integration |
|-----------|---------|------|------|--------|---------|----------|-------------|
| **Trivial** | Y | | | | | | |
| **Simple** | Y | Y | | | | | |
| **Standard** | Y | Y | Y | Y | | | |
| **Complex** | Y | Y | Y | Y | Y | Y | Y |

This is implemented as a simple const function:

```rust
// Source: crates/roko-gate/src/rung_selector.rs, lines 234-249
const fn base_rungs(complexity: PlanComplexity) -> &'static [Rung] {
    match complexity {
        PlanComplexity::Trivial => &[Rung::Compile],
        PlanComplexity::Simple => &[Rung::Compile, Rung::Lint],
        PlanComplexity::Standard => &[Rung::Compile, Rung::Lint, Rung::Test, Rung::Symbol],
        PlanComplexity::Complex => &[
            Rung::Compile,
            Rung::Lint,
            Rung::Test,
            Rung::Symbol,
            Rung::GeneratedTest,
            Rung::PropertyTest,
            Rung::Integration,
        ],
    }
}
```

### Capability Caps

Rung caps narrow the selection based on what the project actually supports. A cap can only **remove** a rung the complexity band selected; it can never **add** one the band did not select:

```rust
// Source: crates/roko-gate/src/rung_selector.rs, lines 181-220
pub struct RungCaps {
    pub has_lint_tool: bool,
    pub has_symbol_manifest: bool,
    pub has_generated_tests: bool,
    pub has_property_tests: bool,
    pub has_integration_scenario: bool,
}

impl RungCaps {
    const fn allows(&self, rung: Rung) -> bool {
        match rung {
            Rung::Compile | Rung::Test => true, // always available
            Rung::Lint => self.has_lint_tool,
            Rung::Symbol => self.has_symbol_manifest,
            Rung::GeneratedTest => self.has_generated_tests,
            Rung::PropertyTest => self.has_property_tests,
            Rung::Integration => self.has_integration_scenario,
        }
    }
}
```

Note that `Compile` and `Test` are always available -- they cannot be capped out.

### Failure Escalation Ladder

Prior failures automatically escalate the effective complexity. Each prior failure moves the complexity one tier toward `Complex`:

```rust
// Source: crates/roko-gate/src/rung_selector.rs, lines 267-274
pub fn select_rungs(complexity: PlanComplexity, caps: &RungCaps, prior_failures: u32) -> Vec<Rung> {
    let effective = complexity.escalate_by(prior_failures);
    base_rungs(effective)
        .iter()
        .copied()
        .filter(|r| caps.allows(*r))
        .collect()
}
```

The escalation works like this:

| Base Complexity | 0 failures | 1 failure | 2 failures | 3+ failures |
|----------------|------------|-----------|------------|-------------|
| **Trivial** | Trivial | Simple | Standard | Complex |
| **Simple** | Simple | Standard | Complex | Complex |
| **Standard** | Standard | Complex | Complex | Complex |
| **Complex** | Complex | Complex | Complex | Complex |

This is the "escalation ladder" -- repeated failures trigger progressively more thorough verification. A trivial change that fails compile once gets linting added. Two failures add the full test suite. Three failures invoke the entire 7-rung pipeline.

> **Source**: `crates/roko-gate/src/rung_selector.rs` -- the entire file defines the selection logic, with comprehensive tests at lines 287-560.

---

## Rung Dispatch: From Enum to Concrete Gates

The `rung_dispatch` module maps each `Rung` enum variant to the concrete gate(s) that execute it. This is the runtime layer that actually shells out to compilers, test runners, and LLM judges.

```rust
// Source: crates/roko-gate/src/rung_dispatch.rs, lines 244-288
pub async fn run_canonical_rung(
    base_signal: &Signal, ctx: &Context, rung: Rung,
    inputs: &RungExecutionInputs, config: &RungExecutionConfig,
) -> Vec<Verdict> {
    match rung {
        Rung::Compile => {
            let mut gate = CompileGate::cargo();
            if let Some(timeout_ms) = gate_timeout_ms(config) {
                gate = gate.with_timeout_ms(timeout_ms);
            }
            vec![gate.verify(base_signal, ctx).await]
        }
        Rung::Lint => {
            let mut gate = ClippyGate::cargo();
            if let Some(timeout_ms) = gate_timeout_ms(config) {
                gate = gate.with_timeout_ms(timeout_ms);
            }
            vec![gate.verify(base_signal, ctx).await]
        }
        Rung::Test => {
            let mut gate = TestGate::cargo();
            if let Some(timeout_ms) = gate_timeout_ms(config) {
                gate = gate.with_timeout_ms(timeout_ms);
            }
            vec![gate.verify(base_signal, ctx).await]
        }
        Rung::Symbol => vec![run_symbol_gate(ctx, inputs, config).await],
        Rung::GeneratedTest => vec![
            run_generated_test_gate(base_signal, ctx, config).await,
            run_verify_chain_gate(base_signal, ctx, config).await,
        ],
        Rung::PropertyTest => vec![
            run_property_test_gate(base_signal, ctx, config).await,
            run_fact_check_gate(ctx, inputs, config).await,
        ],
        Rung::Integration => vec![
            run_llm_judge_gate(ctx, inputs, config).await,
            run_integration_gate(base_signal, ctx, config).await,
        ],
    }
}
```

Note that rungs 4, 5, and 6 each dispatch to **two** concrete gates. This is intentional -- these higher rungs combine multiple verification strategies:

- **Rung 4**: Generate behavioral tests AND verify the chain of evidence
- **Rung 5**: Run property-based tests AND fact-check claims
- **Rung 6**: LLM-judge quality assessment AND run integration tests

Default timeouts are tuned per rung:

| Rung | Default Timeout | Constant |
|------|----------------|----------|
| Compile | 600s (10 min) | `DEFAULT_COMPILE_TIMEOUT_SECS` |
| Lint | 300s (5 min) | `DEFAULT_LINT_TIMEOUT_SECS` |
| Test | 900s (15 min) | `DEFAULT_TEST_TIMEOUT_SECS` |
| Symbol | 120s (2 min) | `DEFAULT_SYMBOL_TIMEOUT_SECS` |
| Generated Test | 900s (15 min) | `DEFAULT_GENERATED_TEST_TIMEOUT_SECS` |
| Verify Chain | 1200s (20 min) | `DEFAULT_VERIFY_CHAIN_TIMEOUT_SECS` |
| Property Test | 900s (15 min) | `DEFAULT_PROPERTY_TEST_TIMEOUT_SECS` |
| Fact Check | 120s (2 min) | `DEFAULT_FACT_CHECK_TIMEOUT_SECS` |
| LLM Judge | 120s (2 min) | `DEFAULT_LLM_JUDGE_TIMEOUT_SECS` |
| Integration | 120s (2 min) | `DEFAULT_INTEGRATION_TIMEOUT_SECS` |

Gates that exceed their timeout return `Verdict::fail` with a timeout reason.

The `GatePipelineBuilder` provides a higher-level API that combines rung selection with config:

```rust
// Source: crates/roko-gate/src/rung_dispatch.rs, lines 88-101
pub struct GatePipelineBuilder;

impl GatePipelineBuilder {
    pub fn from_config(config: &GatesConfig, complexity: PlanComplexity) -> ComposedGatePipeline {
        Self::from_config_with_execution(
            config,
            complexity,
            RungExecutionInputs::default(),
            RungExecutionConfig::default(),
        )
    }
}
```

> **Source**: `crates/roko-gate/src/rung_dispatch.rs`, entire file. Default timeouts at lines 33-42, dispatch at lines 244-288.

---

## The GatePipeline Orchestrator

`GatePipeline` is itself a `Verify` implementation that runs a sequence of inner gates. This is the "ask every gate in order" orchestration layer.

```rust
// Source: crates/roko-gate/src/gate_pipeline.rs
pub struct GatePipeline {
    gates: Vec<Box<dyn Verify>>,
    short_circuit: bool,
    name: String,
}
```

Key behaviors:

**Short-circuiting (default: enabled).** When a gate fails, the pipeline stops and records remaining gates as `[skip]`. This is critical for convergence loops -- compile failures should short-circuit before test gates launch.

**No short-circuit mode.** With `without_short_circuit()`, every inner gate executes regardless of failures. The aggregate verdict records ALL failures. This is useful for getting a complete picture of what is broken.

**Test count aggregation.** When inner gates report `test_count` (passed/failed/ignored), the pipeline aggregates them across all executed gates, so downstream policies see cumulative figures.

**Detail transcript.** Every step is recorded in a bullet-list detail string:

```
GatePipeline 'full' -- 3/3 executed, short_circuit=true
1. [pass] compile (123 ms)
2. [pass] lint (45 ms)
3. [fail] test (890 ms) -- 2 tests failed
```

The `ComposedGatePipeline` extends this with configurable composition modes:

```rust
// Source: crates/roko-gate/src/gate_pipeline.rs
pub enum GateComposition {
    /// Run gates in push order; short-circuit on first failure (default).
    Sequential,
    /// Run the specified gate indices concurrently, collect all verdicts.
    Parallel(Vec<usize>),
    /// Aggregate verdicts from all gates; pass if more than threshold fraction agree.
    Voting { threshold: f64 },
    /// Try the first gate set; if it fails, try the fallback set.
    Fallback(Vec<usize>),
}
```

> **Source**: `crates/roko-gate/src/gate_pipeline.rs`, entire file. The `GatePipeline` struct, `GateComposition`, and `ComposedGatePipeline`.

---

## Gate Composition: Parallel, Voting, Fallback

Beyond the sequential pipeline, roko-gate provides three standalone composition wrappers. Each wraps inner gates and itself implements `Verify`, enabling algebraic composition of verification pipelines.

### ParallelGate

Runs N gates concurrently. ALL must pass for the aggregate to pass. The aggregate score is the minimum of all inner scores.

```rust
// Source: crates/roko-gate/src/composition.rs
pub struct ParallelGate {
    gates: Vec<Box<dyn Verify>>,
    name: String,
}
```

Use case: When inner gates are independent and can safely run simultaneously. For example, `CompileGate` and `FormatCheckGate` have no dependency on each other.

### VotingGate

Runs M gates and requires N-of-M to pass. The aggregate score is the mean of passing verdicts' scores.

```rust
// Source: crates/roko-gate/src/composition.rs
pub struct VotingGate {
    gates: Vec<Box<dyn Verify>>,
    required_passes: usize,
    name: String,
}
```

Use case: When multiple reviewers or verification strategies should agree. For example, 2-of-3 LLM judge gates must agree that code quality is acceptable.

### FallbackGate

Tries a primary gate first. If it fails, tries a fallback. The first passing verdict wins.

```rust
// Source: crates/roko-gate/src/composition.rs
pub struct FallbackGate {
    primary: Box<dyn Verify>,
    fallback: Box<dyn Verify>,
    name: String,
}
```

Use case: When you want to try a fast/cheap check first and fall back to a more thorough one on failure. For example, try a quick compilation check; if it fails due to missing dependencies, fall back to a full build with dependency resolution.

These composition wrappers can be nested arbitrarily because each implements `Verify`:

```rust
// Parallel of [Compile, Lint] fed into a FallbackGate
let fast = ParallelGate::new("fast")
    .with_gate(Box::new(CompileGate::cargo()))
    .with_gate(Box::new(ClippyGate::cargo()));

let slow = GatePipeline::new("slow")
    .with_gate(Box::new(CompileGate::cargo()))
    .with_gate(Box::new(TestGate::cargo()));

let gate = FallbackGate::new("adaptive", Box::new(fast), Box::new(slow));
```

> **Source**: `crates/roko-gate/src/composition.rs`, entire file. `ParallelGate`, `VotingGate`, `FallbackGate`.

---

## Adaptive Thresholds

The `AdaptiveThresholds` system tracks per-rung pass rates using an exponential moving average (EMA) and uses them to make three runtime decisions: how many retries to allow, whether to skip a rung, and whether a distributional shift has occurred.

### Core Mechanism

```rust
// Source: crates/roko-gate/src/adaptive_threshold.rs, lines 168-195
pub struct AdaptiveThresholds {
    rungs: HashMap<u32, RungStats>,
    cusum_sensitivity: f64,        // default: 0.25
    cusum_threshold: f64,          // default: 4.0
    spc_detectors: HashMap<u32, SpcDetector>,
    hotelling: Option<HotellingDetector>,
    pending_spc_alerts: Vec<(u32, SpcAlert)>,
    joint_anomaly_detected: bool,
}
```

Each rung tracks `RungStats`:

```rust
// Source: crates/roko-gate/src/adaptive_threshold.rs, lines 27-43
pub struct RungStats {
    pub ema_pass_rate: f64,         // Exponential moving average [0.0, 1.0]
    pub total_observations: u64,    // Total gate runs for this rung
    pub consecutive_passes: u32,    // Reset on any failure
    pub cusum_high: f64,            // Upward shift accumulator
    pub cusum_low: f64,             // Downward shift accumulator
    pub cusum_shift_detected: bool, // Was a shift detected on last observation?
}
```

### EMA Pass Rate Update

The EMA update uses a decay factor of alpha = 0.1, meaning recent observations weigh more heavily:

```
// For each gate observation:
if first_observation:
    ema_pass_rate = value  // (1.0 for pass, 0.0 for fail)
else:
    ema_pass_rate = 0.1 * value + 0.9 * ema_pass_rate
```

This is implemented directly:

```rust
// Source: crates/roko-gate/src/adaptive_threshold.rs, lines 322-373
pub fn observe(&mut self, rung: u32, passed: bool) {
    let stats = self.rungs.entry(rung).or_default();
    let value = if passed { 1.0 } else { 0.0 };

    if stats.total_observations == 0 {
        stats.ema_pass_rate = value;
    } else {
        stats.ema_pass_rate = EMA_ALPHA.mul_add(value, (1.0 - EMA_ALPHA) * stats.ema_pass_rate);
    }

    stats.total_observations += 1;

    if passed {
        stats.consecutive_passes += 1;
    } else {
        stats.consecutive_passes = 0;
    }

    // CUSUM change detection and SPC detector ensemble follow...
}
```

### Retry Budget Suggestion

The adaptive threshold suggests a retry count inversely proportional to the pass rate:

- **100% pass rate** -> 1 retry (the gate almost always passes)
- **0% pass rate** -> 5 retries (the gate always fails, give it more chances)
- **Unknown rung** (< 5 observations) -> 3 retries (default)

```rust
// Source: crates/roko-gate/src/adaptive_threshold.rs, lines 384-401
pub fn suggested_max_retries(&self, rung: u32) -> u32 {
    let Some(stats) = self.rungs.get(&rung) else { return 3; };
    if stats.total_observations < 5 { return 3; }
    let max_f = f64::from(MAX_RETRIES);      // 5.0
    let range_f = f64::from(MAX_RETRIES - MIN_RETRIES); // 4.0
    let retries = stats.ema_pass_rate.mul_add(-range_f, max_f).round() as u32;
    retries.clamp(MIN_RETRIES, MAX_RETRIES)   // clamp(1, 5)
}
```

### Skip Advisory

A rung that has passed consecutively at least 20 times is flagged as skippable:

```rust
pub fn should_skip_rung(&self, rung: u32) -> bool {
    self.rungs.get(&rung)
        .is_some_and(|s| s.consecutive_passes >= 20)
}
```

This is advisory -- the caller should still run the rung periodically to maintain the baseline.

### Temperament Modulation

Agent temperament adjusts threshold behavior:

| Temperament | Threshold | Retries | Skip |
|-------------|-----------|---------|------|
| **Conservative** | +10% stricter | Max 3 | Never skip |
| **Balanced** | No change | Default | Default streak |
| **Aggressive** | -15% more lenient | Min 2, max 5 | Skip at 10 passes |
| **Exploratory** | -10% more lenient | Same as aggressive | Default streak |

### Domain Profiles

Pre-built threshold profiles provide domain-specific priors:

```rust
// Source: crates/roko-gate/src/adaptive_threshold.rs, lines 93-141
pub fn coding() -> ThresholdProfile {
    // compile: 0.90, clippy: 0.80, test: 0.65, diff: 0.50
}

pub fn research() -> ThresholdProfile {
    // compile: 0.70, clippy: 0.60, test: 0.85, diff: 0.40
}

pub fn security() -> ThresholdProfile {
    // compile: 0.95, clippy: 0.90, test: 0.90, diff: 0.80
}
```

### Persistence

Thresholds are serialized to JSON and support atomic save/load:

```rust
// Source: crates/roko-gate/src/adaptive_threshold.rs, lines 257-287
pub fn save(&self, path: &Path) -> Result<(), io::Error> { ... }
pub fn load_or_new(path: &Path) -> Self { ... }
```

> **Source**: `crates/roko-gate/src/adaptive_threshold.rs`, entire file.

---

## Statistical Process Control (SPC)

The SPC system provides three complementary change-detection algorithms that run in parallel on every gate observation. Any detector that fires produces an `SpcAlert` that the orchestrator can react to.

### CUSUM (Cumulative Sum) Detector

**Purpose**: Detect sustained, gradual shifts in gate pass rates. Good for catching slow degradation that EMA smoothing might absorb.

**Origin**: The CUSUM chart was introduced by E. S. Page in 1954 [1]. It was designed to detect small, sustained shifts in a process mean more quickly than the standard Shewhart control chart.

**Mathematical formulation**:

The CUSUM detector maintains two one-sided statistics:

```
Upper CUSUM (detects upward shifts):
    S_upper(n) = max(0, S_upper(n-1) + x_n - mu_0 - k)

Lower CUSUM (detects downward shifts):
    S_lower(n) = max(0, S_lower(n-1) + mu_0 - x_n - k)
```

Where:
- `x_n` is the current observation (1.0 for pass, 0.0 for fail)
- `mu_0` is the in-control (target) mean (e.g., 0.85 expected pass rate)
- `k` is the drift allowance (slack parameter) -- typically half the minimum shift to detect (`sigma / 2`)
- An alarm fires when either `S_upper > h` or `S_lower > h` (the decision threshold `h`)
- After an alarm, the corresponding accumulator is reset to 0

**Tuning parameters**:
- `h` (threshold): Larger values reduce false alarms but delay detection. Typical range: 2.0 to 8.0. Default in roko: 5.0.
- `k` (drift): Smaller values detect smaller shifts sooner. Typical value: `sigma / 2` where sigma is the process standard deviation.

**Implementation**:

```rust
// Source: crates/roko-gate/src/spc.rs, lines 34-96
pub struct CusumDetector {
    pub target: f64,       // In-control mean (e.g., 0.85)
    pub threshold_h: f64,  // Decision threshold (e.g., 5.0)
    pub drift_k: f64,      // Allowance (e.g., sigma/2 = 0.05)
    cumsum_upper: f64,     // Upper one-sided accumulator
    cumsum_lower: f64,     // Lower one-sided accumulator
    observations: usize,
}

impl CusumDetector {
    pub fn update(&mut self, observation: f64) -> Option<CusumShift> {
        self.observations += 1;

        // Upper CUSUM: detects upward shift
        self.cumsum_upper =
            (self.cumsum_upper + observation - self.target - self.drift_k).max(0.0);
        // Lower CUSUM: detects downward shift
        self.cumsum_lower =
            (self.cumsum_lower + self.target - observation - self.drift_k).max(0.0);

        if self.cumsum_upper > self.threshold_h {
            self.cumsum_upper = 0.0; // Reset after alarm
            return Some(CusumShift::Upward);
        }
        if self.cumsum_lower > self.threshold_h {
            self.cumsum_lower = 0.0; // Reset after alarm
            return Some(CusumShift::Downward);
        }

        None
    }
}
```

**Worked example**: Suppose a gate has a target pass rate of 0.85 and starts experiencing failures (actual rate drops to 0.0 on each failure). With `k = 0.05` and `h = 5.0`, each failure observation adds approximately `0.85 - 0.0 - 0.05 = 0.80` to the lower CUSUM accumulator. After 7 consecutive failures, the accumulator reaches `7 * 0.80 = 5.60 > h = 5.0`, triggering a `CusumShift::Downward` alarm.

> **Source**: `crates/roko-gate/src/spc.rs` lines 22-122 for CUSUM.

### EWMA (Exponentially Weighted Moving Average) Control Chart

**Purpose**: Detect small sustained shifts in gate pass rates with formal statistical control limits. More sensitive to small shifts than standard Shewhart charts because exponential weighting carries memory of recent observations.

**Origin**: The EWMA control chart was introduced by Roberts in 1959 [2]. It was originally called a "geometric moving average chart" and provides faster detection of small shifts than the Shewhart chart.

**Mathematical formulation**:

```
EWMA statistic:
    Z_n = lambda * x_n + (1 - lambda) * Z_{n-1}

Control limits (asymptotic form):
    UCL = mu_0 + L * sigma * sqrt(lambda / (2 - lambda))
    LCL = mu_0 - L * sigma * sqrt(lambda / (2 - lambda))

Warning limits:
    Warning_UCL = mu_0 + (2/3) * L * sigma * sqrt(lambda / (2 - lambda))
    Warning_LCL = mu_0 - (2/3) * L * sigma * sqrt(lambda / (2 - lambda))
```

Where:
- `lambda` (smoothing factor): Controls how much weight recent observations receive. Smaller = more smoothing, better for detecting small shifts. Typical: 0.2.
- `sigma`: Estimated process standard deviation.
- `L`: Control limit multiplier (number of sigma). Typical: 3.0 (for 3-sigma limits).
- `mu_0`: Target (in-control) mean.

Note: The implementation uses the simpler asymptotic form `sqrt(lambda / (2 - lambda))` rather than the exact time-varying form `sqrt((lambda / (2 - lambda)) * (1 - (1 - lambda)^(2n)))`. For processes with more than ~20 observations, the asymptotic form closely approximates the exact formula.

**Three control statuses**:

```rust
// Source: crates/roko-gate/src/spc.rs, lines 127-135
pub enum ControlStatus {
    InControl,      // Within control limits
    Warning,        // Between 2-sigma and 3-sigma -- potential issue developing
    OutOfControl,   // Beyond 3-sigma -- process is out of control
}
```

**Implementation**:

```rust
// Source: crates/roko-gate/src/spc.rs, lines 149-203
pub struct EwmaControlChart {
    lambda: f64,           // Smoothing factor (0.01, 1.0]
    sigma: f64,            // Process standard deviation
    control_limit_l: f64,  // Sigma multiplier for limits
    ewma: f64,             // Current EWMA value
    target: f64,           // In-control mean
    observations: usize,
}

impl EwmaControlChart {
    pub fn update(&mut self, observation: f64) -> ControlStatus {
        self.observations += 1;
        self.ewma = self.lambda * observation + (1.0 - self.lambda) * self.ewma;

        let limit_factor = self.sigma * (self.lambda / (2.0 - self.lambda)).sqrt();
        let ucl = self.target + self.control_limit_l * limit_factor;
        let lcl = self.target - self.control_limit_l * limit_factor;
        let warning_ucl = self.target + (self.control_limit_l * 2.0 / 3.0) * limit_factor;
        let warning_lcl = self.target - (self.control_limit_l * 2.0 / 3.0) * limit_factor;

        if self.ewma > ucl || self.ewma < lcl {
            ControlStatus::OutOfControl
        } else if self.ewma > warning_ucl || self.ewma < warning_lcl {
            ControlStatus::Warning
        } else {
            ControlStatus::InControl
        }
    }
}
```

**Worked example**: With `target = 0.85`, `sigma = 0.05`, `lambda = 0.2`, `L = 3.0`, the asymptotic control limits are:
- `limit_factor = 0.05 * sqrt(0.2 / 1.8) = 0.05 * 0.3333 = 0.01667`
- `UCL = 0.85 + 3.0 * 0.01667 = 0.90`
- `LCL = 0.85 - 3.0 * 0.01667 = 0.80`

If the EWMA drops below 0.80, the chart signals `OutOfControl`.

> **Source**: `crates/roko-gate/src/spc.rs` lines 124-236 for EWMA.

### BOCPD (Bayesian Online Change Point Detection)

**Purpose**: Detect abrupt regime changes (e.g., a model update causes sudden behavior shift, a major refactor changes the test pass pattern). Unlike CUSUM and EWMA which look for gradual shifts, BOCPD maintains a full posterior distribution over run lengths and signals when the probability of a recent change point exceeds a threshold.

**Origin**: BOCPD was introduced by Adams and MacKay in 2007 [3]. It provides an exact, computationally efficient Bayesian method for detecting change points in an online setting.

**Mathematical formulation**:

BOCPD maintains a run-length distribution `P(r_t | x_{1:t})` where `r_t` is the "time since the last change point."

At each new observation `x_t`:

1. **Compute predictive probabilities** for each possible run length:
   ```
   pi(r) = P(x_t | x_{r:t-1}) -- Gaussian predictive using conjugate prior
   ```

2. **Compute growth probabilities** (extend existing runs):
   ```
   P_growth(r+1) = P(r) * pi(r) * (1 - H)
   ```
   where `H` is the hazard rate (prior probability of a change point at each step).

3. **Compute change point probability** (new run starts):
   ```
   P_change = sum_r[ P(r) * pi(r) * H ]
   ```

4. **Assemble and normalize** the new run-length distribution.

5. **Check threshold**: If `P(r_t = 0)` > change_threshold, a change point is detected.

The Gaussian predictive probability uses a conjugate normal model:

```rust
// Source: crates/roko-gate/src/spc.rs, lines 403-426
fn gaussian_predictive(&self, count: usize, sum: f64, sum_sq: f64, observation: f64) -> f64 {
    let n = count as f64;
    let mean = if n > 0.0 {
        (self.prior_var * sum + self.prior_mean) / (n * self.prior_var + 1.0)
    } else {
        self.prior_mean
    };
    let var = if n > 0.0 {
        let sample_var = if n > 1.0 {
            (sum_sq - sum * sum / n) / (n - 1.0)
        } else { self.prior_var };
        sample_var / n + self.prior_var
    } else { self.prior_var };
    let var = var.max(0.001);

    // Gaussian PDF
    let diff = observation - mean;
    (-0.5 * diff * diff / var).exp() / (2.0 * std::f64::consts::PI * var).sqrt()
}
```

**Parameters**:
- `hazard_rate`: Prior probability of a change at each step. Typical: 0.01 (expect a change every ~100 observations).
- `change_threshold`: Posterior probability to trigger alarm. Typical: 0.5.
- `prior_mean`, `prior_var`: Parameters for the Gaussian conjugate prior.

Memory is bounded by trimming run lengths with negligible probability (`< 1e-8`) from the tail.

> **Source**: `crates/roko-gate/src/spc.rs` lines 238-444 for BOCPD.

### Composite SPC Detector

All three detectors are combined in the `SpcDetector` struct, which runs them in parallel on each observation:

```rust
// Source: crates/roko-gate/src/spc.rs, lines 472-525
pub struct SpcDetector {
    pub cusum: CusumDetector,
    pub ewma_chart: EwmaControlChart,
    pub bocpd: BocpdDetector,
}

impl SpcDetector {
    pub fn new(target: f64, sigma: f64) -> Self {
        Self {
            cusum: CusumDetector::new(target, 5.0, sigma / 2.0),
            ewma_chart: EwmaControlChart::new(target, sigma, 0.2, 3.0),
            bocpd: BocpdDetector::new(0.01, 0.5, target, sigma * sigma),
        }
    }

    pub fn update(&mut self, observation: f64) -> Vec<SpcAlert> {
        let mut alerts = Vec::new();
        if let Some(shift) = self.cusum.update(observation) {
            alerts.push(SpcAlert::CusumShift(shift));
        }
        match self.ewma_chart.update(observation) {
            ControlStatus::OutOfControl => alerts.push(SpcAlert::EwmaOutOfControl {
                ewma_value: self.ewma_chart.current(),
            }),
            ControlStatus::Warning => alerts.push(SpcAlert::EwmaWarning {
                ewma_value: self.ewma_chart.current(),
            }),
            ControlStatus::InControl => {}
        }
        if let Some(cp) = self.bocpd.update(observation) {
            alerts.push(SpcAlert::ChangePoint(cp));
        }
        alerts
    }
}
```

The three detectors catch complementary patterns:

| Detector | Best For | Sensitivity | Memory | Reference |
|----------|----------|-------------|--------|-----------|
| **CUSUM** | Gradual, sustained shifts | Configurable via k/h | O(1) | Page (1954) [1] |
| **EWMA** | Small persistent shifts | High (formal control limits) | O(1) | Roberts (1959) [2] |
| **BOCPD** | Abrupt regime changes | Threshold-based | O(n), bounded by trimming | Adams & MacKay (2007) [3] |

> **Source**: `crates/roko-gate/src/spc.rs` lines 446-533 for the composite detector.

---

## Process Reward Model

The Process Reward Model (PRM) tracks per-turn gate snapshots and derives two cybernetic signals for the orchestrator. This is inspired by Lightman et al. 2023 ("Let's Verify Step by Step" / PRM800K) [4] and the AgentPRM framework (arXiv:2502.10325) [5].

### Turn Snapshots

Each agent turn produces a snapshot:

```rust
// Source: crates/roko-gate/src/process_reward.rs, lines 19-29
pub struct TurnSnapshot {
    pub rung: u32,             // Highest rung reached at this turn
    pub verdicts: Vec<Verdict>, // All verdicts from the gate pipeline
    pub error_count: u32,       // Number of distinct errors in gate feedback
    pub diff_lines: u32,        // Number of lines changed in the diff
}
```

### Promise Score

**Promise** predicts the probability of eventual task success given the current trajectory. It is a weighted combination of three signals:

```rust
// Source: crates/roko-gate/src/process_reward.rs, lines 94-108
pub fn promise(&self) -> f64 {
    if self.history.is_empty() {
        return 0.5; // no data => neutral prior
    }

    let pass_rate = self.historical_pass_rate();
    let progression_rate = self.ratchet_progression_rate();
    let convergence = self.diff_convergence();

    // Weighted combination:
    // pass_rate (50%): most important signal
    // progression (30%): are we advancing through rungs?
    // convergence (20%): are diffs getting smaller?
    let raw = pass_rate * 0.5 + progression_rate * 0.3 + convergence * 0.2;
    raw.clamp(0.0, 1.0)
}
```

The three components:

1. **Historical pass rate** (weight: 0.5): Fraction of all verdicts across all turns that passed. This is the strongest signal of eventual success.

2. **Ratchet progression rate** (weight: 0.3): Measures whether we are advancing through rungs over time. Computed as:
   ```
   progression = 0.5 + (last_rung - first_rung) / (2 * max_rung_seen)
   ```
   A value > 0.5 means we are advancing; < 0.5 means we are regressing.

3. **Diff convergence** (weight: 0.2): Are the diffs getting smaller over time? Shrinking diffs suggest the agent is converging on a solution:
   ```
   If diffs are shrinking (last <= first):
       0.5 + (1 - last_diff/first_diff) * 0.5
   If diffs are growing (last > first):
       (first_diff/last_diff) * 0.5
   ```

### Progress Score

**Progress** measures the trajectory delta between the two most recent turns. It ranges from -1.0 (severe regression) to +1.0 (major improvement).

```rust
// Source: crates/roko-gate/src/process_reward.rs, lines 114-144
pub fn progress(&self) -> f64 {
    if self.history.len() < 2 { return 0.0; }

    let prev = &self.history[self.history.len() - 2];
    let curr = &self.history[self.history.len() - 1];

    let mut delta = 0.0;

    // Rung advancement: +0.4 per rung gained, -0.4 per rung lost (clamped)
    let rung_delta = curr.rung as f64 - prev.rung as f64;
    delta += (rung_delta * 0.4).clamp(-0.4, 0.4);

    // Error reduction: going from 5 errors to 0 is +0.3
    if prev.error_count > 0 {
        let error_reduction =
            (prev.error_count as f64 - curr.error_count as f64) / prev.error_count as f64;
        delta += error_reduction * 0.3;
    } else if curr.error_count == 0 {
        delta += 0.3; // still clean
    }

    // Pass rate improvement between turns
    let prev_pass_rate = turn_pass_rate(prev);
    let curr_pass_rate = turn_pass_rate(curr);
    delta += (curr_pass_rate - prev_pass_rate) * 0.3;

    delta.clamp(-1.0, 1.0)
}
```

### Early Termination

The PRM enables early termination: if promise drops below a minimum threshold and we have at least 2 turns of data, abandon the task:

```rust
pub fn should_terminate(&self, min_promise: f64) -> bool {
    if self.history.len() < 2 { return false; }
    self.promise() < min_promise
}
```

### Step Verification

The PRM also scores individual reasoning steps using heuristic analysis:

```rust
// Source: crates/roko-gate/src/process_reward.rs, lines 161-192
pub fn verify_steps(&self, steps: &[ReasoningStep]) -> StepVerdict {
    let step_scores: Vec<f64> = steps.iter().map(|s| score_step(s)).collect();
    let aggregate_score = match self.aggregate {
        AggregateMethod::Min => step_scores.iter().copied().fold(f64::INFINITY, f64::min),
        AggregateMethod::Mean => step_scores.iter().sum::<f64>() / step_scores.len() as f64,
        AggregateMethod::Weighted => {
            // Later steps receive linearly increasing weight
            // weight(i) = i+1, total_weight = n*(n+1)/2
            let n = step_scores.len() as f64;
            let total_weight = n * (n + 1.0) / 2.0;
            step_scores.iter().enumerate()
                .map(|(i, &s)| s * (i as f64 + 1.0))
                .sum::<f64>() / total_weight
        }
    };
    StepVerdict { passed: aggregate_score >= self.step_threshold, step_scores, aggregate_score }
}
```

Step scoring considers:
- Content length (non-trivial steps score higher: >200 chars = 0.4, >50 = 0.3, >10 = 0.1)
- Presence of code blocks (``` or indentation = +0.3)
- Presence of assertion/verification keywords (assert, verify, check, test, ensure, confirm = +0.3)

> **Source**: `crates/roko-gate/src/process_reward.rs`, entire file. References: Lightman et al. 2023 [4], AgentPRM (arXiv:2502.10325) [5], cited in the module doc comment at line 13.

---

## Gate Ratchet: Preventing Rung Regression

The `GateRatchet` prevents rung regression during convergence loops. Once a plan has passed rung N, it should never be allowed to regress to rung N-1.

**Why this matters**: Without a ratchet, an agent can thrash in a convergence loop: fix the compile error but break lint, then fix lint but break compile again. The ratchet makes the second regression visible and blockable.

```rust
// Source: crates/roko-gate/src/ratchet.rs, lines 19-109
pub struct GateRatchet {
    passes: HashMap<String, u8>,  // plan_id -> highest rung passed
}

impl GateRatchet {
    /// Record that `plan_id` passed `rung`.
    /// Only updates if rung is HIGHER than previously recorded.
    pub fn record_pass(&mut self, plan_id: impl Into<String>, rung: u8) {
        let key = plan_id.into();
        let entry = self.passes.entry(key).or_insert(0);
        if rung > *entry { *entry = rung; }
    }

    /// Returns false if accepting `rung` as highest would be a regression.
    pub fn can_regress(&self, plan_id: &str, rung: u8) -> bool {
        match self.passes.get(plan_id) {
            None => true,                 // No history, no regression possible
            Some(&highest) => rung >= highest, // Must be at or above highest
        }
    }
}
```

The ratchet supports JSON persistence with atomic save/load:

```rust
pub fn save(&self, path: &Path) -> Result<(), io::Error> { ... }
pub fn load_or_new(path: &Path) -> Self { ... }
```

Example flow:
```
Turn 1: agent passes compile (rung 0), lint (rung 1), test (rung 2)
         ratchet records: plan-1 -> 2

Turn 2: agent's change passes compile but breaks lint
         ratchet.can_regress("plan-1", 1) -> false!
         Orchestrator blocks the regression and forces a retry.
```

> **Source**: `crates/roko-gate/src/ratchet.rs`, entire file.

---

## Forensic Causal Chain Reconstruction

The forensic system reconstructs the complete causal chain for any task: which agent produced which output, which gate verified it, what the verdict was, and what evidence (compiler output, test results, diff) supports the verdict.

### Hash-Addressed Artifact Store

All gate artifacts (build logs, test output, diff snapshots) are stored in a BLAKE3 content-addressed store:

```rust
// Source: crates/roko-gate/src/artifact_store.rs
pub struct ArtifactStore {
    inner: HashMap<ContentHash, Vec<u8>>,
    root: Option<PathBuf>,  // Optional disk-backed storage
}
```

Properties:
- **Deduplication**: Identical artifacts share storage (keyed by BLAKE3 hash)
- **Addressability**: Any subsystem can refer to an artifact by hash
- **Immutability**: Once stored, content never changes
- **Verification**: Content can be re-hashed to verify integrity

### Causal Chain

```rust
// Source: crates/roko-gate/src/forensic.rs, lines 47-61
pub struct CausalChain {
    pub task_id: String,
    pub agent_model: String,
    pub turns: Vec<TurnRecord>,
    pub verdicts: Vec<(Verdict, Option<ContentHash>)>,
    pub artifacts: Vec<ArtifactMetadata>,
    pub integrity_verified: bool,
}
```

Each turn record links the agent model, verdicts, and artifact hashes:

```rust
// Source: crates/roko-gate/src/forensic.rs, lines 34-44
pub struct TurnRecord {
    pub turn_index: usize,
    pub agent_model: String,
    pub verdicts: Vec<Verdict>,
    pub artifact_hashes: Vec<ContentHash>,
}
```

### Replay Builder

The `ForensicReplayBuilder` accumulates turn records and reconstructs causal chains with integrity verification:

```rust
// Source: crates/roko-gate/src/forensic.rs, lines 132-140
pub struct ForensicReplayBuilder {
    task_turns: HashMap<String, Vec<TurnRecord>>,
    task_verdicts: HashMap<String, Vec<(Verdict, Option<ContentHash>)>>,
    task_models: HashMap<String, String>,
}

impl ForensicReplayBuilder {
    pub fn record_turn(
        &mut self, task_id: &str, turn_index: usize, agent_model: &str,
        verdicts: Vec<Verdict>, artifact_hashes: Vec<ContentHash>,
    ) { ... }

    pub fn replay_task(
        &self, task_id: &str, artifact_store: &ArtifactStore,
    ) -> Result<CausalChain, ForensicError> {
        // Walk content-hash links, verify BLAKE3 chain integrity
        // ...
    }
}
```

The replay process:
1. Retrieve all turns for the task
2. For each artifact hash referenced by any turn, retrieve the content from the artifact store
3. Recompute the BLAKE3 hash and verify it matches the stored hash
4. Build the complete `CausalChain` with `integrity_verified = true` only if all hashes match

Example forensic chain output:
```
Task task-1: 3 turns, 5 verdicts (3 pass, 2 fail), 4 artifacts, integrity=true

Turn 0 (claude-sonnet):
  - [fail] compile -- error[E0308]: mismatched types
    artifact: 7a3b4c... (156 bytes, verified)
Turn 1 (claude-sonnet):
  - [pass] compile
  - [pass] lint
Turn 2 (claude-sonnet):
  - [pass] compile
  - [pass] lint
  - [fail] test -- assertion failed at line 42
    artifact: 9d2e1f... (892 bytes, verified)
```

> **Source**: `crates/roko-gate/src/forensic.rs` (entire file) and `crates/roko-gate/src/artifact_store.rs`.

---

## Acceptance Contracts

Acceptance contracts define what "done" means for a specific task. They are typed specifications that enumerate the evidence a task must produce before it can be marked complete. Missing or malformed evidence is a blocking validation issue -- callers fail closed.

### Contract Structure

```rust
// Source: crates/roko-gate/src/acceptance_contract.rs
pub struct AcceptanceContract {
    pub version: u32,                            // Schema version (only 1 accepted)
    pub gates: Vec<GateRequirement>,              // Compile/test/lint gates
    pub no_stub: Option<NoStubRequirement>,       // No stubs in production paths
    pub agent_output: Option<StructuredAgentOutputRequirement>,
    pub review_verdict: Option<ReviewVerdictRequirement>,
    pub recovery: Option<RecoveryRequirement>,     // Retry/reflection/replan signals
    pub parity_ledger: Option<ParityLedgerRequirement>,
}
```

### Outcome States

The acceptance contract defines 9 possible outcomes, far richer than binary pass/fail:

```rust
// Source: crates/roko-gate/src/acceptance_contract.rs
pub enum AcceptanceOutcome {
    Passed,       // All evidence present and passing
    Failed,       // Required evidence failed or malformed
    Blocked,      // Cannot proceed with current external state
    TimedOut,     // Gate exceeded time budget
    Cancelled,    // Run cancelled before terminal verdict
    NeedsRetry,   // Bounded retry of the same task
    NeedsReplan,  // Change the plan before retrying
    NeedsHuman,   // Requires human review or approval
    NeedsWork,    // Incomplete evidence, needs more implementation
}
```

> **Source**: `crates/roko-gate/src/acceptance_contract.rs`, entire file.

---

## Agent Feedback Filtering

Raw gate output (compiler stderr, test logs, linter JSON) is verbose and full of noise that wastes agent context tokens. The feedback system parses raw output into structured, filtered feedback containing only actionable items.

```rust
// Source: crates/roko-gate/src/feedback.rs, lines 52-64
pub struct GateFeedback {
    pub rung: u8,                    // Which rung produced this
    pub passed: bool,                // Did the gate pass?
    pub errors: Vec<String>,         // Must fix
    pub warnings: Vec<String>,       // Should fix
    pub suggestions: Vec<String>,    // Helpful context
}
```

The classifier strips noise (progress bars, download indicators, compilation progress) and extracts actionable diagnostics:

- **Error patterns**: `error[E`, `panicked at`, `FAILED`, `FAIL `
- **Warning patterns**: `warning:`, `WARNING:`, `warn[`
- **Suggestion patterns**: `help:`, `note:`, `hint:`, `--> ` (file locations)
- **Noise patterns**: `Compiling`, `Downloading`, `Finished`, progress bars

If no classified lines are found but the output is non-empty, the first non-noise line is surfaced as an actionable error to avoid silent failures.

> **Source**: `crates/roko-gate/src/feedback.rs`, entire file.

---

## Hotelling's T-Squared: Joint Anomaly Detection

When multiple gates shift together (e.g., compile AND lint AND test pass rates all drop simultaneously), this signals a systemic problem rather than a gate-specific issue. The Hotelling's T-squared detector is the multivariate extension of the t-test that detects these joint anomalies.

**Origin**: Hotelling introduced the T-squared statistic for multivariate quality control in 1947 [6], in a paper on air testing of sample bombsights. It remains the standard method for monitoring multivariate process means.

### Mathematical Formulation

```
T-squared = n * (x - mu)^T * S^{-1} * (x - mu)
```

Where:
- `x` is the current gate pass rate vector (e.g., `[0.2, 0.1, 0.3]` for compile/lint/test)
- `mu` is the historical mean vector (e.g., `[0.9, 0.85, 0.8]`)
- `S` is the covariance matrix (captures correlations between gates)
- `n` is the sample size
- The result is compared against a chi-squared critical value with `p` degrees of freedom (where `p` is the number of gates)

**Implementation**:

```rust
// Source: crates/roko-gate/src/hotelling.rs, lines 31-46
pub struct HotellingDetector {
    dimension: usize,       // Number of gates tracked
    mean: Vec<f64>,         // Running mean per gate
    covariance: Vec<f64>,   // Flattened p*p covariance matrix
    observations: usize,
    threshold: f64,         // Chi-squared critical value
    m2: Vec<f64>,           // Welford's M2 for online covariance
}

impl HotellingDetector {
    pub fn t_squared(&self, current: &[f64]) -> f64 {
        // d = current - mean
        // T^2 = n * d^T * S^{-1} * d
        // Where S^{-1} is computed via Gauss-Jordan elimination
    }

    pub fn is_anomalous(&self, current: &[f64]) -> bool {
        self.t_squared(current) > self.threshold
    }
}
```

The covariance matrix is updated incrementally using Welford's online algorithm [7] extended to multivariate data, avoiding the numerical instability of naive `sum(x^2) - n*mean^2` formulas. Welford's algorithm computes:

```
delta = x_i - mean_old
mean_new = mean_old + delta / n
delta2 = x_i - mean_new
M2[i][j] += delta[i] * delta2[j]
covariance = M2 / (n - 1)
```

The chi-squared critical value is approximated using the Wilson-Hilferty transformation [8]:

```
chi2_alpha_k ~= k * (1 - 2/(9k) + z_alpha * sqrt(2/(9k)))^3
```

where `z_alpha` is the standard normal quantile (computed via the Abramowitz and Stegun rational approximation formula 26.2.23).

This is integrated into `AdaptiveThresholds.observe_pipeline()`, which records a complete pipeline run as a vector and checks for joint anomalies.

> **Source**: `crates/roko-gate/src/hotelling.rs`, entire file. Wired into `AdaptiveThresholds` via the `observe_pipeline` method in `crates/roko-gate/src/adaptive_threshold.rs` lines 469-485.

---

## PELT: Offline Change Point Detection

The PELT (Pruned Exact Linear Time) algorithm provides offline batch change point detection, complementing the online detectors (CUSUM, EWMA, BOCPD).

**Origin**: PELT was introduced by Killick, Fearnhead, and Eckley in 2012 [9]. It achieves exact optimal segmentation with an average-case computational cost that is linear in the number of data points, through a pruning step that eliminates candidate change points that can never be optimal.

### Algorithm

PELT minimizes:
```
sum_i cost(y[cp_i..cp_{i+1}]) + beta * num_changepoints
```

Where `beta` is the penalty per change point (controls sensitivity). Uses dynamic programming with pruning for O(n) average complexity (O(n^2) worst case).

### Cost Functions

Three cost functions are supported:

```rust
// Source: crates/roko-gate/src/pelt.rs, lines 34-42
pub enum CostFunction {
    L2,     // Squared error from segment mean -- detects mean shifts
    L1,     // Absolute error from segment median -- robust to outliers
    Normal, // Normal log-likelihood -- detects changes in mean AND/OR variance
}
```

**L2 cost** (mean shift detection):
```
cost(segment) = sum_i (x_i - mean(segment))^2
```

**L1 cost** (median shift detection, outlier-robust):
```
cost(segment) = sum_i |x_i - median(segment)|
```

**Normal cost** (mean + variance detection):
```
cost(segment) = n * ln(variance(segment))
```

### Usage

```rust
use roko_gate::pelt::{PeltDetector, CostFunction};

let data = vec![1.0, 1.1, 0.9, 1.0, 5.0, 5.1, 4.9, 5.0];
let detector = PeltDetector::new(CostFunction::L2, 3.0);
let change_points = detector.detect(&data);
// change_points contains index ~4 (where values jump from ~1 to ~5)
```

> **Source**: `crates/roko-gate/src/pelt.rs`, entire file.

---

## IronClaw Integration Plan

This section provides a concrete, step-by-step plan for integrating the roko-gate verification pipeline into IronClaw.

### Phase 1: Foundation (1-2 days)

**Goal**: Add `roko-gate` as a dependency and create a thin adapter crate.

**Step 1.1: Add the dependency**

Add `roko-gate` and `roko-core` to the workspace `Cargo.toml`:

```toml
# Cargo.toml (workspace)
[workspace.dependencies]
roko-gate = { path = "../roko/crates/roko-gate" }
roko-core = { path = "../roko/crates/roko-core" }
```

**Step 1.2: Create `crates/ironclaw_gate/`**

Create a thin adapter crate that wraps roko-gate types and provides IronClaw-specific helpers:

```
crates/ironclaw_gate/
  Cargo.toml
  src/
    lib.rs          # Re-exports, IronClaw-specific wrappers
    complexity.rs   # Map IronClaw change characteristics to PlanComplexity
    feedback.rs     # Convert GateFeedback to IronClaw ToolOutput format
    persistence.rs  # Threshold/ratchet persistence using IronClaw's Database trait
```

**File: `crates/ironclaw_gate/src/lib.rs`**

```rust
//! Gate verification pipeline for IronClaw.
//!
//! Wraps `roko-gate` to provide progressive verification of AI-generated
//! code and tool builds. Start with rungs 0-2 (compile, lint, test) for
//! immediate value, then add adaptive thresholds and higher rungs over time.

pub use roko_gate::{
    // Core types
    AdaptiveThresholds, RungStats,
    GatePipeline, ComposedGatePipeline, GateComposition,
    ProcessRewardModel, TurnSnapshot, AggregateMethod,
    GateRatchet,
    // Gates
    CompileGate, ClippyGate, TestGate, ShellGate,
    // Composition
    ParallelGate, VotingGate, FallbackGate,
    // Selection
    PlanComplexity, Rung, RungCaps, select_rungs,
    // Feedback
    GateFeedback, feedback_for_agent,
    // Acceptance
    AcceptanceContract, AcceptanceDecision, AcceptanceEvidence, AcceptanceOutcome,
    GateRequirement, GateRequirementKind,
    // SPC
    SpcDetector, SpcAlert,
};

pub mod complexity;
pub mod feedback;
pub mod persistence;
```

**File: `crates/ironclaw_gate/src/complexity.rs`**

```rust
use roko_gate::PlanComplexity;
use std::path::Path;

/// Assess the complexity of a change based on file count, line count,
/// and whether cross-module boundaries are crossed.
pub fn assess_complexity(
    changed_files: &[&Path],
    total_lines_changed: usize,
) -> PlanComplexity {
    let file_count = changed_files.len();
    let crosses_modules = changed_files
        .iter()
        .map(|p| p.parent().unwrap_or(Path::new("")))
        .collect::<std::collections::HashSet<_>>()
        .len() > 2;

    match (file_count, total_lines_changed, crosses_modules) {
        (1, 0..=5, false) => PlanComplexity::Trivial,
        (1..=3, 0..=50, false) => PlanComplexity::Simple,
        (_, _, false) if total_lines_changed <= 200 => PlanComplexity::Standard,
        _ => PlanComplexity::Complex,
    }
}
```

### Phase 2: Tool Builder Validation (2-3 days)

**Goal**: Add progressive verification to WASM tool builds.

**File to modify**: `src/tools/builder/validation.rs`

Add a gate-based validation step that runs after the existing WASM validation:

```rust
use ironclaw_gate::{
    CompileGate, ClippyGate, TestGate, GatePipeline,
    PlanComplexity, RungCaps, select_rungs,
    AdaptiveThresholds, feedback_for_agent,
};
use roko_core::{Context, Engram};

/// Validate a WASM tool build with progressive rigor.
pub async fn validate_tool_build(
    working_dir: &Path,
    complexity: PlanComplexity,
    thresholds: &mut AdaptiveThresholds,
) -> Result<ValidationResult, ToolError> {
    // 1. Select rungs based on change complexity
    let caps = RungCaps {
        has_lint_tool: true,
        has_symbol_manifest: false,
        has_generated_tests: false,
        has_property_tests: false,
        has_integration_scenario: false,
    };
    let rungs = select_rungs(complexity, &caps, 0);

    // 2. Build and run the pipeline
    let mut pipeline = GatePipeline::new("wasm-tool-validation");
    for rung in &rungs {
        match rung {
            Rung::Compile => pipeline.push(Box::new(
                CompileGate::cargo().with_timeout_ms(300_000)
            )),
            Rung::Lint => pipeline.push(Box::new(
                ClippyGate::cargo().with_timeout_ms(120_000)
            )),
            Rung::Test => pipeline.push(Box::new(
                TestGate::cargo().with_timeout_ms(600_000)
            )),
            _ => {} // Other rungs not applicable to WASM tool builds
        }
    }

    // 3. Create a synthetic engram from the working directory
    let engram = Engram::from_text(working_dir.to_string_lossy());
    let ctx = Context::now();
    let verdict = pipeline.verify(&engram, &ctx).await;

    // 4. Update adaptive thresholds
    for rung in &rungs {
        thresholds.observe(rung.as_index(), verdict.passed);
    }

    // 5. Generate filtered feedback
    let feedback = feedback_for_agent(
        verdict.detail.as_deref().unwrap_or(""),
        rungs.last().map_or(0, |r| r.as_index() as u8),
    );

    Ok(ValidationResult { verdict, feedback })
}
```

### Phase 3: Code Generation Quality Gate (3-5 days)

**Goal**: Gate-check agent-generated code before presenting to the user.

**File to modify**: `src/agent/` (within the agent loop, after code generation)

```rust
use ironclaw_gate::{
    AdaptiveThresholds, ProcessRewardModel, TurnSnapshot,
    PlanComplexity, RungCaps, select_rungs, feedback_for_agent,
};

/// Gate-check agent-generated code before presenting to user.
pub async fn gate_check_generated_code(
    working_dir: &Path,
    change_files: &[PathBuf],
    thresholds: &mut AdaptiveThresholds,
    prm: &mut ProcessRewardModel,
) -> GateCheckResult {
    let complexity = ironclaw_gate::complexity::assess_complexity(
        &change_files.iter().map(|p| p.as_path()).collect::<Vec<_>>(),
        count_total_lines(change_files),
    );

    let caps = RungCaps {
        has_lint_tool: true,
        has_symbol_manifest: false,
        has_generated_tests: false,
        has_property_tests: false,
        has_integration_scenario: false,
    };
    let rungs = select_rungs(complexity, &caps, 0);

    // Build and run pipeline (same pattern as Phase 2)
    let pipeline = build_gate_pipeline(&rungs, working_dir);
    let verdict = pipeline.verify(&engram, &ctx).await;

    // Record in PRM for trajectory tracking
    let snapshot = TurnSnapshot {
        rung: rungs.last().map_or(0, |r| r.as_index()),
        verdicts: vec![verdict.clone()],
        error_count: count_errors(&verdict),
        diff_lines: count_diff_lines(change_files),
    };
    prm.record_turn(snapshot);

    // Check for early termination
    if prm.should_terminate(0.3) {
        return GateCheckResult::EarlyTermination {
            promise: prm.promise(),
            progress: prm.progress(),
        };
    }

    let feedback = feedback_for_agent(
        verdict.detail.as_deref().unwrap_or(""),
        rungs.last().map_or(0, |r| r.as_index() as u8),
    );

    GateCheckResult::Complete { verdict, feedback }
}
```

### Phase 4: Sandbox Pre-Validation (1 day)

**Goal**: Run compile + lint before spending Docker container resources.

**File to modify**: `src/sandbox/manager.rs`

```rust
use ironclaw_gate::{CompileGate, ClippyGate, GatePipeline, feedback_for_agent};

/// Pre-validate code before launching a sandbox container.
pub async fn pre_sandbox_gate(working_dir: &Path) -> Result<(), SandboxError> {
    let pipeline = GatePipeline::new("pre-sandbox")
        .with_gate(Box::new(CompileGate::cargo()))
        .with_gate(Box::new(ClippyGate::cargo()));

    let engram = Engram::from_text(working_dir.to_string_lossy());
    let ctx = Context::now();
    let verdict = pipeline.verify(&engram, &ctx).await;

    if !verdict.passed {
        let feedback = feedback_for_agent(
            verdict.detail.as_deref().unwrap_or(""), 0
        );
        return Err(SandboxError::PreValidationFailed {
            reason: verdict.reason,
            feedback,
        });
    }
    Ok(())
}
```

### Phase 5: Acceptance-Contract-Driven Job Completion (2-3 days)

**Goal**: Define acceptance contracts per job type, validate evidence before marking jobs complete.

**File to modify**: `src/evaluation/` (new evaluator)

```rust
use ironclaw_gate::{
    AcceptanceContract, AcceptanceEvidence, AcceptanceOutcome,
    GateRequirement, GateRequirementKind, NoStubRequirement,
};

/// Create a standard acceptance contract for code generation jobs.
pub fn code_generation_contract(project_src_path: &str) -> AcceptanceContract {
    AcceptanceContract {
        version: 1,
        gates: vec![
            GateRequirement {
                id: "compile".to_string(),
                kind: GateRequirementKind::Compile,
                command: Some("cargo check".to_string()),
                required: true,
            },
            GateRequirement {
                id: "test".to_string(),
                kind: GateRequirementKind::Test,
                command: Some("cargo test".to_string()),
                required: true,
            },
        ],
        no_stub: Some(NoStubRequirement {
            required: true,
            production_paths: vec![project_src_path.to_string()],
        }),
        review_verdict: None,
        agent_output: None,
        recovery: None,
        parity_ledger: None,
    }
}

/// Evaluate job completion against its acceptance contract.
pub fn evaluate_completion(
    contract: &AcceptanceContract,
    evidence: &AcceptanceEvidence,
) -> JobDecision {
    let decision = contract.validate_evidence(evidence);
    match decision.outcome {
        AcceptanceOutcome::Passed => JobDecision::Complete,
        AcceptanceOutcome::NeedsRetry => JobDecision::Retry,
        AcceptanceOutcome::NeedsReplan => JobDecision::Replan,
        AcceptanceOutcome::NeedsHuman => JobDecision::Escalate,
        AcceptanceOutcome::NeedsWork => JobDecision::Continue,
        _ => JobDecision::Failed(decision),
    }
}
```

### Phase 6: Threshold Persistence (1 day)

**Goal**: Persist adaptive thresholds and ratchet state across sessions.

**File to modify**: `crates/ironclaw_gate/src/persistence.rs`

```rust
use ironclaw_gate::{AdaptiveThresholds, GateRatchet};
use std::path::PathBuf;

/// Resolve the path for gate state persistence.
pub fn gate_state_dir(base_dir: &Path) -> PathBuf {
    base_dir.join("gate")
}

pub fn threshold_path(base_dir: &Path) -> PathBuf {
    gate_state_dir(base_dir).join("thresholds.json")
}

pub fn ratchet_path(base_dir: &Path) -> PathBuf {
    gate_state_dir(base_dir).join("ratchet.json")
}

/// Load or create adaptive thresholds from the IronClaw base directory.
pub fn load_thresholds(base_dir: &Path) -> AdaptiveThresholds {
    AdaptiveThresholds::load_or_new(&threshold_path(base_dir))
}

/// Save adaptive thresholds to the IronClaw base directory.
pub fn save_thresholds(
    base_dir: &Path,
    thresholds: &AdaptiveThresholds,
) -> Result<(), std::io::Error> {
    thresholds.save(&threshold_path(base_dir))
}

/// Load or create gate ratchet from the IronClaw base directory.
pub fn load_ratchet(base_dir: &Path) -> GateRatchet {
    GateRatchet::load_or_new(&ratchet_path(base_dir))
}
```

---

## Implementation Architecture

### Full File Listing

```
crates/roko-gate/src/
  lib.rs                    # Re-exports, module doc, two-tier architecture summary
  gate_pipeline.rs          # GatePipeline + ComposedGatePipeline orchestrator
  rung_selector.rs          # PlanComplexity, Rung, RungCaps, select_rungs()
  rung_dispatch.rs          # Runtime mapping: Rung enum -> concrete gate(s)
  composition.rs            # ParallelGate, VotingGate, FallbackGate
  adaptive_threshold.rs     # AdaptiveThresholds, RungStats, ThresholdProfile
  spc.rs                    # CusumDetector, EwmaControlChart, BocpdDetector, SpcDetector
  hotelling.rs              # HotellingDetector, joint anomaly via T-squared
  pelt.rs                   # PeltDetector, offline change point detection
  process_reward.rs         # ProcessRewardModel (promise + progress)
  ratchet.rs                # GateRatchet (anti-regression)
  forensic.rs               # ForensicReplayBuilder, CausalChain, ArtifactMetadata
  artifact_store.rs         # BLAKE3 content-addressed artifact storage
  acceptance_contract.rs    # AcceptanceContract, AcceptanceEvidence, AcceptanceDecision
  feedback.rs               # GateFeedback, feedback_for_agent()
  error.rs                  # GateError types
  payload.rs                # GatePayload, BuildSystem, TestSelector
  registry.rs               # GateKind, GateRegistry, GateSpec, GateStatus
  review_verdict.rs         # Structured review verdict parsing
  verdict_publisher.rs      # VerdictPublisher for broadcasting outcomes
  env_builder.rs            # GateEnvBuilder for constructing gate environments
  gate_service.rs           # GateService (higher-level service layer)
  error_patterns.rs         # Failure pattern recording and classification
  compile_errors.rs         # Structured compile error classification
  # --- Concrete gates ---
  compile.rs                # CompileGate (cargo/npm/go/python/forge/make)
  clippy_gate.rs            # ClippyGate (Rust linting)
  test_gate.rs              # TestGate (cargo/npm/go/pytest/forge test)
  shell.rs                  # ShellGate (arbitrary command)
  diff_gate.rs              # DiffGate (git diff analysis)
  code_exec.rs              # CodeExecutionGate (sandboxed execution)
  symbol_gate.rs            # SymbolGate (symbol manifest verification)
  generated_test_gate.rs    # GeneratedTestGate (LLM-generated tests)
  verify_chain_gate.rs      # VerifyChainGate (evidence chain verification)
  property_test_gate.rs     # PropertyTestGate (property-based/fuzz)
  fact_check.rs             # FactCheckGate (claim verification via search oracle)
  llm_judge_gate.rs         # LlmJudgeGate (LLM quality assessment)
  integration_gate.rs       # IntegrationGate (full integration tests)
  benchmark_gate.rs         # BenchmarkRegressionGate (perf regression)
  format_check_gate.rs      # FormatCheckGate (code formatting)
  security_scan_gate.rs     # SecurityScanGate (vulnerability scanning)
  generated.rs              # GateGenerator, GeneratedCheck (dynamic)
  eval_generator.rs         # EvalGenerator, EvalTemplate (evaluation synthesis)
```

### Build Systems Supported

```rust
// Source: crates/roko-gate/src/payload.rs
pub enum BuildSystem {
    Cargo,   // Rust
    Npm,     // JavaScript/TypeScript
    Go,      // Go
    Python,  // Python
    Forge,   // Solidity (Foundry)
    Make,    // Make
}
```

Each of `CompileGate`, `ClippyGate`, and `TestGate` takes a `BuildSystem` and picks the appropriate command.

---

## Complexity Assessment

### Core Pipeline Effort

| Component | Estimated Lines | Notes |
|-----------|----------------|-------|
| Gate pipeline + composition | ~600 | Already in roko-gate |
| Rung selector + dispatch | ~500 | Already in roko-gate |
| Adaptive thresholds + SPC | ~1,000 | Already in roko-gate |
| Process reward model | ~450 | Already in roko-gate |
| Gate ratchet | ~300 | Already in roko-gate |
| Forensic replay | ~400 | Already in roko-gate |
| Acceptance contracts | ~950 | Already in roko-gate |
| Agent feedback | ~400 | Already in roko-gate |
| Hotelling T-squared | ~440 | Already in roko-gate |
| PELT offline detection | ~350 | Already in roko-gate |
| Individual concrete gates | ~200-400 each | ~15 gates shipped |

### IronClaw Integration Effort

| Phase | Integration Point | Estimated Lines | Risk | Duration |
|-------|-------------------|----------------|------|----------|
| 1 | `ironclaw_gate` wrapper crate | ~200 | Low | 1-2 days |
| 2 | Tool builder validation | ~300 | Low | 2-3 days |
| 3 | Code generation quality gate | ~500 | Medium | 3-5 days |
| 4 | Sandbox pre-validation | ~100 | Low | 1 day |
| 5 | Acceptance-contract-driven completion | ~400 | Medium | 2-3 days |
| 6 | Threshold persistence | ~100 | Low | 1 day |

### Risk Assessment

- **Low-medium overall**: The entire roko-gate crate is a library with no I/O dependencies beyond subprocess execution. Integration is progressive -- start with rungs 0-2 (compile, lint, test) for immediate value, then add adaptive thresholds and higher rungs over time.
- **Dependencies**: Language-specific tooling (cargo, npm, etc.) for concrete gates; LLM provider for generated tests and LLM judge gates.
- **Performance**: Compile/lint/test gates shell out to real tools, so gate duration is dominated by those tools. The SPC detectors, PRM, and threshold calculations are pure math on small data structures -- effectively zero overhead.

---

## References

[1] Page, E.S. (1954). "Continuous Inspection Schemes." *Biometrika*, 41(1/2), 100-115. The original CUSUM (Cumulative Sum) control chart paper. Introduced sequential accumulation of deviations from a target to detect sustained shifts in a process mean.

[2] Roberts, S.W. (1959). "Control Chart Tests Based on Geometric Moving Averages." *Technometrics*, 1(3), 239-250. Introduced the EWMA (Exponentially Weighted Moving Average) control chart, showing improved detection of small shifts in the process mean compared to Shewhart charts.

[3] Adams, R.P. and MacKay, D.J.C. (2007). "Bayesian Online Changepoint Detection." arXiv:0710.3742. Introduced BOCPD, a computationally efficient exact Bayesian method for online detection of change points by maintaining a run-length distribution.

[4] Lightman, H., Kosaraju, V., Burda, Y., Edwards, H., Baker, B., Lee, T., Leike, J., Schulman, J., Sutskever, I., and Cobbe, K. (2023). "Let's Verify Step by Step." arXiv:2305.20050. Demonstrated that process supervision (providing feedback at each reasoning step) significantly outperforms outcome supervision for training reward models, achieving 78% accuracy on MATH benchmarks. Released the PRM800K dataset of 800,000 step-level human feedback labels.

[5] Setlur, A., Nagpal, C., Fisch, A., Geng, X., Eisenstein, J., Agarwal, R., Aghajanyan, A., Zaheer, M., and Bansal, S. (2025). "Process Reward Models for LLM Agents: Practical Framework and Directions." arXiv:2502.10325. Proposed AgentPRM, a lightweight actor-critic framework for training LLM agents using Monte Carlo rollouts to compute reward targets, achieving results that outperform GPT-4o baselines.

[6] Hotelling, H. (1947). "Multivariate Quality Control -- Illustrated by the Air Testing of Sample Bombsights." In Eisenhart, C., Hastay, M.W., and Wallis, W.A. (Eds.), *Techniques of Statistical Analysis*, McGraw-Hill, New York, 111-184. Introduced the T-squared statistic for multivariate quality control, enabling simultaneous monitoring of multiple process variables.

[7] Welford, B.P. (1962). "Note on a Method for Calculating Corrected Sums of Squares and Products." *Technometrics*, 4(3), 419-420. Introduced a numerically stable single-pass algorithm for computing variance and covariance, avoiding the catastrophic cancellation inherent in naive two-pass formulas. Presented in Knuth's *The Art of Computer Programming*, Vol. 2.

[8] Wilson, E.B. and Hilferty, M.M. (1931). "The Distribution of Chi-Square." *Proceedings of the National Academy of Sciences*, 17(12), 684-688. Showed that the cube root of a chi-squared variable divided by its degrees of freedom is approximately normally distributed, providing a simple and accurate approximation for chi-squared critical values.

[9] Killick, R., Fearnhead, P., and Eckley, I.A. (2012). "Optimal Detection of Changepoints With a Linear Computational Cost." *Journal of the American Statistical Association*, 107(500), 1590-1598. Introduced the PELT algorithm, which achieves exact optimal segmentation of time series data with an average-case linear computational cost through a pruning step that eliminates suboptimal candidate change points.
