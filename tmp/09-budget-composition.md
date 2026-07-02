# Budget-Constrained Prompt Composition

**Source crate**: `roko-compose` (`crates/roko-compose/src/`)
**Priority**: MEDIUM -- enhances system prompt building and context management
**Key files**: `prompt.rs`, `auction.rs`, `scorer.rs`, `budget.rs`, `system_prompt_builder.rs`, `attention.rs`, `foraging.rs`, `strategy.rs`, `context_provider.rs`, `budget_predictor.rs`, `cost_attribution.rs`

---

## 1. Introduction: What Is Prompt Composition?

Prompt composition is the process of assembling the text that gets sent to a large language model (LLM) before it generates a response. For a simple chatbot, this might be a system message plus user input. For an AI agent that writes code, manages projects, and coordinates with other agents, the prompt can contain dozens of distinct components:

- **Identity and role instructions** (who the agent is, safety rules, behavioral constraints)
- **Tool definitions** (schemas, usage instructions, rate limits)
- **Project conventions** (coding standards, naming patterns, architecture rules)
- **Task context** (what to do, acceptance criteria, verification commands)
- **Memory** (relevant past experiences, knowledge entries, heuristics)
- **Conversation history** (prior turns, user instructions, error feedback)
- **Skills and playbooks** (domain-specific techniques, learned strategies)
- **Coordination signals** (peer agent pheromones, dependency outputs)

When the total token cost of all these components exceeds the model's context window, the system must make triage decisions. Naive approaches -- fixed priority ordering, round-robin, or truncating at the end -- waste tokens on low-value content while starving high-value content of space. Worse, research shows that simply filling the context window degrades output quality because LLMs attend unevenly to content at different positions.

**Budget-constrained prompt composition** treats prompt assembly as a formal resource-allocation problem. Rather than ad hoc concatenation, it applies:

- **Mechanism design from economics** (VCG auctions) to allocate token budget across competing content sources
- **Online learning from statistics** (Thompson Sampling) to learn which content contributes to task success
- **Ecological foraging theory from biology** (Marginal Value Theorem) to decide when to stop retrieving context from each source
- **Active inference from computational neuroscience** (Expected Free Energy) to balance goal-directed inclusion with uncertainty-reducing exploration

Roko's `roko-compose` crate implements this full pipeline. This document explains the theory, walks through the implementation, and maps it to IronClaw's existing architecture.

---

## 2. The U-Shaped Attention Curve: "Lost in the Middle"

### 2.1 The Research Finding

Large language models attend unevenly to content at different positions in the prompt. Liu et al. (2024) demonstrate a U-shaped attention curve in their paper "Lost in the Middle: How Language Models Use Long Contexts": models attend most strongly to content at the **beginning** (primacy effect) and **end** (recency effect) of the context window, with significantly degraded attention to content in the **middle**.

This is not a minor effect. The degradation can be severe enough that a model shown 10 relevant documents performs worse at retrieval when the answer is in document 5 than when shown only a single relevant document. The middle of the context window is an attention dead zone.

> **Citation**: Liu, N. F., Lin, K., Hewitt, J., Paranjape, A., Bevilacqua, M., Petroni, F., & Liang, P. (2024). Lost in the Middle: How Language Models Use Long Contexts. *Transactions of the Association for Computational Linguistics*, 12, 157-173. [ACL Anthology](https://aclanthology.org/2024.tacl-1.9/)

### 2.2 Roko's Position Attention Model

Roko models this U-shaped curve explicitly with the `PositionAttentionModel` struct in `crates/roko-compose/src/attention.rs`:

```rust
/// Attention multiplier based on position within a context window.
///
/// This is the scaffold-level model described in the composition docs for
/// approximating the U-shaped "lost in the middle" curve.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionAttentionModel {
    /// Primacy contribution at the beginning of the prompt.
    pub primacy_weight: f64,      // default: 0.35
    /// Decay applied to the primacy contribution as position increases.
    pub primacy_decay: f64,       // default: 3.0
    /// Recency contribution near the end of the prompt.
    pub recency_weight: f64,      // default: 0.30
    /// Decay applied to the recency contribution as position approaches zero.
    pub recency_decay: f64,       // default: 3.0
    /// Baseline attention that remains across the full prompt.
    pub baseline: f64,            // default: 0.35
}
```

**Source**: `crates/roko-compose/src/attention.rs`, lines 14-26.

The attention at normalized position `p` (0.0 = start, 1.0 = end) is:

```
attention(p) = primacy_weight * exp(-primacy_decay * p)
             + recency_weight * exp(-recency_decay * (1 - p))
             + baseline
```

**Source**: `attention_at()` method, `crates/roko-compose/src/attention.rs`, lines 43-48.

**Worked example** with default parameters:

| Position | Primacy term | Recency term | Baseline | Total | Interpretation |
|----------|-------------|-------------|----------|-------|----------------|
| p = 0.0 (start) | 0.35 * exp(0) = 0.35 | 0.30 * exp(-3) = 0.015 | 0.35 | 0.715 | High attention (primacy) |
| p = 0.25 | 0.35 * exp(-0.75) = 0.165 | 0.30 * exp(-2.25) = 0.032 | 0.35 | 0.547 | Declining |
| p = 0.5 (middle) | 0.35 * exp(-1.5) = 0.078 | 0.30 * exp(-1.5) = 0.067 | 0.35 | 0.495 | Attention trough |
| p = 0.75 | 0.35 * exp(-2.25) = 0.037 | 0.30 * exp(-0.75) = 0.142 | 0.35 | 0.529 | Rising |
| p = 1.0 (end) | 0.35 * exp(-3) = 0.017 | 0.30 * exp(0) = 0.30 | 0.35 | 0.667 | High attention (recency) |

The model is clamped to [0.0, 1.0]. The test directly asserts the U-shape:

```rust
#[test]
fn position_attention_model_has_u_shape() {
    let model = PositionAttentionModel::default();
    let start = model.attention_at(0.0);   // ~0.715
    let mid = model.attention_at(0.5);     // ~0.495
    let end = model.attention_at(1.0);     // ~0.667

    assert!(start > mid);
    assert!(end > mid);
}
```

**Source**: `crates/roko-compose/src/attention.rs`, lines 151-159.

### 2.3 Placement Zones

Roko uses three placement zones to exploit the U-shaped curve (`crates/roko-compose/src/prompt.rs`, lines 63-78):

```rust
/// Where in the final prompt the section should be placed.
///
/// U-shaped attention (Start/End) defeats "Lost in the Middle" effects for
/// large context windows. Use `Start` for role/instructions and `End` for
/// the current task.
pub enum Placement {
    /// Place near the top (role prompt, critical instructions).
    Start,
    /// Middle -- most vulnerable to attention loss.
    Middle,
    /// Place near the bottom (current task, recent errors).
    End,
}
```

Placement affects effective scores with constant multipliers (`crates/roko-compose/src/attention.rs`, lines 84-92):

```rust
pub const fn placement_adjusted_score(base_score: f64, placement: Placement) -> f64 {
    match placement {
        Placement::Start => base_score,         // 1.00x -- full attention zone
        Placement::End => base_score * 0.95,    // 0.95x -- strong attention zone
        Placement::Middle => base_score * 0.70, // 0.70x -- attention dead zone
    }
}
```

**Design rationale**: Start gets 1.0x because the primacy effect is the strongest. End gets 0.95x (nearly as good) because the recency effect is strong. Middle gets 0.70x -- a 30% penalty reflecting the measured attention degradation from Liu et al.

### 2.4 Dynamic Placement

The `dynamic_placement()` function in `crates/roko-compose/src/attention.rs` (lines 98-124) automatically reassigns non-critical sections to higher-attention positions based on their relevance to the current query. Sections are ranked by an information-density proxy (term overlap with query, content uniqueness, and compactness). The top third goes to Start, the bottom third to End, and the remaining third stays in Middle. Critical sections always keep their original placement.

### 2.5 Per-Model Fitted Curves

Different models have different attention profiles. The `ModelAttentionCurves` struct (`crates/roko-compose/src/attention.rs`, lines 58-64) stores per-model fitted parameters:

```rust
pub struct ModelAttentionCurves {
    /// Model id to fitted curve mapping.
    pub curves: HashMap<String, PositionAttentionModel>,
    /// Fallback curve used when a model-specific fit is unavailable.
    pub default_curve: PositionAttentionModel,
}
```

This allows the system to use Claude-specific attention parameters when calling Claude, GPT-specific parameters when calling GPT, and so on.

---

## 3. The 9-Layer System Prompt Builder

### 3.1 Architecture

The `SystemPromptBuilder` in `crates/roko-compose/src/system_prompt_builder.rs` assembles system prompts from 9 distinct layers, each targeting a different stability tier for LLM prefix-cache optimization.

The builder uses a fluent API pattern:

```rust
let prompt = SystemPromptBuilder::new("You are an implementer...")
    .with_conventions("Use snake_case, thiserror for errors")
    .with_domain("DeFi protocol context: ...")
    .with_task("Implement the rate limiter in crates/golem-core")
    .with_tools("MCP tools available: Read, Write, Bash")
    .with_anti_patterns(vec!["Never call unwrap in library crates"])
    .build();
```

### 3.2 The 9 Layers and Their Cache Behavior

From the module doc comment in `system_prompt_builder.rs` (lines 8-21):

| Layer | Content | Cache Tier | Stability |
|-------|---------|------------|-----------|
| **1. Role identity** | Who am I, what's my job | System (stable) | Changes only when agent role changes. Cached across all tasks for a given role. |
| **2. Conventions** | Project coding standards | System (semi-stable) | Changes when project conventions are updated. Cached across tasks within one project. |
| **3. Domain context** | Project-specific knowledge | Session (semi-stable) | Broader project knowledge injected once per session. Cached across tasks within one plan. |
| **3c. Active signals** | Pheromone / stigmergic guidance | Session (semi-stable) | Coordination signals from peer agents. May change between ticks but stable within a session. |
| **4. Task context** | Current task details | Task (volatile) | The actual task brief, acceptance criteria, verification commands. Changes every task. |
| **4b. Gate feedback** | Prior verification failure digest | Dynamic | Retry-specific guidance from previous gate failures. Only present on retries. |
| **5. Tool instructions** | Available tools and usage | System (stable) | Tool definitions rarely change mid-session. Cached alongside role identity. |
| **6. Relevant techniques** | Learned playbooks and skills | Task (volatile) | Selected per-task from the skill/playbook library. Changes every task. |
| **7. Anti-patterns** | What NOT to do | Task (volatile) | Warnings and negative examples specific to the current task. |
| **8. Affect guidance** | Emotional tone and focus | Dynamic | PAD (Pleasure-Arousal-Dominance) derived tone guidance. Changes every tick based on the Daimon's behavioral state. |

### 3.3 Cache Layer Tiers

Each layer maps to one of four cache tiers (`crates/roko-compose/src/prompt.rs`, lines 45-61):

```rust
pub enum CacheLayer {
    /// System prompt, role instructions, tool definitions.
    Role = 0,       // Most stable -- cached longest
    /// Workspace map, cross-plan context, durable project context.
    Workspace = 1,  // Semi-stable -- cached across tasks in a plan
    /// Plan/task brief content that is stable within a plan.
    Plan = 2,       // Per-plan -- cached within one task sequence
    /// Turn-local content such as review feedback or error output.
    Volatile = 3,   // Per-turn -- never cached
}
```

The key design insight from the system_prompt_builder module doc:

> "System prompts matter enormously (3-4x quality gap per the `--bare` experiment), AND they should be task-specific, not one-size-fits-all."

Sections are emitted in cache-layer order (Role first, Volatile last), with cache alignment markers (`<!-- cache:TIER -->`) placed between stability tiers. This allows downstream API callers to set `cache_control` breakpoints so the LLM provider reuses the longest possible KV-cache prefix across related turns. Layers 1 + 2 + 5 form the prefix-cacheable "system" tier; layers 3 and 3c form the "session" tier; layers 4 + 6 + 7 are per-task; layers 4b + 8 are dynamic retry/tone guidance.

### 3.4 Cache Alignment Markers

The `budget.rs` module (lines 128-133) generates cache break hints after stable layers:

```rust
// Cache break hints: insert breaks after stable layers so the LLM
// prefix cache can reuse the system/session prefix across turns.
//
// Layer boundaries (from CacheLayer enum):
//   System  -> role identity, agents.md, conventions
//   Session -> plan, workspace_map, brief
//   Task    -> tasks, file_context, enhancements
//   Dynamic -> reviews, error digest
let cache_breaks = vec![
    "conventions",   // end of System layer
    "workspace_map", // end of Session layer
    "file_context",  // end of Task layer
];
```

A convenience function formats these as HTML comments for downstream renderers:

```rust
pub fn cache_marker(layer_name: &str) -> String {
    format!("<!-- cache:{layer_name} -->")
}
```

### 3.5 Learned Section Effectiveness

The builder supports learned section-effectiveness adjustments via the `with_section_effectiveness()` method. This uses a `SectionEffectivenessRegistry` from `roko-learn` that tracks per-role, per-section statistics on whether including a section correlates with gate success. When the registry has sufficient evidence, sections with positive lift get priority promotions and sections with negative lift get demoted.

---

## 4. The PromptSection Type

Every piece of content that could appear in a prompt is represented as a `PromptSection` (`crates/roko-compose/src/prompt.rs`, lines 104-119):

```rust
pub struct PromptSection {
    /// Stable section identifier. Defaults to `prompt:<normalized name>`.
    pub section_id: String,
    /// Human-readable label (e.g. "role", "task", "workspace_map").
    pub name: String,
    /// The section's text content.
    pub content: String,
    /// Priority for budget-pressure dropping.
    pub priority: SectionPriority,
    /// Cache layer for LLM prefix-cache optimization.
    pub cache_layer: CacheLayer,
    /// Where in the final prompt to place this section.
    pub placement: Placement,
    /// Optional per-section token ceiling.
    pub hard_cap: Option<usize>,
    /// Which subsystem is bidding for this section's inclusion.
    pub bidder: AttentionBidder,
    /// Source metadata for attribution and learning.
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub provenance: Option<String>,
    pub experiment_id: Option<String>,
}
```

### 4.1 Priority Levels

Four priority levels determine survival under budget pressure (`prompt.rs`, lines 31-43):

```rust
pub enum SectionPriority {
    /// Drop first under pressure (fluff, historical context).
    Low = 0,
    /// Keep if possible (conventions, hints).
    Normal = 1,
    /// Essential to the task (the actual task, acceptance criteria).
    High = 2,
    /// Never drop (role instructions, safety hooks).
    Critical = 3,
}
```

**Critical sections are contractually guaranteed inclusion.** The composer will return an error rather than silently drop a Critical section. This ensures safety rules and role identity are always present.

### 4.2 Hard Caps and Truncation

Each section can have a `hard_cap` (maximum tokens). The `enforce_hard_cap()` method truncates content to fit, appending a `...[truncated N tokens]` marker. Truncation preserves the head of the content, which matters because the U-shaped attention curve means the beginning of each section receives the most attention.

### 4.3 Token Estimation

Token counts use a fast heuristic (`prompt.rs`, lines 23-26):

```rust
pub const fn estimate_tokens(text: &str) -> usize {
    text.len().div_ceil(4)
}
```

This approximation (4 bytes per token) is adequate for budget accounting. For precise counting, the `TokenCounter` in `crates/roko-compose/src/token_counter.rs` supports tiktoken (for OpenAI/Claude models), HuggingFace tokenizers, and a heuristic fallback. Claude averages ~3.5 chars/token on code-heavy prompts.

---

## 5. The 8 Attention Bidders

Each prompt section belongs to a cognitive subsystem that "bids" for its inclusion. The `AttentionBidder` enum (`crates/roko-compose/src/prompt.rs`, lines 80-101) defines 8 subsystems:

```rust
pub enum AttentionBidder {
    /// Durable knowledge retrieved from Neuro.
    Neuro,
    /// Affect or somatic guidance from Daimon.
    Daimon,
    /// Recent turns, retries, and prior task outputs.
    IterationMemory,
    /// Symbols, files, and structural workspace context.
    CodeIntelligence,
    /// Skills, playbooks, and distilled reusable rules.
    PlaybookRules,
    /// Research memos and external domain context.
    Research,
    /// Task brief, plan brief, verification, PRD slices, and related directives.
    TaskContext,
    /// Predictions, warnings, or forecast-like oracle outputs.
    Oracles,
}
```

| Bidder | What It Provides | Typical Bid Range | Bids High When |
|--------|-----------------|-------------------|----------------|
| **Neuro** | Knowledge store entries (insights, heuristics, warnings) | 0.3-0.8 | High similarity to current task |
| **Daimon** | Affect-tagged memories (prospect markers, PAD annotations) | 0.2-0.6 | High arousal or strong prior association |
| **IterationMemory** | Recent turn history within current task | 0.4-0.7 | Multi-turn task with cumulative context |
| **CodeIntelligence** | Symbols, files, structural workspace context | 0.3-0.7 | Task requires specific file/symbol knowledge |
| **PlaybookRules** | Machine-evolved rules from dream consolidation | 0.5-0.9 | Rule condition matches current situation |
| **Research** | Research memos and external domain context | 0.3-0.7 | Task is in an unfamiliar domain |
| **TaskContext** | Task brief, acceptance criteria, verification | 0.6-0.9 | Task is well-defined with clear verification |
| **Oracles** | Predictions, warnings, forecast-like outputs | 0.1-0.4 | Oracle has high confidence on relevant prediction |

---

## 6. The VCG Auction Mechanism

### 6.1 Background: Why an Auction?

When total content exceeds the token budget, the system must decide which sections to include. This is a classic resource-allocation problem with competing demands. Roko uses a Vickrey-Clarke-Groves (VCG) auction because it provides a mathematically proven property: **truthful bidding is the dominant strategy**.

In a VCG auction, each bidder's payment equals the **externality** they impose on other bidders -- the total value others lost because this bidder was included. This means:

- No bidder can increase its allocation by inflating its bid (inflating forces higher payments without additional allocation).
- Each bidder's optimal strategy is to report its true value.
- The mechanism maximizes total social welfare (the sum of all included sections' values).

> **Citations**:
> - Vickrey, W. (1961). Counterspeculation, Auctions, and Competitive Sealed Tenders. *The Journal of Finance*, 16(1), 8-37. [Wiley](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1540-6261.1961.tb02789.x)
> - Clarke, E. H. (1971). Multipart Pricing of Public Goods. *Public Choice*, 11, 17-33.
> - Groves, T. (1973). Incentives in Teams. *Econometrica*, 41(4), 617-631.

### 6.2 The VCG Allocation Algorithm

The implementation lives in `crates/roko-compose/src/auction.rs`, function `vcg_allocate()` (lines 380-501):

```rust
/// Allocate context window tokens using a greedy VCG-style mechanism.
///
/// Bids are sorted by `adjusted_bid / tokens` (value density). Sections
/// are included greedily until the budget is exhausted. VCG payments are
/// computed as the externality each winner imposes on others.
pub fn vcg_allocate(
    bids: Vec<VcgBid>,
    total_budget: usize,
    modulation: &AffectModulation,
) -> VcgAllocation {
```

**Step 1: Sort by value density** (lines 407-422).

Each bid has an `adjusted_bid` (value after affect modulation) and a `tokens` count. The algorithm sorts by value density = `adjusted_bid / tokens`, descending. This ensures the most value-per-token sections are considered first.

```rust
sorted.sort_by(|a, b| {
    let density_a = if a.tokens > 0 {
        a.adjusted_bid / a.tokens as f64
    } else {
        f64::INFINITY
    };
    let density_b = if b.tokens > 0 {
        b.adjusted_bid / b.tokens as f64
    } else {
        f64::INFINITY
    };
    density_b.partial_cmp(&density_a).unwrap_or(std::cmp::Ordering::Equal)
});
```

**Step 2: Greedy allocation** (lines 424-435).

Include sections in value-density order as long as they fit within the remaining budget:

```rust
let mut remaining = total_budget;
let mut winners = Vec::new();
let mut excluded = Vec::new();
for bid in sorted {
    if bid.tokens <= remaining {
        remaining -= bid.tokens;
        winners.push(bid);
    } else {
        excluded.push(bid);
    }
}
```

**Step 3: Compute VCG payments** (lines 440-449).

For each winner, the payment is the highest excluded bid that would have fit in its token slot -- the externality this winner imposed on the next-best alternative:

```rust
for winner in &winners {
    let payment = excluded
        .iter()
        .filter(|e| e.tokens <= winner.tokens)
        .map(|e| e.adjusted_bid)
        .fold(0.0_f64, f64::max);
    payments.push((winner.section_name.clone(), payment));
}
```

### 6.3 Full Mathematical Formulation

Given:
- A set of bids B = {b_1, b_2, ..., b_n} where each b_i = (v_i, t_i) with value v_i and token cost t_i
- A total budget T

**Value density** for bid i:

```
d_i = v_i / t_i
```

**Greedy allocation**: Sort bids by d_i descending. Include bid i if:

```
sum(t_j for all previously included j) + t_i <= T
```

Let W = set of winners, E = set of excluded bids.

**VCG payment** for winner i:

```
p_i = max{v_j : j in E, t_j <= t_i}
```

This is a simplified VCG payment that captures the core incentive-compatibility property: the payment equals the "opportunity cost" that winner i imposes by occupying its token slot.

**Total welfare** = sum of all winners' adjusted bids.

**Pareto optimality** check (`auction.rs`, lines 226-244): An allocation is Pareto-optimal if no swap of an included section for an excluded section can improve total welfare. The `is_pareto_optimal()` function checks this:

```rust
pub fn is_pareto_optimal(
    included: &[SectionAllocation],
    excluded: &[SectionAllocation],
    budget_remaining: usize,
) -> bool {
    for excluded_section in excluded {
        if excluded_section.tokens <= budget_remaining {
            return false; // could add more without removing anything
        }
        for included_section in included {
            if included_section.value < excluded_section.value
                && included_section.tokens >= excluded_section.tokens
            {
                return false; // swap would improve welfare
            }
        }
    }
    true
}
```

### 6.4 Worked Example: VCG Auction in Action

Suppose the token budget is 800 tokens and three subsystems submit bids:

| Section | Bidder | Tokens | Adjusted Bid | Value Density |
|---------|--------|--------|-------------|---------------|
| **knowledge** | Neuro | 500 | 0.8 | 0.0016 |
| **task** | TaskContext | 300 | 0.6 | 0.0020 |
| **research** | Research | 400 | 0.4 | 0.0010 |

**Step 1 -- Sort by value density**: task (0.0020) > knowledge (0.0016) > research (0.0010)

**Step 2 -- Greedy allocation**:
1. Include **task** (300 tokens). Remaining: 800 - 300 = 500 tokens.
2. Include **knowledge** (500 tokens). Remaining: 500 - 500 = 0 tokens.
3. Exclude **research** (400 tokens > 0 remaining).

**Winners**: {task, knowledge} using 800/800 tokens (100% utilization).
**Excluded**: {research}.

**Step 3 -- VCG payments**:
- Payment for **task** (300 tokens): max excluded bid where tokens <= 300. Research has 400 tokens > 300, so no eligible excluded bid. Payment = 0.
- Payment for **knowledge** (500 tokens): max excluded bid where tokens <= 500. Research has 400 <= 500, bid = 0.4. Payment = 0.4.

**Interpretation**: Knowledge's VCG payment of 0.4 means it displaced research (which had value 0.4). If knowledge's true value were below 0.4, it would not be worth including -- it would "pay more than it's worth." This incentivizes truthful value reporting.

This matches the test in `auction.rs` (lines 608-645):

```rust
#[test]
fn vcg_allocate_basic() {
    let bids = vec![
        VcgBid { bidder: SubsystemId::Neuro, section_name: "knowledge".into(),
                 tokens: 500, raw_bid: 0.8, adjusted_bid: 0.8, valence: 0.0 },
        VcgBid { bidder: SubsystemId::TaskContext, section_name: "task".into(),
                 tokens: 300, raw_bid: 0.6, adjusted_bid: 0.6, valence: 0.0 },
        VcgBid { bidder: SubsystemId::Research, section_name: "research".into(),
                 tokens: 400, raw_bid: 0.4, adjusted_bid: 0.4, valence: 0.0 },
    ];
    let allocation = vcg_allocate(bids, 800, &AffectModulation::default());
    assert_eq!(allocation.winners.len(), 2, "should fit 2 of 3 bids in 800 tokens");
    assert_eq!(allocation.excluded.len(), 1);
    assert!(allocation.total_tokens_used <= 800);
}
```

### 6.5 Why Truthful Bidding Matters

In the context of prompt composition, the "bidders" are not humans -- they are subsystems (knowledge retrieval, task context, code intelligence, etc.). But truthful bidding still matters because it means the system converges on the *actual* relative value of each content source rather than being distorted by hardcoded priority weights. The VCG mechanism ensures that the most genuinely valuable content wins budget space.

> **Note on the approximation**: Roko's implementation is a *greedy VCG-style* mechanism, not a full combinatorial VCG auction. The full VCG mechanism would solve an NP-hard knapsack optimization to find the welfare-maximizing allocation. The greedy approximation (sort by value density, include greedily) is computationally efficient and provides a 1/2-approximation to optimal welfare for the 0-1 knapsack problem. The payment rule is also simplified to "highest displaced bid" rather than the full externality computation. This is a practical trade-off: the incentive-compatibility guarantee is approximate rather than exact, but the mechanism remains strategy-proof in practice because subsystem bidders are cooperative, not adversarial.

### 6.6 Affect Modulation

The VCG auction supports emotional-state modulation via `AffectModulation` (`auction.rs`, lines 292-336). The agent's PAD (Pleasure-Arousal-Dominance) state modifies bids:

```rust
pub struct AffectModulation {
    /// Arousal-derived urgency multiplier (default 1.0, range [0.5, 2.0]).
    pub urgency_multiplier: f64,
    /// Pleasure-derived valence bias (range [-1.0, 1.0]).
    pub affect_weight: f64,
}

impl AffectModulation {
    pub fn from_pad(pleasure: f64, arousal: f64) -> Self {
        Self {
            urgency_multiplier: (1.0 + arousal * 0.5).clamp(0.5, 2.0),
            affect_weight: pleasure.clamp(-1.0, 1.0),
        }
    }

    pub fn adjust_bid(&self, base_bid: f64, entry_valence: f64) -> f64 {
        let valence = entry_valence.clamp(-1.0, 1.0);
        base_bid * self.urgency_multiplier * (1.0 + self.affect_weight * valence)
    }
}
```

This means:
- **High arousal** (urgency) increases all bids, giving more context to high-stakes situations
- **Positive pleasure** biases toward positive-valence content (proven strategies, success patterns)
- **Negative pleasure** biases toward negative-valence content (warnings, failure patterns, risk assessments)

**Worked example**: Agent is struggling (pleasure = -0.4, arousal = 0.8):
- `urgency_multiplier` = 1.0 + 0.8 * 0.5 = 1.4 (everything gets 40% more budget)
- `affect_weight` = -0.4
- A warning section (valence = -0.7): adjusted_bid = base * 1.4 * (1 + (-0.4) * (-0.7)) = base * 1.4 * 1.28 = base * 1.792 (79% boost)
- A success pattern (valence = 0.8): adjusted_bid = base * 1.4 * (1 + (-0.4) * 0.8) = base * 1.4 * 0.68 = base * 0.952 (5% penalty)

When struggling, the system naturally up-weights warnings and down-weights "feel-good" content.

### 6.7 Auction Diagnostics

Every auction produces a diagnostic record (`AuctionDiagnostics`, `auction.rs` lines 173-189):

```rust
pub struct AuctionDiagnostics {
    pub total_welfare: f64,
    pub total_payments: f64,
    pub welfare_loss: f64,
    pub pareto_optimal: bool,
    pub highest_payment_sections: Vec<(String, f64)>,
    pub displaced_sections: Vec<(String, f64)>,
    pub budget_utilization: f64,
}
```

### 6.8 Bid Correlation Detection

The `detect_bid_correlation()` function (`auction.rs`, lines 211-222) identifies structurally coupled bidder pairs using Pearson correlation. If two bidders always bid high or low together (correlation > threshold), this signals structural coupling that could be exploited -- or that the bidders should be merged.

---

## 7. Thompson Sampling Learning Bidders

### 7.1 The Learning Problem

Static bidding is suboptimal because the value of a section depends on the task. A section on "database optimization" is high-value for a SQL task but low-value for a CSS task. The system needs to *learn* which sections contribute to task success and adjust bids accordingly.

> **Citation**: Thompson, W. R. (1933). On the Likelihood that One Unknown Probability Exceeds Another in View of the Evidence of Two Samples. *Biometrika*, 25(3-4), 285-294.
>
> For the modern theoretical analysis:
> Agrawal, S. & Goyal, N. (2012). Analysis of Thompson Sampling for the Multi-armed Bandit Problem. *Proceedings of the 25th Annual Conference on Learning Theory (COLT)*. [PMLR](http://proceedings.mlr.press/v23/agrawal12/agrawal12.pdf)

### 7.2 The LearningBidder

The `LearningBidder` in `crates/roko-compose/src/auction.rs` (lines 31-170) maintains a Beta-distribution posterior for each section it has observed:

```rust
pub struct LearningBidder {
    /// Subsystem this bidder represents.
    pub subsystem_id: SubsystemId,
    /// Beta-posterior parameters keyed by section name.
    pub section_betas: HashMap<String, (f64, f64)>,  // (alpha, beta)
    /// Cost-effectiveness observations keyed by section name.
    pub section_costs: HashMap<String, SectionCostStats>,
    /// Prior value used before any observations are recorded.
    pub prior_bid: f64,
}
```

### 7.3 Beta-Distribution Posterior

Each section has parameters `(alpha, beta)` initialized to `(1.0, 1.0)` (the Bayes-Laplace uniform prior). After each task:

- If the section was included and the downstream gate **passed**: `alpha += 1.0`
- If the section was included and the downstream gate **failed**: `beta += 1.0`

This is standard Bayesian updating for a Beta-Bernoulli conjugate model. The Beta distribution is the conjugate prior for the Bernoulli likelihood, meaning the posterior remains a Beta distribution after each observation. The posterior mean `alpha / (alpha + beta)` estimates the probability that including this section leads to task success.

**Why Beta-Bernoulli?** The gate outcome is binary (pass/fail), making Bernoulli the natural likelihood. The Beta conjugate prior gives a closed-form posterior update -- no MCMC sampling or variational inference needed. The `(1, 1)` prior is uninformative (uniform on [0, 1]), expressing maximal uncertainty before any observations.

### 7.4 Thompson-Style Sampling

The `bid()` method (`auction.rs`, lines 74-82) uses a deterministic Thompson-style approximation rather than stochastic sampling (to avoid non-determinism in tests):

```rust
pub fn bid(&self, section_name: &str, relevance: f64) -> f64 {
    let (alpha, beta) = self
        .section_betas
        .get(section_name)
        .copied()
        .unwrap_or((1.0, 1.0));
    let sampled_track_record = thompson_like_sample(section_name, alpha, beta);
    sampled_track_record * relevance.max(0.0) * self.prior_bid.max(0.0)
}
```

The `thompson_like_sample` function (`auction.rs`, lines 246-253):

```rust
fn thompson_like_sample(section_name: &str, alpha: f64, beta: f64) -> f64 {
    let total = (alpha + beta).max(f64::EPSILON);
    let mean = alpha / total;
    let variance = (alpha * beta) / (total.powi(2) * (total + 1.0)).max(f64::EPSILON);
    let spread = variance.sqrt();
    let centered_unit = hash_to_unit(section_name) * 2.0 - 1.0;
    (mean + centered_unit * spread).clamp(0.0, 1.0)
}
```

### 7.5 Mathematical Formulation of Thompson-Like Sampling

For a Beta(alpha, beta) distribution, the exact moments are:

```
Posterior mean:     mu      = alpha / (alpha + beta)
Posterior variance: sigma^2 = (alpha * beta) / ((alpha + beta)^2 * (alpha + beta + 1))
```

The deterministic "sample" is:

```
sample = mu + hash_offset * sqrt(sigma^2)
```

where `hash_offset` is a deterministic pseudo-random value in [-1, 1] derived from hashing the section name. This provides stable exploration: the same section always gets the same offset, but different sections explore different parts of the posterior.

**Worked example**: After 8 gate passes and 2 gate failures for a section:

```
alpha = 1 + 8 = 9,  beta = 1 + 2 = 3
mu = 9/12 = 0.75
sigma^2 = (9 * 3) / (144 * 13) = 27/1872 = 0.0144
sigma = 0.120

If hash_offset = 0.3 (from hashing section name):
sample = 0.75 + 0.3 * 0.120 = 0.786
```

Compare with a section that has only 1 pass and 1 failure:

```
alpha = 2, beta = 2
mu = 0.5
sigma^2 = (4) / (16 * 5) = 0.05
sigma = 0.224

sample = 0.5 + 0.3 * 0.224 = 0.567
```

The well-observed successful section (0.786) bids much higher than the uncertain section (0.567). Over time, as more observations accumulate, the variance shrinks, and the sample converges to the mean -- the system "locks in" on its learned estimates.

### 7.6 Cost-Aware Bidding

The `bid_with_cost()` method multiplies the base bid by a cost-effectiveness factor (`auction.rs`, lines 155-169):

```rust
fn cost_effectiveness_factor(&self, section_name: &str) -> f64 {
    let Some(stats) = self.section_costs.get(section_name) else {
        return 1.0;  // no data yet, neutral factor
    };
    if stats.observation_count < 3 || stats.total_tokens == 0 {
        return 1.0;  // not enough data
    }

    let pass_rate = stats.passes as f64 / stats.observation_count as f64;
    let cost_per_1k_tokens = stats.total_cost_usd.max(0.0) / stats.total_tokens as f64 * 1000.0;
    let cost_efficiency = 1.0 / (1.0 + cost_per_1k_tokens);
    let quality = (0.7 * pass_rate + 0.3 * cost_efficiency).clamp(0.0, 1.0);

    (0.5 + quality * 1.5).clamp(0.5, 2.0)
}
```

This means sections that are cheap AND effective get up to a 2x bid multiplier, while sections that are expensive AND ineffective get a 0.5x penalty.

### 7.7 The Complete Feedback Loop

```
Bidders submit bids
    |
    v
Auction allocates tokens (Compose Cell)
    |
    v
Context assembled -> LLM inference -> Action taken
    |
    v
Gate pipeline runs (Verify Cell)
    |
    v
Verdict (pass/fail) + which sections were included
    |
    v
LearningBidder.update(section, included, passed)
    |
    v (loop)
Next tick: section effects modulate bid values
    |
    v
Bidders submit adjusted bids
```

---

## 8. Composition Strategy: Auto-Selection

### 8.1 Three Strategies

The `CompositionStrategy` enum (`crates/roko-compose/src/strategy.rs`, lines 13-26) defines how tokens are allocated:

```rust
pub enum CompositionStrategy {
    /// Select Vcg once learned bidder observations are warm; otherwise use
    /// the deterministic density-greedy path.
    Auto,        // default
    /// Deterministic greedy allocation by score density.
    DensityGreedy,
    /// Backward-compatible alias for density-greedy allocation.
    WeightedSum,
    /// VCG-style allocation with payments and displacement diagnostics.
    Vcg,
}
```

### 8.2 Auto Strategy Resolution

The `Auto` strategy uses the minimum observation count across all active bidders to decide when to switch from simple greedy to full VCG:

```rust
pub const DEFAULT_VCG_WARMUP_OBSERVATIONS: u32 = 10;

pub fn auto_select(
    bidder_observations: &HashMap<AttentionBidder, u32>,
    warmup_observations: u32,
) -> Self {
    let min_obs = bidder_observations.values().copied().min().unwrap_or(0);
    if min_obs >= warmup_observations {
        Self::Vcg
    } else {
        Self::DensityGreedy
    }
}
```

**Source**: `crates/roko-compose/src/strategy.rs`, lines 10, 48-58.

**Rationale**: VCG payments and affect modulation are only meaningful when the learning bidders have enough history to produce informed bids. During cold-start (first 10 observations per bidder), the deterministic density-greedy path is more stable. Once all bidders have warmed up, VCG provides better allocation through its truthful-bidding guarantees.

---

## 9. Context Tier Routing

### 9.1 Three Tiers

The `ContextTier` enum in `crates/roko-compose/src/context_provider.rs` (lines 38-75) defines three tiers based on task complexity and model capability:

| Tier | Token Budget | Model Targets | What Gets Included |
|------|-------------|---------------|-------------------|
| **Surgical** | ~4,000 tokens | Haiku, Ollama, Gemma -- mechanical tasks | Inline files, symbol signatures, anti-patterns, verification only. No enrichment artifacts, no plan context. |
| **Focused** | ~12,000 tokens | Sonnet -- focused/integrative tasks | Surgical + task-scoped brief, dependency graph excerpt, prior task outputs. |
| **Full** | ~24,000 tokens | Opus -- architectural tasks | Focused + plan-level brief, cross-plan context, research memo, invariants/rubric. |

```rust
pub enum ContextTier {
    Surgical,
    Focused,
    Full,
}

impl ContextTier {
    pub fn from_task_and_model(task_tier: &str, model_slug: &str) -> Self {
        if is_local_model(model_slug) {
            return Self::Surgical;
        }
        match task_tier {
            "mechanical" => Self::Surgical,
            "architectural" => Self::Full,
            _ => Self::Focused,
        }
    }

    pub const fn default_token_budget(self) -> usize {
        match self {
            Self::Surgical => 4_000,
            Self::Focused => 12_000,
            Self::Full => 24_000,
        }
    }
}
```

**Key design decision**: Local models (Ollama, Gemma, Llama, etc.) **always** get Surgical tier regardless of task complexity, because they cannot reliably use tools and have smaller context windows. The `is_local_model()` function detects these by slug prefix.

### 9.2 Complexity-Adaptive Budget Scaling

The `budget.rs` module adds a complexity dimension on top of per-role budgets:

```rust
pub enum Complexity {
    /// Single-file, trivial change. Drop PRD, research, decomposition sections.
    Trivial,
    /// Standard multi-file task. Full budget at role defaults.
    Standard,
    /// Cross-crate or architectural work. Inflated budgets for context.
    Complex,
}
```

From `adjusted_budget_from_base()` (`budget.rs`, lines 84-142):

- **Trivial**: Zero out `prd2`, `context`, and `skills` sections (they add noise for simple tasks). Halve `workspace_map` and `brief`.
- **Standard**: Use the base budget as-is.
- **Complex**: Inflate `workspace_map` by 50%, `context` by 100%, `file_context` by 50%.

---

## 10. The PromptComposer: Full Assembly Pipeline

### 10.1 Overview

The `PromptComposer` struct (`crates/roko-compose/src/prompt.rs`) implements the `Compose` trait from `roko-core`. It takes a set of `Signal<PromptSection>` inputs, a `Budget`, a `Scorer`, and a `Context`, and produces a single `Signal<Prompt>` output.

### 10.2 The Composition Algorithm

1. **Decode** all input sections from signal bodies.
2. **Drop** any that do not decode (provenance-tainted or wrong kind).
3. **Partition**: Split into Critical (never drop) and Optional sections.
4. **Budget check**: If Critical sections alone exceed the budget, return an error.
5. **Score**: Compute bid density for each Optional section, incorporating learning bidders.
6. **Dedup** (COMP-04): If HDC dedup is enabled, remove near-duplicate candidates (cosine similarity > threshold).
7. **Forage** (COMP-03): If a `MultiPatchForager` is configured, apply the MVT stopping rule to limit candidates.
8. **Select strategy**: Resolve `Auto` to either `DensityGreedy` or `Vcg` based on bidder warmth.
9. **Allocate**: Run the selected strategy to pick winners within budget.
10. **Sort by placement**: Order kept sections by Placement (Start -> Middle -> End), ties broken by cache layer.
11. **Render**: Concatenate with optional section headers. Build output Signal with composition manifest metadata.

### 10.3 The Bid Pipeline

For each Optional section, the bid value is computed as:

```
bid_value = score * learned_multiplier

where:
  score = candidate_score(section, source_signal, scorer, ctx)
  learned_multiplier = learning_bidders[section.bidder].bid_with_cost(section.name, 1.0)

bid_density = bid_value / estimated_tokens
```

When learning bidders are not registered for a subsystem, the `learned_multiplier` defaults to 1.0, so scoring falls back to the pure scorer.

### 10.4 Diversity Boost and Diminishing Returns

The `effective_candidate_bid()` function applies two additional adjustments:

```rust
fn effective_candidate_bid(
    candidate: &AuctionCandidate<'_>,
    bidder_wins: &HashMap<AttentionBidder, usize>,
    affect: Option<&AuctionAffectState>,
) -> f32 {
    let bidder = candidate.section.bidder;
    let wins = bidder_wins.get(&bidder).copied().unwrap_or(0);
    let diversity_boost = if wins == 0 { 1.18 } else { 1.0 };
    let diminishing_returns = 0.82_f32.powi(wins as i32);
    let affect_multiplier = bidder_affect_multiplier(&candidate.section, affect);
    candidate.bid_density * diversity_boost * diminishing_returns * affect_multiplier
}
```

- **Diversity boost** (+18%): A subsystem's first section gets a bonus to ensure diverse representation.
- **Diminishing returns** (0.82^n): Each additional section from the same subsystem is worth 18% less than the previous one. This prevents any single subsystem from monopolizing the budget.

**Worked example**: Neuro subsystem submits 4 sections with base bid_density = 1.0:

| Section # | Diversity | Diminishing | Effective |
|-----------|-----------|-------------|-----------|
| 1st | 1.18 | 0.82^0 = 1.000 | 1.180 |
| 2nd | 1.00 | 0.82^1 = 0.820 | 0.820 |
| 3rd | 1.00 | 0.82^2 = 0.672 | 0.672 |
| 4th | 1.00 | 0.82^3 = 0.551 | 0.551 |

By the 4th section, Neuro's effective bid is less than half the 1st section's. This ensures other subsystems get budget space.

### 10.5 Composition Manifest

Every composition produces a `CompositionManifest` that records:

- Strategy requested vs. strategy actually used
- Included sections with section ID, action ID, bidder, estimated tokens, score, bid value, VCG payment
- Excluded sections with the same metadata
- VCG diagnostics (when VCG was selected)
- Total tokens and budget limit

This manifest enables downstream learning systems to correlate section inclusion with task outcomes.

---

## 11. Section Scoring: Goal-Directed Heuristic Scoring

### 11.1 The SectionScorer

The basic `SectionScorer` in `crates/roko-compose/src/scorer.rs` (lines 22-91) ranks sections by four dimensions:

| Dimension | Weight | How It's Computed |
|-----------|--------|-------------------|
| **Confidence** | Priority-mapped | Critical=1.0, High=0.8, Normal=0.4, Low=0.2 |
| **Novelty** | Time-decayed | 1.0 if < 1 hour old, linear decay to 0.0 over 24 hours |
| **Utility** | Size-inverse | `1000 / content_length`, capped at 10.0. Shorter = higher utility per token |
| **Reputation** | Trust-based | 1.0 for trusted provenance, 0.1 for tainted |

### 11.2 The GoalDirectedHeuristicScorer (ActiveInferenceScorer)

The `GoalDirectedHeuristicScorer` (`scorer.rs`, lines 127-258) adds goal-directed scoring that approximates Expected Free Energy (EFE). It computes two values for each section:

**Pragmatic value** (how aligned with the current goal):

```rust
fn pragmatic_value(&self, signal: &Signal, section: Option<&PromptSection>) -> f32 {
    let embedding_similarity = cosine_similarity(&self.goal_embeddings, &section_embedding);
    let lexical_similarity = token_overlap(&section.content, &self.goal_text);
    let goal_similarity = (0.65 * embedding_similarity + 0.35 * lexical_similarity)
        .clamp(0.0, 1.0);
    let priority_bonus = match section.priority {
        Critical => 0.18, High => 0.12, Normal => 0.06, Low => 0.02,
    };
    (goal_similarity + priority_bonus).clamp(0.0, 1.0)
}
```

**Epistemic value** (how much uncertainty it would reduce):

```rust
fn epistemic_value(&self, signal: &Signal, section: Option<&PromptSection>) -> f32 {
    let uncertainty = (1.0 - topic_belief).clamp(0.0, 1.0);
    let novelty_hint = signal.score.novelty.clamp(0.0, 1.0);
    let informational_leverage = (1.0 / content_length.sqrt()).clamp(0.0, 1.0);
    (0.65 * uncertainty + 0.2 * novelty_hint + 0.15 * informational_leverage)
        .clamp(0.0, 1.0)
}
```

### 11.3 HDC-Approximate EFE (Design Note COMP-05)

The scorer uses Hyperdimensional Computing (HDC) hash embeddings as a lightweight approximation of full Bayesian EFE. From the extensive design note in `scorer.rs` (lines 100-125):

> "The spec (doc 07) calls for full Expected Free Energy (EFE) scoring with proper Bayesian belief updates (KL divergence between posterior and prior). This implementation uses an HDC-inspired hash embedding + cosine similarity as an intentional approximation."

**Three-point justification:**

1. **Correlation**: HDC cosine similarity correlates with information gain for text sections. High-similarity sections are redundant (low epistemic value); low-similarity sections provide novel information (high epistemic value).

2. **Computational cost**: Proper Bayesian belief updates require maintaining a full posterior distribution and simulating updates for each candidate section during prompt assembly. This is prohibitively expensive for real-time composition where hundreds of candidates are scored per prompt build.

3. **Pragmatic adequacy**: The hash-embedding approach produces ranking decisions that are good enough for prompt assembly budgeting. The quality gap between this approximation and proper EFE is small relative to the benefit of having any scoring at all.

The HDC embedding is computed in `embed_text()` (`scorer.rs`, lines 297-310):

```rust
fn embed_text(text: &str, dimensions: usize) -> Vec<f32> {
    let mut vector = vec![0.0_f32; dimensions.max(1)];
    for (position, token) in tokenize(text).into_iter().enumerate() {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        token.hash(&mut hasher);
        position.hash(&mut hasher);
        let hash = hasher.finish();
        let index = (hash as usize) % vector.len();
        let sign = if hash & 1 == 0 { 1.0 } else { -1.0 };
        vector[index] += sign;
    }
    normalize_embedding(vector)
}
```

This creates a 32-dimensional hash-based embedding that captures token co-occurrence patterns without requiring a trained model.

---

## 12. Expected Free Energy: Mathematical Formulation

### 12.1 The EFE Equation

Active inference frames all agent behavior as minimizing Expected Free Energy (EFE) -- a quantity that combines goal-directed action (pragmatic value) with information-seeking behavior (epistemic value).

```
G(a) = -pragmatic(a) - epistemic(a) + cost(a)

pragmatic(a) = -D_KL(predicted_obs(a) || preferred_obs)
epistemic(a) = sum_s [ b(s) * H[P(o|s)] ]
cost(a) = dollar_cost(a) * cost_sensitivity
```

Where:
- **Pragmatic value**: KL divergence between predicted observations under action `a` and preferred observations. Measures how close the action gets to the goal.
- **Epistemic value**: Expected entropy of observations under the current belief state. High entropy = high information gain = high epistemic value.
- **Cost**: Direct resource cost of the action.

The system selects `argmin G(a)` -- the action with lowest free energy (most negative = best).

> **Citations**:
> - Friston, K. (2006). A Free Energy Principle for the Brain. *Journal of Physiology -- Paris*, 100(1-3), 70-87.
> - Friston, K. (2010). The Free-Energy Principle: A Unified Brain Theory? *Nature Reviews Neuroscience*, 11(2), 127-138.
> - Parr, T., Pezzulo, G., & Friston, K. J. (2022). *Active Inference: The Free Energy Principle in Mind, Brain, and Behavior*. MIT Press. [MIT Press](https://direct.mit.edu/books/oa-monograph/5299/Active-InferenceThe-Free-Energy-Principle-in-Mind)

### 12.2 The State Space

The state space is factorized into three dimensions:

```
State = (TaskPhase, ContextQuality, Uncertainty)

TaskPhase in {Understanding, Planning, GatheringContext, Implementing, Verifying, Complete}
ContextQuality in {None, Insufficient, Partial, Adequate, Comprehensive}
Uncertainty in {High, Medium, Low}

Total: 6 x 5 x 3 = 90 states
```

This factorized 90-state model makes active inference tractable. The key insight: "do not model the world -- model the agent's epistemic situation."

> **Citation**: Prakki, R. (2024). Active Inference for Self-Organizing Multi-LLM Systems: A Bayesian Thermodynamic Approach to Adaptation. [arXiv:2412.10425](https://arxiv.org/abs/2412.10425)
>
> For the pymdp active inference framework used as reference implementation:
> Heins, C., Millidge, B., Demekas, D., Klein, B., Friston, K., Couzin, I. D., & Tschantz, A. (2022). pymdp: A Python library for active inference in discrete state spaces. *Journal of Open Source Software*, 7(73), 4098. [JOSS](https://joss.theoj.org/papers/10.21105/joss.04098)

### 12.3 How EFE Relates to Prompt Composition

In the composition context, EFE is approximated by the `GoalDirectedHeuristicScorer`:

- **Pragmatic value** -> `pragmatic_value()`: cosine similarity between goal embedding and section embedding
- **Epistemic value** -> `epistemic_value()`: uncertainty (1 - belief), novelty, and informational leverage
- **Cost** -> token count (implicit in the budget constraint)

The scorer produces a combined score that naturally balances "include content that helps achieve the goal" (pragmatic) with "include content that reduces uncertainty" (epistemic), all under the token budget constraint (cost).

### 12.4 Summary of Research References

| Concept | Reference |
|---------|-----------|
| EFE routing | Friston (2006), "A free energy principle for the brain," *J. Physiology -- Paris* |
| Active inference framework | Friston (2010), "The free-energy principle: a unified brain theory?" *Nature Reviews Neuroscience*; Parr, Pezzulo & Friston (2022), *Active Inference*, MIT Press |
| Active inference for LLM systems | Prakki (2024), arXiv:2412.10425 |
| VCG auction | Vickrey (1961), *J. Finance*; Clarke (1971), *Public Choice*; Groves (1973), *Econometrica* |
| pymdp framework | Heins et al. (2022), *JOSS* |
| Thompson Sampling | Thompson (1933), *Biometrika*; Agrawal & Goyal (2012), *COLT* |
| Lost in the Middle | Liu et al. (2024), *TACL* |
| Marginal Value Theorem | Charnov (1976), *Theoretical Population Biology* |

---

## 13. Multi-Patch Foraging with Active Inference

### 13.1 The Foraging Problem

When assembling context, the agent draws from multiple sources (knowledge store, file system, conversation history, research memos). Each source has diminishing returns: the first retrieval from a source is highly valuable, but subsequent retrievals from the same source yield progressively less new information. The question is: when should the agent stop retrieving from one source and switch to another?

This is the classic **patch foraging problem** from behavioral ecology. The Marginal Value Theorem (MVT) provides the optimal solution.

> **Citation**: Charnov, E. L. (1976). Optimal Foraging, the Marginal Value Theorem. *Theoretical Population Biology*, 9(2), 129-136. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/004058097690040X)

### 13.2 The Marginal Value Theorem

The MVT states that an optimal forager should leave a patch (context source) when the **marginal gain rate** in the current patch drops to the **average gain rate** across all patches in the environment. In other words: stop exploiting the current source when you could do better by switching to a fresh one.

The `MultiPatchForager` in `crates/roko-compose/src/foraging.rs` (lines 24-112) implements MVT. Each context source is modeled as a "patch" with a gain curve:

```rust
pub struct SourceForagingProfile {
    /// Source this profile describes.
    pub source: ContextSource,
    /// Asymptotic relevance available from this source.
    pub g_max: f64,
    /// Saturation rate for the diminishing-returns curve.
    pub lambda: f64,
    /// Setup and switching cost for the source.
    pub travel_cost: f64,
}
```

The gain from `n` iterations in a source follows an exponential saturation curve:

```
gain(n) = g_max * (1 - exp(-lambda * n))
marginal_gain(n) = d/dn gain(n) = g_max * lambda * exp(-lambda * n)
```

**Worked example**: Knowledge store with g_max = 0.9, lambda = 0.25:

| Iteration | Cumulative Gain | Marginal Gain | Decision (if env_rate = 0.05) |
|-----------|----------------|---------------|-------------------------------|
| 1 | 0.9 * (1 - e^-0.25) = 0.198 | 0.9 * 0.25 * e^-0.25 = 0.175 | Continue (0.175 > 0.05) |
| 2 | 0.9 * (1 - e^-0.50) = 0.354 | 0.9 * 0.25 * e^-0.50 = 0.137 | Continue (0.137 > 0.05) |
| 3 | 0.9 * (1 - e^-0.75) = 0.475 | 0.9 * 0.25 * e^-0.75 = 0.106 | Continue (0.106 > 0.05) |
| 5 | 0.9 * (1 - e^-1.25) = 0.642 | 0.9 * 0.25 * e^-1.25 = 0.065 | Continue (0.065 > 0.05) |
| 7 | 0.9 * (1 - e^-1.75) = 0.744 | 0.9 * 0.25 * e^-1.75 = 0.039 | **Leave** (0.039 < 0.05) |

The forager should retrieve 6-7 entries from the knowledge store before switching to the next source.

### 13.3 Optimal Visitation Order

The `optimal_order()` method sorts sources by expected initial gain (`g_max * lambda`):

```rust
pub fn optimal_order(&self) -> Vec<ContextSource> {
    let mut profiles = self.source_profiles.clone();
    profiles.sort_by(|left, right| {
        self.expected_initial_gain(&right.source)
            .partial_cmp(&self.expected_initial_gain(&left.source))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    profiles.into_iter().map(|profile| profile.source).collect()
}
```

**Source**: `crates/roko-compose/src/foraging.rs`, lines 41-50.

### 13.4 Should-Visit Decision

The `should_visit()` method (lines 65-75) checks whether a source's initial gain justifies the travel cost:

```rust
pub fn should_visit(&self, source: &ContextSource) -> bool {
    let Some(profile) = self.profile_for(source) else {
        return false;
    };
    let base_threshold = self.environment_rate * profile.travel_cost.max(0.0);
    // Active inference: lower the visit threshold when bias > 0,
    // making exploration more likely under uncertainty.
    let bias = self.active_inference_bias.clamp(0.0, 1.0);
    let adjusted_threshold = base_threshold * (1.0 - bias * 0.5);
    self.expected_initial_gain(source) > adjusted_threshold
}
```

### 13.5 Active Inference Bias

The `active_inference_bias` parameter (range 0.0 to 1.0) controls exploration vs. exploitation:

- **bias = 0.0**: Pure MVT. Stay in the best patch until marginal gain drops below environment average.
- **bias = 1.0**: Maximum exploration. Visit threshold is halved, patch stay is shortened by 30%.

At high bias:
- `should_visit()` threshold is halved -> more patches are visited
- `optimal_iterations()` threshold is raised by 30% -> shorter stays per patch -> more patches explored

This implements a simple Expected Free Energy influence on foraging: high uncertainty biases toward exploring more sources rather than deeply exploiting a known good source.

### 13.6 Optimal Iterations Per Source

The `optimal_iterations()` method (lines 83-105) uses binary search to find the iteration count where marginal gain equals the adjusted threshold:

```rust
pub fn optimal_iterations(&self, source: &ContextSource) -> usize {
    let mut lo = 1usize;
    let mut hi = 20usize;
    while lo < hi {
        let mid = (lo + hi) / 2;
        let marginal = profile.g_max * profile.lambda * (-profile.lambda * mid as f64).exp();
        let threshold =
            (self.environment_rate + profile.travel_cost / mid as f64) * (1.0 + bias * 0.3);
        if marginal > threshold {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo.clamp(1, 10)
}
```

### 13.7 Context Sufficiency and Stopping

The `estimate_context_sufficiency()` function (`foraging.rs`, lines 166-193) estimates whether retrieved context covers the task's requirements by checking term overlap between task terms and retrieved chunk content.

The `should_stop_searching()` function combines MVT ratio and sufficiency:

```rust
pub fn should_stop_searching(mvt_ratio: f64, sufficiency: f64, threshold: f64) -> bool {
    mvt_ratio <= 1.0 || sufficiency >= threshold
}
```

Stop when either:
- The marginal value ratio drops below 1.0 (diminishing returns)
- Sufficiency exceeds the threshold (enough context has been gathered)

### 13.8 Social Foraging Boost

The `social_foraging_boost()` function (lines 132-162) applies a capped relevance boost to context entries that were successfully used by peer agents for similar tasks. This is analogous to ant pheromone trails: if another agent used a knowledge entry for an "integration" task and it led to gate success, that entry gets a boost when the current agent is working on an "integration" task.

The boost decays exponentially with a configurable half-life and is capped at +0.3 relevance to prevent runaway amplification.

### 13.9 Calibration-Aware Stopping

The `calibration_to_foraging_factor()` function (`foraging.rs`, lines 225-229) adjusts the MVT stopping threshold based on the agent's recent prediction accuracy:

```rust
pub fn calibration_to_foraging_factor(recent_accuracy: f64, confidence: f64) -> f64 {
    if confidence < 0.1 { return 1.0; }  // cold start
    (0.5 + recent_accuracy).clamp(0.5, 1.5)
}
```

- High accuracy (0.8) -> factor 1.3 -> stop sooner (trust your context selection)
- Low accuracy (0.2) -> factor 0.7 -> search longer (context selection needs improvement)

---

## 14. Budget Prediction and Section Influence

### 14.1 Budget Predictor

The `BudgetPredictor` in `crates/roko-compose/src/budget_predictor.rs` (lines 88-222) predicts optimal token budgets from historical task data. It uses per-feature-key EMA (exponential moving average) of actual token usage:

```rust
pub struct BudgetPredictor {
    observations: HashMap<String, BudgetObservation>,
    pub alpha: f64,               // EMA smoothing factor, default 0.3
    pub fallback_tokens: u64,     // fallback when no history, default 100_000
    pub failure_inflation: f64,   // inflation on failure, default 1.3
}
```

Features are keyed by `role:complexity:domain` (e.g., `Implementer:standard:code`). When a task fails, the budget for that feature key is inflated by 30%, encouraging larger budgets for historically difficult feature combinations.

The EMA update rule:

```
ema_tokens_new = alpha * actual_tokens + (1 - alpha) * ema_tokens_old
```

With `alpha = 0.3`, recent observations have roughly 3x the weight of older ones. The predicted budget includes a 20% safety margin: `predicted = ema_tokens * 1.2`.

### 14.2 Section Influence

The `SectionInfluence` tracker (`budget_predictor.rs`, lines 276-375) performs leave-one-out influence scoring. For each prompt section, it tracks:

- `successes_with`: Tasks that included this section and succeeded
- `failures_with`: Tasks that included this section and failed
- `successes_without`: Tasks that excluded this section and succeeded
- `failures_without`: Tasks that excluded this section and failed

**Lift** = `rate_with - rate_without` measures the section's causal effect on success. Positive lift (section helps) maps to weight > 1.0; negative lift (section hurts) maps to weight < 1.0. The weight formula maps lift [-1.0, 1.0] to the multiplier range [0.5, 1.5]:

```
weight = (1.0 + lift).clamp(0.5, 1.5)
```

This is a natural experiment approach: since context assembly naturally varies (some sections are included in some tasks and excluded from others due to relevance, budget constraints, or availability), the system observes the correlation between inclusion and success over many tasks.

---

## 15. Per-Section Cost Attribution

The `CostAttribution` system in `crates/roko-compose/src/cost_attribution.rs` (lines 9-108) attributes the actual dollar cost of an LLM call back to individual prompt sections:

```rust
pub struct CostAttribution {
    pub turn_id: String,
    pub total_input_tokens: u64,
    pub total_cost_usd: f64,
    pub sections: Vec<SectionCost>,
    pub strategy: CompositionStrategy,
    pub vcg_payments: Vec<(String, f64)>,
}
```

Each section's attributed cost is proportional to its estimated token fraction:

```
attributed_cost_i = total_cost * (estimated_tokens_i / total_estimated_tokens)
```

After the downstream gate runs, the result is stamped onto each section with `stamp_gate_result()`. This allows computing per-section cost-effectiveness:

```
effectiveness_i = (1 if gate_passed else 0) / attributed_cost_i
```

Sections with high cost and low effectiveness are candidates for exclusion or token-cap reduction in future compositions.

---

## 16. Conversation History Compaction

The `compaction.rs` module (`crates/roko-compose/src/compaction.rs`) handles conversation-history compaction under budget pressure. When the conversation history grows too large, older messages are compressed into a summary while:

- **Anchor roles** (e.g., `system`) are preserved verbatim
- **Tool-result errors** are preserved verbatim (they contain diagnostic information)
- **Gate results and tool outcomes** are carried forward as structured JSON

The compaction is iterative: previously compacted summaries can be compacted again without losing their structured metadata.

---

## 17. End-to-End Worked Example

To make the full pipeline concrete, here is a complete example of how the composition system processes a task.

### 17.1 Scenario

An Implementer agent receives a task: "Add rate limiting to the HTTP webhook handler." The system has a 12,000-token budget (Focused tier, Sonnet model). Eight subsystems submit candidate sections.

### 17.2 Candidate Sections

| Section | Bidder | Tokens | Priority | Base Score | Relevance | Cost Factor |
|---------|--------|--------|----------|-----------|-----------|-------------|
| Role identity | TaskContext | 200 | Critical | -- | -- | -- |
| Safety rules | TaskContext | 150 | Critical | -- | -- | -- |
| Task brief: "Add rate limiting..." | TaskContext | 400 | High | 0.85 | 1.0 | 1.0 |
| Rate limiter knowledge entry | Neuro | 600 | Normal | 0.70 | 0.9 | 1.2 |
| HTTP handler file context | CodeIntelligence | 800 | Normal | 0.60 | 0.8 | 1.0 |
| Database optimization heuristic | Neuro | 500 | Normal | 0.50 | 0.2 | 0.8 |
| Prior task output: "Fixed webhook auth" | IterationMemory | 350 | Normal | 0.45 | 0.6 | 1.0 |
| Playbook: "Rust error handling" | PlaybookRules | 300 | Normal | 0.55 | 0.7 | 1.1 |
| Research: "DeFi protocol analysis" | Research | 700 | Low | 0.30 | 0.1 | 0.9 |
| Oracle: "Build time prediction" | Oracles | 250 | Low | 0.20 | 0.15 | 1.0 |

### 17.3 Step-by-Step Processing

**Phase 1: Partition**
- Critical sections (must include): Role identity (200) + Safety rules (150) = 350 tokens
- Remaining budget for Optional sections: 12,000 - 350 = 11,650 tokens

**Phase 2: Compute bids**

Using `bid = thompson_sample(alpha, beta) * relevance * prior_bid * cost_factor`:

| Section | Thompson Sample | * Relevance | * Cost Factor | Final Bid | Density (bid/tokens) |
|---------|----------------|------------|---------------|-----------|---------------------|
| Task brief | 0.78 | * 1.0 | * 1.0 | 0.780 | 0.00195 |
| Rate limiter knowledge | 0.72 | * 0.9 | * 1.2 | 0.778 | 0.00130 |
| HTTP handler context | 0.65 | * 0.8 | * 1.0 | 0.520 | 0.00065 |
| Playbook: error handling | 0.58 | * 0.7 | * 1.1 | 0.447 | 0.00149 |
| Prior task output | 0.50 | * 0.6 | * 1.0 | 0.300 | 0.00086 |
| DB optimization heuristic | 0.55 | * 0.2 | * 0.8 | 0.088 | 0.00018 |
| DeFi research | 0.35 | * 0.1 | * 0.9 | 0.032 | 0.00005 |
| Build time oracle | 0.25 | * 0.15 | * 1.0 | 0.038 | 0.00015 |

**Phase 3: Apply diversity boost and diminishing returns**

Neuro has two sections. The first gets the 1.18x diversity boost. The second gets 0.82x diminishing returns:
- Rate limiter knowledge (1st Neuro): 0.00130 * 1.18 = 0.00153
- DB optimization (2nd Neuro): 0.00018 * 0.82 = 0.00015

**Phase 4: VCG allocation** (sorted by effective density)

| Rank | Section | Tokens | Cumulative | Fits? |
|------|---------|--------|-----------|-------|
| 1 | Task brief | 400 | 400 | Yes |
| 2 | Rate limiter knowledge | 600 | 1,000 | Yes |
| 3 | Playbook: error handling | 300 | 1,300 | Yes |
| 4 | Prior task output | 350 | 1,650 | Yes |
| 5 | HTTP handler context | 800 | 2,450 | Yes |
| 6 | Build time oracle | 250 | 2,700 | Yes |
| 7 | DB optimization | 500 | 3,200 | Yes |
| 8 | DeFi research | 700 | 3,900 | Yes |

All fit! Total: 3,900 tokens used out of 11,650 available. If the budget were tighter (say 2,000 tokens), sections 6-8 would be excluded, and the VCG payments would reflect their displacement.

**Phase 5: Sort by placement**

| Position | Section | Placement | Cache Layer |
|----------|---------|-----------|-------------|
| 1 | Role identity | Start | Role |
| 2 | Safety rules | Start | Role |
| 3 | Playbook: error handling | Start | Workspace |
| 4 | Rate limiter knowledge | Middle | Plan |
| 5 | HTTP handler context | Middle | Plan |
| 6 | Prior task output | Middle | Volatile |
| 7 | DB optimization | Middle | Volatile |
| 8 | Task brief | End | Plan |
| 9 | Build time oracle | End | Volatile |
| 10 | DeFi research | End | Volatile |

Cache break markers are inserted after Safety rules (end of Role tier) and after Playbook (end of Workspace tier).

---

## 18. IronClaw Integration Plan

### A. Current IronClaw Prompt Architecture

IronClaw currently builds system prompts in `crates/ironclaw_engine/src/executor/prompt.rs`. The system:

1. Loads a CodeAct preamble from `crates/ironclaw_engine/prompts/codeact_preamble.md` via `include_str!`
2. Appends capability summaries (available tools, background status, activatable integrations)
3. Appends prior knowledge from completed threads
4. Appends active skills
5. Appends a CodeAct postamble with strategy instructions

The prompt is refreshed on each loop iteration via `refresh_system_prompt()` in `executor/loop_engine.rs`. Memory docs are retrieved and injected as context. The engine already has an `IRONCLAW_DISABLE_CODEACT` flag for structured-tool-only mode.

### B. Phase 1: Layered Prompt Builder (Low Risk)

**Map IronClaw's existing sections to the 9-layer model:**

| Roko Layer | IronClaw Equivalent | Source |
|------------|-------------------|--------|
| 1. Role identity | CodeAct preamble (`CODEACT_PREAMBLE`) + SOUL.md, AGENTS.md identity files | `executor/prompt.rs`, `workspace/` identity injection |
| 2. Conventions | Project CLAUDE.md content | Loaded from workspace |
| 3. Domain context | Prior knowledge from completed threads | `PRIOR_KNOWLEDGE_HEADING` section |
| 3c. Active signals | (Not yet implemented -- coordination signals) | Future: multi-agent coordination |
| 4. Task context | Current thread goal + message history | `executor/context.rs` |
| 4b. Gate feedback | (Not yet implemented -- retry guidance) | Future: gate pipeline feedback |
| 5. Tool instructions | Capability summaries + enabled tools | `CapabilitySummary` rendering |
| 6. Relevant techniques | Active skills | `ACTIVE_SKILLS_HEADING` section |
| 7. Anti-patterns | (Not yet implemented) | Future: learned warnings |
| 8. Affect guidance | (Not applicable -- no PAD system) | N/A |

**Implementation**:
- Create `crates/ironclaw_engine/src/executor/prompt_composer.rs`
- Define `IronClawPromptSection` that wraps each existing section with a `CacheLayer` and `Placement`
- Emit cache break markers between Role/Workspace/Plan tiers
- Existing `build_codeact_system_prompt_with_docs()` becomes the composition entry point

### C. Phase 2: Cache-Aware API Calls (Medium Impact)

**Where**: `crates/ironclaw_llm/` and `src/bridge/llm_adapter.rs`

Anthropic's API supports `cache_control` headers for prompt caching. IronClaw can use the cache alignment markers to set these:

```rust
// When building the Anthropic API call:
for (i, section) in composed_prompt.sections.iter().enumerate() {
    let is_cache_break = composed_prompt.cache_breakpoints.contains(&i);
    messages.push(ContentBlock {
        text: section.content.clone(),
        cache_control: if is_cache_break {
            Some(CacheControl::Ephemeral)
        } else {
            None
        },
    });
}
```

This directly reduces API costs by caching the stable prefix (identity + conventions + tools) across multiple turns.

### D. Phase 3: VCG Auction for Context Selection (Higher Complexity)

**Where**: `src/workspace/` memory retrieval and `crates/ironclaw_engine/src/memory/retrieval.rs`

When workspace memories are injected into the prompt, the VCG auction selects the most valuable subset within budget:

```rust
pub struct MemoryContextBidder {
    bidder: LearningBidder,
}

impl MemoryContextBidder {
    pub fn select_memories(
        &self,
        memories: &[MemoryEntry],
        task: &TaskContext,
        token_budget: usize,
    ) -> Vec<MemoryEntry> {
        let bids: Vec<VcgBid> = memories.iter().map(|memory| {
            let relevance = memory.similarity_to(task);
            let bid_value = self.bidder.bid_with_cost(&memory.id, relevance);
            VcgBid {
                bidder: AttentionBidder::Neuro,
                section_name: memory.id.clone(),
                tokens: estimate_tokens(&memory.content),
                raw_bid: bid_value,
                adjusted_bid: bid_value,
                valence: 0.0,
            }
        }).collect();

        let allocation = vcg_allocate(bids, token_budget, &AffectModulation::default());
        allocation.winners.iter()
            .filter_map(|w| memories.iter().find(|m| m.id == w.section_name))
            .cloned()
            .collect()
    }
}
```

### E. Phase 4: Progressive Tool Disclosure Enhancement

**Where**: Already partially implemented! (`feat(reborn): Context management -- progressive tool disclosure`)

The existing progressive tool disclosure feature determines which tools to include in the prompt. The VCG auction model can enhance this by having tools compete for prompt space based on:

- Historical use frequency for similar tasks (from `LearningBidder`)
- Relevance to the current task (from `GoalDirectedHeuristicScorer`)
- Token cost of the tool definition (from `estimate_tokens`)

### F. Phase 5: Multi-Patch Foraging for Context Assembly

**Where**: `src/workspace/` context retrieval, `crates/ironclaw_engine/src/memory/retrieval.rs`

When IronClaw's workspace needs to assemble context from multiple sources (memory search, file reading, conversation history), the `MultiPatchForager` can optimize retrieval:

```rust
let forager = MultiPatchForager {
    source_profiles: vec![
        SourceForagingProfile {
            source: ContextSource::KnowledgeEntry { .. },
            g_max: 0.9,   // memory search is high-value
            lambda: 0.25,  // but saturates slowly
            travel_cost: 0.1,
        },
        SourceForagingProfile {
            source: ContextSource::InlineFile { .. },
            g_max: 0.8,
            lambda: 0.5,   // file reads saturate faster
            travel_cost: 0.2,
        },
    ],
    environment_rate: 0.05,
    active_inference_bias: 0.3,  // moderate exploration
};

// Retrieve in optimal order, stopping when sufficient
for source in forager.optimal_order() {
    if !forager.should_visit(&source) { continue; }
    let iterations = forager.optimal_iterations(&source);
    for _ in 0..iterations {
        let chunk = retrieve_from(source);
        context.push(chunk);
    }
    let sufficiency = estimate_context_sufficiency(&context, &task);
    if should_stop_searching(mvt_ratio, sufficiency, 0.85) { break; }
}
```

### G. Persistence Requirements

Learned state needs to persist between sessions:

| Data | Path | Format |
|------|------|--------|
| Budget predictor EMA | `~/.ironclaw/learn/budget-predictor.json` | `BudgetPredictor` (serde) |
| Section influence | `~/.ironclaw/learn/section-influence.json` | `SectionInfluence` (serde) |
| Learning bidder posteriors | `~/.ironclaw/learn/bidder-posteriors.json` | Per-bidder `(alpha, beta)` maps |
| Model attention curves | `~/.ironclaw/learn/attention-curves.json` | `ModelAttentionCurves` (serde) |

All of these are small JSON files (< 100KB each) that serialize the accumulated learning state.

---

## 19. Complexity Assessment

| Component | Estimated Lines | Notes |
|-----------|----------------|-------|
| 9-Layer prompt builder | ~400-600 | Wrap existing prompt building with SystemPromptBuilder pattern |
| VCG auction integration | ~300-400 | Simpler than it sounds -- the algorithm is 120 lines, the rest is wiring |
| Thompson Sampling bidders | ~200-300 | LearningBidder is self-contained, needs persistence |
| Cache breakpoint optimization | ~200 | Cache markers + Anthropic API cache_control headers |
| Section scoring (HDC approx) | ~300 | GoalDirectedHeuristicScorer, no ML dependencies |
| Multi-patch foraging | ~150-200 | MultiPatchForager + sufficiency estimation |
| Budget prediction | ~200-300 | EMA-based, needs persistence in ~/.ironclaw/learn/ |
| Cost attribution | ~150 | Proportional attribution from composition manifest |
| Integration with existing engine | ~400-500 | Wiring into ironclaw_engine prompt assembly |
| **Total** | **~2,300-3,050** | |

**Risk**: Low -- can be introduced gradually, phase by phase. The VCG auction is only activated after warmup (10+ observations per bidder), so the system degrades gracefully to deterministic greedy allocation during cold-start.

**Dependencies**: Token counting (already available via `ironclaw_llm`), serde for persistence (already used throughout), no ML framework dependencies (HDC embeddings use std hash, Thompson sampling is closed-form).

---

## 20. Key Source File Reference

| File | What It Contains |
|------|-----------------|
| `crates/roko-compose/src/lib.rs` | Module declarations and public re-exports |
| `crates/roko-compose/src/prompt.rs` | `PromptSection`, `PromptComposer` (Compose impl), `CacheLayer`, `Placement`, `AttentionBidder`, `CompositionManifest` |
| `crates/roko-compose/src/auction.rs` | `LearningBidder`, `VcgBid`, `VcgAllocation`, `vcg_allocate()`, `AffectModulation`, `SectionCostStats`, `detect_bid_correlation()`, `is_pareto_optimal()` |
| `crates/roko-compose/src/scorer.rs` | `SectionScorer`, `GoalDirectedHeuristicScorer` (aka `ActiveInferenceScorer`), HDC embedding functions |
| `crates/roko-compose/src/budget.rs` | `AdjustedBudget`, `Complexity`, `adjusted_budget_for()`, cache break hints |
| `crates/roko-compose/src/system_prompt_builder.rs` | `SystemPromptBuilder` (9-layer builder with cache markers, budget profiles, section effectiveness) |
| `crates/roko-compose/src/attention.rs` | `PositionAttentionModel`, `ModelAttentionCurves`, `dynamic_placement()`, `placement_adjusted_score()` |
| `crates/roko-compose/src/foraging.rs` | `MultiPatchForager`, `SourceForagingProfile`, `social_foraging_boost()`, `estimate_context_sufficiency()`, `should_stop_searching()` |
| `crates/roko-compose/src/strategy.rs` | `CompositionStrategy`, `DEFAULT_VCG_WARMUP_OBSERVATIONS` |
| `crates/roko-compose/src/context_provider.rs` | `ContextTier`, `ContextSection`, `ContextBidder`, `LearningContextBidder`, `ContextInjectionBudget` |
| `crates/roko-compose/src/budget_predictor.rs` | `BudgetPredictor`, `SectionInfluence`, `TaskFeatures` |
| `crates/roko-compose/src/cost_attribution.rs` | `CostAttribution`, `SectionCost` |
| `crates/roko-compose/src/compaction.rs` | `compact_history()`, `CompactionPolicy`, `ChatMessage` |
| `crates/roko-compose/src/token_counter.rs` | `TokenCounter` (tiktoken, HuggingFace, heuristic) |

---

## 21. References

### Academic Papers

1. **Vickrey, W.** (1961). Counterspeculation, Auctions, and Competitive Sealed Tenders. *The Journal of Finance*, 16(1), 8-37.
2. **Clarke, E. H.** (1971). Multipart Pricing of Public Goods. *Public Choice*, 11, 17-33.
3. **Groves, T.** (1973). Incentives in Teams. *Econometrica*, 41(4), 617-631.
4. **Thompson, W. R.** (1933). On the Likelihood that One Unknown Probability Exceeds Another in View of the Evidence of Two Samples. *Biometrika*, 25(3-4), 285-294.
5. **Agrawal, S. & Goyal, N.** (2012). Analysis of Thompson Sampling for the Multi-armed Bandit Problem. *Proceedings of the 25th Annual Conference on Learning Theory (COLT)*.
6. **Liu, N. F., Lin, K., Hewitt, J., Paranjape, A., Bevilacqua, M., Petroni, F., & Liang, P.** (2024). Lost in the Middle: How Language Models Use Long Contexts. *Transactions of the Association for Computational Linguistics*, 12, 157-173.
7. **Charnov, E. L.** (1976). Optimal Foraging, the Marginal Value Theorem. *Theoretical Population Biology*, 9(2), 129-136.
8. **Friston, K.** (2006). A Free Energy Principle for the Brain. *Journal of Physiology -- Paris*, 100(1-3), 70-87.
9. **Friston, K.** (2010). The Free-Energy Principle: A Unified Brain Theory? *Nature Reviews Neuroscience*, 11(2), 127-138.
10. **Parr, T., Pezzulo, G., & Friston, K. J.** (2022). *Active Inference: The Free Energy Principle in Mind, Brain, and Behavior*. MIT Press.
11. **Heins, C., Millidge, B., Demekas, D., Klein, B., Friston, K., Couzin, I. D., & Tschantz, A.** (2022). pymdp: A Python library for active inference in discrete state spaces. *Journal of Open Source Software*, 7(73), 4098.
12. **Prakki, R.** (2024). Active Inference for Self-Organizing Multi-LLM Systems: A Bayesian Thermodynamic Approach to Adaptation. arXiv:2412.10425.

### Roko Design Documents

- `docs/v2-depth/07-agent-runtime/24-attention-auction-and-cortical-state.md` -- VCG auction specification, 8 bidder subsystems, CorticalState, section effects feedback loop
- `docs/v2-depth/07-agent-runtime/25-active-inference-state-space.md` -- EFE formulation, 90-state POMDP, A/B/C/D matrices
- `docs/v2-depth/07-agent-runtime/dual-process-and-efe-routing.md` -- T0/T1/T2 tier routing, EFE vs LinUCB comparison
- `docs/v2-depth/07-agent-runtime/cross-cut-functors.md` -- VCG arbitration between Memory, Daimon, and Dreams
- `docs/v2/04-EXECUTION.md` -- COMPOSE step in execution pipeline, VCG + section effects
