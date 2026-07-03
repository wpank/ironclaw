# Universal Engram: Rich Memory Primitives for Cognitive AI Systems

**Captured source set**: `roko-core` (`crates/roko-core/`)
**Priority**: HIGH -- richer memory entries with multi-axis scoring and decay
**Captured source identifiers**:
- `crates/roko-core/src/engram.rs` -- Engram struct, builder, HDC helpers, derive methods
- `crates/roko-core/src/score.rs` -- Score struct with 7 axes, `effective()` formula, arithmetic
- `crates/roko-core/src/decay.rs` -- Decay enum (None, HalfLife, Ttl, Ebbinghaus), `apply()`, `is_alive()`
- `crates/roko-core/src/demurrage.rs` -- Demurrage trait (balance, tick, replenish, is_depleted)
- `crates/roko-core/src/hash.rs` -- ContentHash (BLAKE3), hex serialization, `of()`, `short()`
- `crates/roko-core/src/provenance.rs` -- Provenance, Taint (9 variants), TaintInfo, coherence checks
- `crates/roko-core/src/traits.rs` -- Store, Score (trait), Verify, Route, Compose, React, Bus, ColdStore
- `crates/roko-core/src/kind.rs` -- Kind enum (30+ variants), Compound, `identity_key()`
- `crates/roko-core/src/body.rs` -- Body enum (Empty, Text, Json, Bytes), `canonical_bytes()`
- `crates/roko-core/src/attestation.rs` -- Ed25519 attestation, `sign()`, `verify()`, ChainAttestation
- `crates/roko-core/src/datum.rs` -- Datum polymorphism (Engram | Pulse)
- `crates/roko-core/src/signal.rs` -- Signal type alias (Engram -> Signal rename in progress)

Captured paths in this document identify source-corpus provenance only. They are not accessible checkout paths or dependencies to add to IronClaw.

---

## Table of Contents

1. [Introduction -- What Is an Engram?](#1-introduction----what-is-an-engram)
2. [Why a Universal Data Type for Knowledge](#2-why-a-universal-data-type-for-knowledge)
3. [The Engram Struct -- Complete Anatomy](#3-the-engram-struct----complete-anatomy)
4. [Content-Addressed Identity (BLAKE3)](#4-content-addressed-identity-blake3)
5. [7-Axis Scoring System](#5-7-axis-scoring-system)
6. [Four Decay Variants](#6-four-decay-variants)
7. [Demurrage -- Attention Economics](#7-demurrage----attention-economics)
8. [Kind -- Semantic Type System](#8-kind----semantic-type-system)
9. [Body -- Typed Payload](#9-body----typed-payload)
10. [Lineage DAG](#10-lineage-dag)
11. [Provenance and Taint Propagation](#11-provenance-and-taint-propagation)
12. [Cryptographic Attestation](#12-cryptographic-attestation)
13. [HDC Fingerprinting and Semantic Search](#13-hdc-fingerprinting-and-semantic-search)
14. [The Core Traits](#14-the-core-traits)
15. [The Engram/Pulse Duality](#15-the-engrampulse-duality)
16. [Engram Lifecycle -- A Worked Example](#16-engram-lifecycle----a-worked-example)
17. [Mermaid Diagrams](#17-mermaid-diagrams)
18. [Benchmarking](#18-benchmarking)
19. [Practical Examples](#19-practical-examples)
20. [IronClaw Integration Plan](#20-ironclaw-integration-plan)
21. [Complexity Assessment](#21-complexity-assessment)
22. [References](#22-references)

---

## 1. Introduction -- What Is an Engram?

The term "engram" originates from neuroscience. Richard Semon coined it in 1904 (Semon 1904) to describe the hypothetical physical trace that a memory leaves in the brain -- a biophysical change in neural tissue that encodes experience. For decades the engram was theoretical: Karl Lashley spent thirty years searching for it (Lashley 1950), ultimately concluding that memories were not localized to any single region. The concept was vindicated in 2015 when Susumu Tonegawa's group at MIT identified specific neurons ("engram cells") in the hippocampus and amygdala whose artificial reactivation could recall or suppress a fear memory (Tonegawa et al. 2015, *Science* 348(6238):1007-1013).

In the captured Roko design, an **Engram** is a content-addressed, scored, decaying unit of cognition. It acts as a common record shape for observations, decisions, outputs, and derived knowledge. Where biological engrams encode memories as patterns of synaptic weights, this software pattern encodes knowledge as structured data with explicit quality scores, decay functions, provenance records, and lineage graphs.

The key properties that make an Engram a rich memory primitive rather than a simple record:

- **Content-addressed identity**: An Engram is identified by a BLAKE3 hash of its content, not by an arbitrary ID. Identical canonical content produces identical identifiers, which supports deduplication, integrity checks, and linkability.
- **Multi-dimensional scoring**: Seven independent axes (confidence, novelty, utility, reputation, precision, salience, coherence) that collapse into a single effective score via a multiplicative formula. Zero confidence or zero reputation structurally produces zero score -- invalid or untrusted information cannot be prioritized.
- **Temporal decay**: Every Engram has a decay function -- none, exponential half-life, hard TTL cutoff, or Ebbinghaus forgetting curve -- that causes its effective weight to diminish over time. The system's memory is not a static database; it is a living substrate where information competes for representation based on recency, utility, and reinforcement.
- **Provenance and taint**: Every Engram knows who produced it, how trusted that producer is, and whether the data is tainted (from an unverified source, generated by an LLM, flagged by a user, or inherited from a tainted parent). Taint propagates through the lineage DAG.
- **Lineage DAG**: Every Engram records which parent Engrams it was derived from, forming a directed acyclic graph that enables causal replay, impact analysis, and forensic audit of any decision.

For IronClaw, adopting the Engram pattern would transform its current flat memory system (`src/workspace/document.rs`, `MemoryDocument` with path/content/metadata) into one with mathematical decay, multi-axis quality scores, content-addressed deduplication, lineage tracking, and taint propagation.

---

## 2. Why a Universal Data Type for Knowledge

Classical software architectures use many types: tasks, events, messages, requests, responses, records, logs. Each type has its own schema, its own storage, its own lifecycle. Adding a new capability means adding a new type, a new store, a new API. This proliferation creates a combinatorial explosion of adapters, converters, and integration surfaces.

The captured design deliberately narrows the data model: Engram is the common record type and a small trait set operates on it. Events, agent outputs, gate verdicts, knowledge entries, predictions, and tool traces can all be represented as Engrams. This gives three practical consequences:

1. **Universal composability**: Any Scorer can score any Engram. Any Store can store any Engram. Any Gate can verify any Engram. Components compose freely because they all speak the same language. You never need a new adapter to connect two subsystems -- they already understand Engrams.

2. **Full audit trails**: Every Engram carries lineage -- the `ContentHash`es of the parent Engrams it was derived from. This forms a directed acyclic graph (DAG) that can be traversed to explain any decision: why was this model chosen? What context was used? What gate verdict was rendered? Follow the lineage back to its roots.

3. **Temporal dynamics**: Every Engram decays. Knowledge fades. Pheromone signals expire. Context becomes stale. The system's "memory" is not a static database -- it is a living substrate where information has weight that changes over time. This mirrors biological memory, where memories compete for representation based on recency, utility, and reinforcement.

For IronClaw, this concept is compelling because IronClaw's current memory system (`src/workspace/`) stores documents as `MemoryDocument` structs with a UUID id, user_id, path, content string, created_at/updated_at timestamps, and a generic JSON metadata field. Adopting the Engram pattern would give every memory entry:
- Multi-axis quality scores (not just a metadata blob)
- Mathematical decay models (not just "delete after N days")
- Content-addressed identity for deduplication (not just UUID keys)
- Lineage tracking for provenance (not just a path string)
- Taint propagation for safety (not just implicit trust)

---

## 3. The Engram Struct -- Complete Anatomy

The actual Engram struct from `crates/roko-core/src/engram.rs` (lines 62-98):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### 3.1 Field Partitioning -- What Gets Hashed vs. What Doesn't

This is a critical architectural decision. The Engram's fields are partitioned into two sets:

**Identity fields (hashed -- changing these creates a new Engram)**:
- `kind` -- the semantic type
- `body` -- the payload
- `provenance.author` -- who produced this
- `provenance.taint` -- whether the data is tainted (via `is_tainted()` boolean)
- `lineage` -- parent ContentHashes
- `tags` -- all key-value pairs (BTreeMap provides deterministic ordering)

**Mutable fields (NOT hashed -- can change without changing identity)**:
- `score` -- scores can be recomputed by different Scorers
- `decay` -- decay can be adjusted (e.g., promoted from HalfLife to None)
- `created_at_ms` -- creation time is metadata, not content
- `fingerprint` -- the HDC vector is derived, not inherent
- `attestation` -- cryptographic proof added after creation
- `emotional_tag` -- affective metadata
- `balance` -- demurrage balance changes with usage

This partitioning is analogous to Protocol Buffers' field number stability rule (Kleppmann 2017, *Designing Data-Intensive Applications*, O'Reilly): identity fields are frozen; mutable fields can change without breaking existing hashes.

### 3.2 The Builder Pattern

Engrams are constructed via a builder that provides sensible defaults. From `crates/roko-core/src/engram.rs` (lines 310-343):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

Usage examples:

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### 3.3 Effective Weight

The single most important runtime calculation on an Engram is its **effective weight**, which combines score and decay into a single scalar. From `crates/roko-core/src/engram.rs` (lines 136-142):

```rust
impl Engram {
    /// The effective weight of this engram at the given current time.
    /// Combines score x decay.
    #[must_use]
    pub fn weight_at(&self, now_ms: i64) -> f32 {
        let age = now_ms - self.created_at_ms;
        self.score.effective() * self.decay.apply(age)
    }
}
```

This is the primary ordering criterion for Store queries with `min_weight` filters. An Engram that was highly scored at creation but has decayed significantly may fall below the weight threshold and be excluded from query results -- or pruned entirely by `Store::prune()`.

**Worked example -- weight decay over time**:

Consider a pheromone Engram created at time 0 with `Score::new(0.8, 0.5, 0.0, 1.0)` (effective score = 0.8 * 1.5 * 1.0 * 1.0 = 1.2) and `Decay::HalfLife { half_life_ms: 7_200_000 }` (2-hour half-life):

```
Time          Age (ms)      Decay multiplier    Effective weight
t=0           0             1.0                 1.200
t=1h          3,600,000     0.707               0.849
t=2h          7,200,000     0.500               0.600
t=4h          14,400,000    0.250               0.300
t=8h          28,800,000    0.0625              0.075
t=24h         86,400,000    0.000244            0.000293
```

At a pruning threshold of 0.01, this pheromone would be pruned after approximately 14 hours. At 0.001, after about 20 hours.

---

## 4. Content-Addressed Identity (BLAKE3)

Every Engram is identified by a `ContentHash` -- a 32-byte BLAKE3 digest of its identity fields.

### 4.1 The ContentHash Type

From `crates/roko-core/src/hash.rs` (lines 17-66):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### 4.2 Why BLAKE3 Over SHA-256

BLAKE3 was designed by Jack O'Connor, Jean-Philippe Aumasson, Samuel Neves, and Zooko Wilcox-O'Hearn and announced at Real World Crypto 2020 (O'Connor et al. 2020, "BLAKE3: one function, fast everywhere," [BLAKE3 specification](https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf)). The choice of BLAKE3 over SHA-256 is motivated by three properties:

1. **Speed**: BLAKE3 uses a Merkle tree internal structure and SIMD optimizations to achieve approximately 5-14x faster throughput than SHA-256 on modern hardware. On a single core with AVX-512, BLAKE3 achieves over 1 GB/s; with multi-threading it scales near-linearly with core count.
2. **Streaming**: BLAKE3 supports incremental hashing natively via its `Hasher` type, which matters when Engrams contain large payloads (file contents, compiled artifacts). You can feed data in chunks without buffering the entire input.
3. **Keyed mode**: BLAKE3 supports keyed hashing for MAC computation (`blake3::keyed_hash(key, data)`), useful for attestation without requiring a separate HMAC construction.

These advantages over SHA-256 come without any reduction in security: BLAKE3 provides 128-bit collision resistance (256-bit output), which is the same effective collision security as SHA-256.

### 4.3 The Exact Hash Computation

From `crates/roko-core/src/engram.rs` (lines 113-134), the `content_hash()` method feeds identity fields into a BLAKE3 hasher with pipe delimiters between fields:

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

**Key details**:
- Fields are separated by `|` pipe delimiters to prevent ambiguity
- Tags use `key=value;` format with semicolons
- `BTreeMap` provides lexicographic key order, making the hash deterministic regardless of insertion order
- The `Kind` uses `identity_key()` which handles compound kinds: `Kind::Compound([Task, Prompt])` hashes as `"compound(task+prompt)"`
- Taint contributes a single byte (`0x00` for clean, `0x01` for tainted) -- so the same content from tainted and clean sources produces **different** hashes

### 4.4 Benefits of Content-Addressing

Content-addressed storage is a well-established paradigm with deep roots in distributed systems:

1. **Deduplication**: Same content produces the same ID. `Store::put()` is **idempotent** -- re-putting the same Engram is a no-op. This property is shared with IPFS/CID (Benet 2014, "IPFS - Content Addressed, Versioned, P2P File System," arXiv:1407.3561) and Git's object model.

2. **Integrity**: Any modification to identity fields changes the hash. Corruption is detectable by recomputing the hash, a property that traces back to Merkle's hash trees (Merkle 1979, U.S. Patent 4,309,569).

3. **Linkability**: Other Engrams can reference by hash in their `lineage` field, forming tamper-evident chains analogous to blockchain data structures.

4. **Addressable storage**: Stores can be indexed by hash for O(1) lookup, avoiding sequential scans.

5. **CRDT compatibility**: The lineage DAG is structurally compatible with Merkle-CRDTs (Sanjuan et al. 2020, "Merkle-CRDTs: Merkle-DAGs meet CRDTs," arXiv:2004.00107). If two agents independently derive Engrams from the same parent, the union of their lineage graphs is well-defined by content addressing.

### 4.5 Serialization

ContentHashes serialize as 64-character hex strings in JSON via a custom serde module (`hex_bytes`) in `crates/roko-core/src/hash.rs`:

```json
{"id":"a1b2c3d4e5f6...","kind":"task","body":{"format":"text","data":"implement login"}}
```

---

## 5. 7-Axis Scoring System

Every Engram carries a multi-dimensional quality score. The Score struct from `crates/roko-core/src/score.rs` (lines 50-69) provides seven independent axes -- four primary (always populated) and three extended (opt-in):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### 5.1 The Four Primary Axes

#### Confidence -- [0, 1]

**What it measures**: How sure are we that this Engram is correct, valid, or reliable?

**Critical property**: Zero confidence produces zero effective score regardless of other axes. The formula `effective = confidence x ...` enforces this structurally. False positives are never prioritized.

**Update sources**: Gate verdicts (pass/fail), prediction tracking, human ratings, source verification.

**Connection to Active Inference**: Maps to Friston's (2010) precision weighting -- prediction errors are weighted by their precision (inverse variance) to determine how much they should update the generative model (Friston 2010, "The free-energy principle: a unified brain theory?", *Nature Reviews Neuroscience* 11(2):127-138).

#### Novelty -- [0, 1]

**What it measures**: How new or surprising is this Engram compared to what the system already knows?

**Role in scoring**: Acts as a multiplicative bonus via `(1 + novelty)`. An Engram with novelty 0.0 has a multiplier of 1.0x; with novelty 1.0, a multiplier of 2.0x. Novel information is prioritized without penalizing routine information.

**Connection to information theory**: Novelty can be approximated by compression ratio as a Kolmogorov complexity proxy (Kolmogorov 1965, "Three approaches to the quantitative definition of information," *Problems of Information Transmission* 1(1):1-7): `ratio = len(compress(body)) / len(body)`. High ratio = incompressible = likely novel. Also maps to Bayesian surprise: `S(data) = KL[P(M|data) || P(M)]` (Itti and Baldi 2005, "A principled approach to detecting surprising events in video," *CVPR*; Schmidhuber 2010, "Formal theory of creativity, fun, and intrinsic motivation," *IEEE Transactions on Autonomous Mental Development* 2(3):230-247).

#### Utility -- [0, infinity)

**What it measures**: How pragmatically useful has this Engram proven to be? Utility accumulates over time as the Engram is referenced, used in compositions, or leads to successful outcomes.

**Range**: Unbounded above. Unlike confidence and novelty (clamped to [0,1]), utility grows without limit. A playbook rule applied 50 times with positive outcomes will have high utility.

**Role in scoring**: Like novelty, acts as a multiplicative bonus via `(1 + utility)`. With utility 5.0, the multiplier is 6.0x.

#### Reputation -- `crates/roko-core/src/provenance.rs` lines 297-347):
- `Provenance::trusted()` --> trust 1.0, taint Clean
- `Provenance::agent()` --> trust 0.75, taint Clean
- `Provenance::user()` --> trust 0.5, taint UserInput
- `Provenance::external()` --> trust 0.1, taint UnverifiedSource

### 5.2 The Three Extended Axes

These three axes complete the full 7-axis appraisal. They default to 0.0 (opt-in) and use `#[serde(default)]` for backward compatibility:

#### Precision -- [0, 1]
How specific and well-defined is this Engram's content? **Intentionally excluded from the `effective()` formula** -- it describes applicability narrowness, not quality. Consumed separately by routers that need specificity ranking.

#### Salience -- [0, 1]
How relevant is this Engram to the current context? When non-zero, applies a soft damping factor: `salience_factor = 0.5 + 0.5 * salience`. When zero (default), the factor is 1.0 (no effect).

#### Coherence -- [0, 1]
How consistent is this Engram with the system's existing knowledge base? Like salience, applies a soft damping factor when non-zero. Connection to Minimum Description Length: `coherence = 1.0 - (L(D|M) / L(D|null_model))` (Gruenwald 2007, *The Minimum Description Length Principle*, MIT Press).

### 5.3 The Effective Score Formula

All seven axes collapse into a single scalar via a 6-factor formula. From `crates/roko-core/src/score.rs` (lines 179-201):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

The formula in mathematical notation:

```
effective = confidence
          x (1 + novelty)
          x (1 + utility)
          x reputation
          x salience_factor
          x coherence_factor

where:
  salience_factor  = 1.0           if salience == 0
                   = 0.5 + 0.5s    otherwise
  coherence_factor = 1.0           if coherence == 0
                   = 0.5 + 0.5c    otherwise
```

**Design rationale**: The extended factors are opt-in soft damping. When salience and coherence are zero (the default for `Score::new()`), the 6-factor formula reduces to the original 4-factor formula: `confidence x (1 + novelty) x (1 + utility) x reputation`. Precision remains excluded because it describes applicability narrowness, not quality. The choice of seven dimensions connects to Miller's (1956) "magical number seven" channel capacity (*Psychological Review* 63(2):81-97) and Scherer's (2001) component process model of appraisal which uses 5-7 evaluation dimensions (*Applied AI* 15:5-131).

**Formula properties**:

| Property | Guarantee | Why It Matters |
|---|---|---|
| `confidence = 0` -> `effective = 0` | Zero confidence kills the score | Invalid information is never prioritized |
| `reputation = 0` -> `effective = 0` | Zero reputation kills the score | Untrusted sources are structurally excluded |
| `novelty = 0` -> multiplier = 1.0 | No penalty for routine information | Routine is normal, not bad |
| `novelty = 1` -> multiplier = 2.0 | Novel information gets 2x priority | Surprise drives attention |
| `utility = 0` -> multiplier = 1.0 | New Engrams start at baseline | No penalty for lack of history |
| `utility = n` -> multiplier = `(1+n)` | Utility accumulates multiplicatively | Frequently-useful Engrams dominate |
| Non-finite input -> 0.0 | NaN/Inf safety | Numeric robustness guardrail |

**Worked examples**:

```
Score::NEUTRAL  // confidence=0.5, novelty=0, utility=0, reputation=1
--> 0.5 x 1.0 x 1.0 x 1.0 = 0.5

Verified novel insight from trusted source:
Score { confidence: 0.95, novelty: 0.8, utility: 0, reputation: 1.2 }
--> 0.95 x 1.8 x 1.0 x 1.2 = 2.052

Highly-utilized playbook rule:
Score { confidence: 0.9, novelty: 0, utility: 5.0, reputation: 1.0 }
--> 0.9 x 1.0 x 6.0 x 1.0 = 5.4

Untrusted external observation:
Score { confidence: 0.8, novelty: 1.0, utility: 0, reputation: 0.1 }
--> 0.8 x 2.0 x 1.0 x 0.1 = 0.16

With extended axes (salience=0.8, coherence=0.6):
Score { confidence: 0.9, novelty: 0.5, utility: 1.0, reputation: 1.0, salience: 0.8, coherence: 0.6 }
salience_factor = 0.5 + 0.5*0.8 = 0.9
coherence_factor = 0.5 + 0.5*0.6 = 0.8
--> 0.9 x 1.5 x 2.0 x 1.0 x 0.9 x 0.8 = 1.944
```

### 5.4 Score Constants and Arithmetic

```rust
impl Score {
    pub const ZERO: Self = Self {
        confidence: 0.0, novelty: 0.0, utility: 0.0, reputation: 0.0,
        precision: 0.0, salience: 0.0, coherence: 0.0,
    };
    pub const NEUTRAL: Self = Self {
        confidence: 0.5, novelty: 0.0, utility: 0.0, reputation: 1.0,
        precision: 0.0, salience: 0.0, coherence: 0.0,
    };
}
```

Scores support element-wise arithmetic:
- **Multiplication** (`Score * Score`) -- scales each axis independently (for per-axis modifiers)
- **Addition** (`Score + Score`) -- aggregates evidence from multiple scorers (confidence/novelty clamped to 1.0; utility/reputation accumulate unbounded)

### 5.5 Numerical Safety

All Score constructors sanitize non-finite values (from `crates/roko-core/src/score.rs` lines 29-43):

```rust
fn finite_unit_interval(value: f32) -> f32 {
    if value.is_finite() { value.clamp(0.0, 1.0) } else { 0.0 }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 { value } else { 0.0 }
}
```

`Score::new(f32::NAN, f32::INFINITY, f32::NEG_INFINITY, f32::NAN)` produces `Score::ZERO`. The `effective()` method returns 0.0 if any axis is non-finite.

---

## 6. Four Decay Variants

Every Engram has a decay function that determines how its weight diminishes over time. The `Decay` enum from `crates/roko-core/src/decay.rs` (lines 18-69):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### 6.1 Decay::None -- Permanent Weight

**Mathematical formulation**: `weight(t) = 1.0` for all `t`.

**Use cases**: Core identity, schemas, configuration records, fixed policy records. Things that should never expire unless explicitly replaced.

### 6.2 Decay::HalfLife -- Exponential Decay

**Mathematical formulation**:

```
weight(age) = 0.5 ^ (age_ms / half_life_ms)
            = 2^(-age_ms / half_life_ms)
            = e^(-age_ms * ln(2) / half_life_ms)
```

**Properties**:
- At `age = 0`: weight = 1.0
- At `age = half_life_ms`: weight = 0.5
- At `age = 2 * half_life_ms`: weight = 0.25
- At `age = n * half_life_ms`: weight = 0.5^n
- Asymptotically approaches 0 but never reaches it

**Implementation** (from `crates/roko-core/src/decay.rs` lines 93-99):

```rust
Self::HalfLife { half_life_ms } => {
    if *half_life_ms == 0 { return 0.0; }
    let hl = *half_life_ms as f32;
    finite_weight((0.5_f32).powf(age_ms / hl))
}
```

**Predefined constants** for the agent-chain pheromone system (lines 126-144):

```rust
impl Decay {
    pub const THREAT: Self = Self::HalfLife { half_life_ms: 7_200_000 };       // 2 hours
    pub const OPPORTUNITY: Self = Self::HalfLife { half_life_ms: 14_400_000 }; // 4 hours
    pub const WISDOM: Self = Self::HalfLife { half_life_ms: 86_400_000 };      // 24 hours
    pub const GATE_VERDICT: Self = Self::HalfLife { half_life_ms: 86_400_000 };// 24 hours
}
```

### 6.3 Decay::Ttl -- Hard Expiration

**Mathematical formulation**:

```
weight(age) = | 1.0   if age_ms < ttl_ms
              | 0.0   if age_ms >= ttl_ms
```

This is a step function -- full weight until the TTL, then instant death.

**Design note**: The TTL is a **relative duration** (milliseconds since emission), not an absolute timestamp. `Decay::apply()` takes a relative `age_ms` parameter. Absolute deadlines are handled at a higher layer: the emitter computes `ttl_ms = deadline - now` at construction time.

**Use cases**: Strict validity windows (offers, bounties), session-scoped data, tool output whose validity is strictly time-bound.

### 6.4 Decay::Ebbinghaus -- Forgetting Curve

**Mathematical formulation**:

```
weight(age) = exp(-age_ms / (strength * scale_ms))
            = e^(-age_ms / (strength * scale_ms))
```

Where:
- `strength` (float, `crates/roko-core/src/decay.rs` lines 107-114):

```rust
Self::Ebbinghaus { strength, scale_ms } => {
    if *scale_ms == 0 || !strength.is_finite() || *strength <= 0.0 {
        return 0.0;
    }
    let scale = (*strength) * (*scale_ms as f32);
    finite_weight((-age_ms / scale).exp())
}
```

**Connection to psychology**: Named after Hermann Ebbinghaus, who in 1885 published *Uber das Gedachtnis* ("Memory: A Contribution to Experimental Psychology"), documenting the first experimental study of memory decay. Through self-experimentation with nonsense syllables (CVC trigrams like "WID" and "ZOF"), he discovered that retention follows an approximately exponential curve: `R = e^(-t/S)`, where R is retention, t is time since learning, and S is a memory strength parameter that increases with repetition and spaced practice (Ebbinghaus 1885; Murre and Dros 2015, "Replication and Analysis of Ebbinghaus' Forgetting Curve," *PLOS ONE* 10(7):e0120644). Roko's `strength` parameter directly maps to Ebbinghaus's S.

**Ebbinghaus with tier shaping**: In Roko's Neuro subsystem, the effective half-life combines a knowledge-type base with a validation tier multiplier:

| Knowledge Type | Base Half-Life |
|---|---|
| Warning | 7 days |
| StrategyFragment | 14 days |
| Insight | 30 days |
| CausalLink | 60 days |
| Heuristic | 90 days |
| Fact | 365 days |

| Validation Tier | Multiplier |
|---|---|
| Transient | 0.1x |
| Working | 0.5x |
| Consolidated | 1.0x |
| Persistent | 5.0x |

A Transient Warning decays with an effective half-life of 0.7 days (0.1 x 7). A Persistent Fact decays with an effective half-life of 1825 days (5.0 x 365 = 5 years). This creates a dynamic range spanning four orders of magnitude.

### 6.5 The `is_alive` Check

From `crates/roko-core/src/decay.rs` (lines 117-121):

```rust
impl Decay {
    pub fn is_alive(&self, age_ms: i64, threshold: f32) -> bool {
        threshold.is_finite() && self.apply(age_ms) > threshold
    }
}
```

Used by garbage collection and pruning routines. Note that negative ages (clock skew) return `1.0` -- the implementation handles this at lines 87-89.

---

## 7. Demurrage -- Attention Economics

Beyond the four decay variants, Roko defines a **demurrage** system that models knowledge retention as an attention economy. The concept is borrowed from Silvio Gesell's 1916 *The Natural Economic Order* (Gesell 1916), which proposed that money should carry a holding cost ("demurrage") to encourage circulation. Just as Gesellian demurrage taxes idle money to prevent hoarding, Roko's demurrage taxes idle knowledge to ensure active validation.

From `crates/roko-core/src/demurrage.rs` (lines 9-32):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

**Mathematical formulation**:

```
balance(t + dt) = balance(t) * (1 - rate) ^ dt

where:
  dt   = elapsed time in hours
  rate = hourly decay rate in [0, 1]
```

This is compounding decay. With a rate of 0.01 per hour:

```
After 1 hour:     balance = 0.99^1   = 0.990
After 24 hours:   balance = 0.99^24  = 0.786
After 100 hours:  balance = 0.99^100 = 0.366
After 200 hours:  balance = 0.99^200 = 0.134
After 459 hours:  balance = 0.99^459 = 0.010 (depleted)
```

**The key insight**: Demurrage is fundamentally different from time-based decay. Decay is passive -- the weight drops regardless of activity. Demurrage models an **attention economy** where knowledge must earn its place:

- **Reinforced knowledge stays warm** -- the `touch()` method resets balance to 1.0
- **Unused knowledge loses balance** gradually instead of waiting for a hard prune
- **Frozen knowledge can be thawed** without breaking lineage (cold storage, not deletion)

The Engram struct includes a `balance` field (default 1.0) and a `touch()` method (line 150-153):

```rust
impl Engram {
    pub fn touch(&mut self) {
        self.balance = 1.0;
    }
}
```

**Reinforcement types** (from design docs):

| Kind | Meaning | Proposed Bonus |
|---|---|---|
| `Cited` | Engram participates in another's lineage | 0.05 |
| `Retrieved` | Engram solved a query | 0.02 |
| `Gated` | Engram survived verification | 0.03 |
| `Surprised` | Engram was informationally novel | 0.15 |
| `AgentQuoted` | Another agent turned it into output | 0.08 |

---

## 8. Kind -- Semantic Type System

The `Kind` enum from `crates/roko-core/src/kind.rs` (lines 23-109) tells consumers how to interpret an Engram's body. There are 30+ built-in variants organized by architectural concern:

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

**Compound kinds**: A single Engram can carry multiple semantics. `Kind::compound(&[Kind::GateVerdict, Kind::Metric])` represents a gate verdict that is also a metric reading. The `matches()` method handles constituent lookup, and `identity_key()` produces a deterministic hash representation like `"compound(gate_verdict+metric)"`.

**Extensibility**: The enum is `#[non_exhaustive]` and has a `Custom(String)` escape hatch. Extensions use reverse-DNS prefixes to avoid collisions: `Kind::Custom("com.example.widget".into())`.

---

## 9. Body -- Typed Payload

The `Body` enum from `crates/roko-core/src/body.rs` (lines 14-25) carries the Engram's actual content in one of four formats:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "format", content = "data", rename_all = "snake_case")]
pub enum Body {
    Empty,                                              // marker signal
    Text(String),                                       // UTF-8 text
    Json(serde_json::Value),                            // structured JSON
    Bytes(#[serde(with = "base64_bytes")] Vec<u8>),     // raw bytes (base64 in JSON)
}
```

Key methods:
- `body.canonical_bytes()` -- stable JSON serialization for content hashing (uses `serde_json::to_vec(self)`)
- `body.byte_size()` -- approximate payload size for budget tracking
- `body.as_json::<T>()` -- typed JSON deserialization
- `body.as_text()` -- text extraction (returns `Result`, errors on wrong variant)
- `body.kind_hint()` -- human-readable format name ("empty", "text", "json", "bytes")

**Canonical encoding**: The `canonical_bytes()` method ensures that identical bodies always produce the same bytes, critical for content hashing. The tagged serde format (`{"format":"text","data":"..."}`) means different Body variants with similar content produce different canonical bytes -- `Body::text("hello")` and `Body::bytes(b"hello")` hash differently.

---

## 10. Lineage DAG

The `lineage` field on every Engram is a `Vec<ContentHash>` identifying the parent Engrams from which this Engram was derived. This forms a directed acyclic graph (DAG) that enables:

- **Causal replay**: Trace any decision back to its inputs by following lineage chains
- **Impact analysis**: Find all Engrams that depend on a given input (forward graph traversal)
- **Autocatalytic metrics**: Measure how many downstream Engrams an input catalyzed
- **Forensic audit**: Reconstruct the complete chain of reasoning for any output

### 10.1 Derivation Methods

From `crates/roko-core/src/engram.rs` (lines 167-193):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

The `derive_verdict()` method carries forward the **entire** known lineage chain (via `derived_lineage()`, which appends `self.id` to all existing lineage entries, deduplicating), preserves all parent tags (with child tags overriding on collision), and applies the standard `Decay::GATE_VERDICT` (24-hour half-life).

### 10.2 DAG Traversal

The lineage DAG can be traversed by querying the Store for each parent ContentHash (BFS traversal with cycle detection via `HashSet`):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

---

## 11. Provenance and Taint Propagation

Every Engram carries a `Provenance` record that answers three questions: who produced this, how trusted are they, and is the data tainted?

### 11.1 The Provenance Struct

From `crates/roko-core/src/provenance.rs` (lines 263-292):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### 11.2 The Taint Enum -- All 9 Variants

The `Taint` enum from `crates/roko-core/src/provenance.rs` (lines 21-67) has 9 variants. Each variant is `#[non_exhaustive]` so new taint reasons can be added without breaking downstream matches:

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

The taint system implements a form of **dynamic taint analysis** (Schwartz et al. 2010, "All You Ever Wanted to Know About Dynamic Taint Analysis and Forward Symbolic Execution," *IEEE S&P*). Data from untrusted sources is tagged at the boundary and the tag propagates through derivation chains via the `Propagated` variant.

### 11.3 Taint Propagation Rules

**Taint affects the content hash**: The taint boolean (`is_tainted()`) is included in the content hash computation. This means that the same content from a tainted source and a clean source produces **different** ContentHashes -- you cannot substitute a tainted Engram for a clean one.

The `Propagated` variant is the mechanism for taint propagation through the lineage DAG:

```rust
Taint::Propagated {
    detail: "inherited from parent".into(),
    inherited_from: Some(parent_engram.id),
}
```

The `inherited_from` field provides a direct link back to the originating tainted Engram, enabling forensic audit of how taint spread through the system.

### 11.4 Provenance Constructors

From `crates/roko-core/src/provenance.rs` (lines 294-347):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### 11.5 Trust Checking

From `crates/roko-core/src/provenance.rs` (lines 409-412):

```rust
impl Provenance {
    pub fn is_trusted(&self, min_trust: f32) -> bool {
        self.trust >= min_trust && !self.is_tainted()
    }
}
```

The `is_trusted()` method requires **both** sufficient trust score **and** no taint. A tainted Engram from a high-trust author is still not trusted.

---

## 12. Cryptographic Attestation

Engrams support optional cryptographic proof of origin via Ed25519 signatures. From `crates/roko-core/src/attestation.rs` (lines 54-64):

```rust
/// Cryptographic proof that a specific signer produced an Engram.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
    /// Ed25519 signature over the Engram's content hash.
    pub signature: Ed25519Signature,  // [u8; 64]
    /// Public key of the signer or attesting runtime.
    pub public_key: PublicKey,        // [u8; 32]
    /// Optional chain witness for timestamped publication.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_attestation: Option<ChainAttestation>,
}
```

The sign/verify workflow (from `crates/roko-core/src/attestation.rs` lines 87-109):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

**Key design decisions**:
- Attestations are **excluded** from the content hash, so an Engram can be attested after creation without changing its ID
- The `witness_hash()` method excludes `chain_attestation` so the anchored proof stays stable before and after on-chain publication
- Chain attestation (`ChainAttestation { chain_id, tx_hash, block_number }`) provides timestamped proof-of-existence on a blockchain

---

## 13. HDC Fingerprinting and Semantic Search

Every Engram can carry an HDC (Hyperdimensional Computing) fingerprint for fast similarity search. For full coverage of the HDC system — theory, all four algebraic operations, capacity analysis, benchmarks, and IronClaw integration plan — see **[core-concepts/hyperdimensional-computing/](./hyperdimensional-computing/README.md)**. What follows is the Engram-specific surface.

The fingerprint field from `crates/roko-core/src/engram.rs` (lines 17-27):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HdcFingerprint {
    /// The semantic fingerprint vector for this engram.
    pub vector: HdcVector,      // 10,240-bit binary vector (1,280 bytes)
    /// Monotonic version of the encoder used to derive vector.
    pub encoder_version: u32,
}
```

Note that `fingerprint` is excluded from the content hash — it is a derived artifact, not part of identity. The vector can be regenerated from the body content at any time.

The three algebraic operations exposed on `Engram` (lines 270-291):

| Operation | HDC op | Engram meaning |
|---|---|---|
| **bind** | XOR | Associate two Engrams (key-value pair) |
| **bundle** | Majority vote | Cluster centroid / composite of a set |
| **at_position** | Cyclic bit shift | Encode temporal ordering in a sequence |

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

Similarity search is wired through the `Store` trait's `query_similar()` method (see Section 14.1). For HDC-backed lookup, a single fixed-width comparison is cheap but a brute-force scan is still O(N); use the HDC benchmark harness to set local latency targets.

---

## 14. The Core Traits

The entire Roko system is built from the Engram and traits defined in `crates/roko-core/src/traits.rs`. These traits define the complete operational surface:

### 14.1 Store (lines 37-80)

The persistence abstraction. All storage backends implement this trait. For concrete database implementations (PostgreSQL, libSQL/Turso backends, migration SQL, and the IronClaw `src/workspace/` integration) see **[context-memory/persistence-storage.md](../context-memory/persistence-storage.md)**.

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### 14.2 ColdStore (lines 101-151)

Archival store for aged-out Engrams. Methods: `archive()`, `thaw()`, `purge_before()`. Migration flow: `Store (hot) -> ColdStore (cold/archive)`. Critical for IronClaw's "LLM data is never deleted" invariant. See [context-memory/persistence-storage.md](../context-memory/persistence-storage.md) for the cold-storage implementation plan.

### 14.3 Score trait (lines 167-198)

Rates Engrams along the 7 axes. Pure functions of `(engram, context)`. Polymorphic over Engrams and Pulses via `Datum` dispatch.

```rust
#[async_trait]
pub trait Scorer: Send + Sync {
    async fn score(&self, datum: Datum<'_>, ctx: &ScoringContext) -> Result<Score, ScorerError>;
}
```

### 14.4 Verify (Gate) (lines 214-226)

Verifies Engrams against ground truth (compile, run tests, simulate transactions). Returns a `Verdict`.

```rust
#[async_trait]
pub trait Verify: Send + Sync {
    async fn verify(&self, engram: &Engram) -> Result<Verdict, VerifyError>;
}
```

### 14.5 Route (lines 242-267)

Selects one Engram from many candidates and can learn via `feedback()`. Captured implementations include static, contextual-bandit, cascade, and weighted routers; IronClaw routing docs should use the current term **14D Cascade Router** for model/tool routing work.

```rust
#[async_trait]
pub trait Route: Send + Sync {
    async fn choose(&self, candidates: &[Engram], ctx: &RoutingContext)
        -> Result<ContentHash, RouteError>;
    async fn feedback(&self, choice: ContentHash, reward: f32) -> Result<(), RouteError>;
}
```

### 14.6 Compose (lines 285-320)

Combines multiple Engrams into one under a `Budget`. Primary use case: assembling prompts from PromptSection Engrams under a token budget.

```rust
#[async_trait]
pub trait Compose: Send + Sync {
    async fn compose(&self, parts: &[Engram], budget: &Budget)
        -> Result<Engram, ComposeError>;
}
```

### 14.7 React (Policy) (lines 339-365)

Watches streams of Engrams/Pulses and emits interventions. The reactive/behavioral layer: conductor watchers, circuit breakers, episode logging, pheromone reactions.

```rust
#[async_trait]
pub trait React: Send + Sync {
    async fn react(&self, datum: Datum<'_>) -> Result<Vec<Engram>, ReactError>;
}
```

### 14.8 Bus (lines 385-395)

Publish/subscribe transport for ephemeral Pulses. `publish()` returns a monotonic sequence number; `subscribe()` takes a `TopicFilter`.

```rust
#[async_trait]
pub trait Bus: Send + Sync {
    async fn publish(&self, pulse: Pulse) -> Result<u64, BusError>;
    async fn subscribe(&self, filter: TopicFilter) -> Result<PulseStream, BusError>;
}
```

---

## 15. The Engram/Pulse Duality

Roko has two data mediums, not one:

| Property | Signal/Engram (durable) | Pulse (ephemeral) |
|---|---|---|
| **Identity** | Content hash (BLAKE3) | `(topic, seq)` tuple |
| **Durability** | Store (JSONL, knowledge store) | Ring buffer on Bus (~4,096 entries) |
| **Lineage** | Full `Vec<ContentHash>` DAG | Optional `lineage_hint` |
| **Scoring** | 7-dimensional Score | None |
| **Retention** | Demurrage (Gesell 1916) | Ring buffer eviction |
| **HDC fingerprint** | 10,240-bit binary vector | None (too transient) |
| **Typical rate** | 1 Hz - 1 kHz | 1 Hz - 1 MHz |
| **Typical lifetime** | Minutes to permanent | Milliseconds to seconds |

The `Datum` enum from `crates/roko-core/src/datum.rs` (lines 34-40) provides a polymorphic input surface:

```rust
pub enum Datum<'a> {
    Engram(&'a Engram),
    Pulse(&'a Pulse),
}
```

The only bridges between the two mediums are explicit:
- **Graduation**: `Pulse -> Signal` via `Engram::from_pulse_synthetic()` (the only path from transport into the audit DAG)
- **Projection**: `Signal -> Pulse` (lossy broadcast of stored Signals)

The `signal.rs` module provides a forward-compatible alias for the ongoing Engram -> Signal rename:

```rust
pub use crate::engram::{Engram as Signal, EngramBuilder as SignalBuilder, HdcFingerprint};
```

---

## 16. Engram Lifecycle -- A Worked Example

This section traces how Engrams flow through a real scenario: a user asks the agent to fix a compilation error.

**Step 1 -- User input arrives**

```rust
let user_msg = Engram::builder(Kind::Task)
    .body(Body::text("Fix the compilation error in auth.rs"))
    .provenance(Provenance::user("alice"))    // trust=0.5, taint=UserInput
    .decay(Decay::None)                       // tasks don't decay
    .build();
// user_msg.score = Score::NEUTRAL (confidence=0.5, reputation=1.0)
// user_msg.id = ContentHash(blake3 of kind|body|author|taint|lineage|tags)
```

**Step 2 -- Agent produces a code patch (derived from user task)**

```rust
let patch = user_msg.derive(Kind::AgentOutput,
        Body::text("--- a/auth.rs\n+++ b/auth.rs\n..."))
    .provenance(Provenance::agent("code_agent"))  // trust=0.75
    .score(Score::new(0.7, 0.3, 0.0, 0.75))      // moderate confidence, some novelty
    .build();
// patch.lineage == [user_msg.id]   -- derived from the user's task
```

**Step 3 -- Gate verification (compilation check)**

```rust
let verdict = patch.derive_verdict(Body::text("compilation succeeded"))
    .score(Score::new(1.0, 0.0, 0.0, 1.0))     // full confidence -- gate passed
    .tag("passed", "true")
    .build();
// verdict.lineage == [user_msg.id, patch.id]   -- full chain
// verdict.decay == Decay::GATE_VERDICT (24h half-life)
// verdict.kind == Kind::GateVerdict
```

**Step 4 -- Score evolution over time**

After the patch is applied successfully 5 times:

```rust
patch.score.utility = 5.0;  // accumulated over 5 successful uses
// New effective score: 0.7 * 1.3 * 6.0 * 0.75 = 4.095
// Compare to original: 0.7 * 1.3 * 1.0 * 0.75 = 0.6825
// The patch is now 6x more important in ranking
```

**Step 5 -- Demurrage and lifecycle**

```
Time      Balance    Event
t=0       1.0        Patch created, stored
t=24h     0.786      Demurrage tick (rate=0.01/hr): 0.99^24
t=24h     1.0        Patch cited by another agent -> touch() resets balance
t=168h    0.232      Unused for a week: 0.99^144 = 0.232
t=459h    0.010      Balance falls to depleted threshold
                      -> Move to ColdStore (archive, not delete)
                      -> Can be thawed if lineage traversal needs it
```

**Step 6 -- Taint propagation**

If the user task referenced external data:

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

---

## 17. Mermaid Diagrams

### 17.1 Engram Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Created: builder().build()
    Created --> Active: Store::put() succeeds
    Active --> Active: touch() / replenish()\nresets balance to 1.0
    Active --> Decaying: Time passes\nbalance decreases
    Decaying --> Active: Cited / Retrieved\ntouch() called
    Decaying --> Archived: balance < 0.1\nor weight < threshold\nColdStore::archive()
    Archived --> Active: ColdStore::thaw()\nlineage traversal needed
    Active --> GarbageCollected: Store::prune()\nweight < min_weight
    note right of GarbageCollected
        Only for non-LLM data.\nIronClaw: LLM data never deleted.
        ColdStore preserves lineage.
    end note
```

### 17.2 Content Hash Computation Flow

```mermaid
flowchart TD
    E[Engram fields] --> KIK[kind.identity_key()]
    E --> CBC[body.canonical_bytes()]
    E --> AUT[provenance.author]
    E --> TAI["is_tainted() -> 0x00 or 0x01"]
    E --> LIN["lineage: Vec&lt;ContentHash&gt;"]
    E --> TAGS["tags: BTreeMap&lt;String,String&gt;"]

    KIK --> H{BLAKE3 Hasher}
    CBC --> H
    AUT --> H
    TAI --> H
    LIN --> H
    TAGS --> H

    H -->|finalize()| CH[ContentHash\n32 bytes]

    EXCL["Excluded from hash:\nscore, decay, created_at_ms,\nfingerprint, attestation,\nemotional_tag, balance"] -.->|NOT fed| H

    style EXCL fill:#ffeeee,stroke:#cc0000
    style CH fill:#eeffee,stroke:#00aa00
```

### 17.3 Taint Propagation DAG

```mermaid
flowchart LR
    subgraph Untainted
        A["Task A\nClean\ntrust=0.5"]
        B["Tool Result B\nClean\ntrust=0.75"]
    end

    subgraph Tainted Sources
        C["External Feed C\nUnverifiedSource\ntrust=0.1"]
        D["LLM Output D\nLlmHallucination\ntrust=0.5"]
    end

    subgraph Derived
        E["Analysis E\nPropagated from C\nis_tainted=true"]
        F["Summary F\nPropagated from D\nis_tainted=true"]
        G["Verdict G\nClean (from A+B only)\nis_tainted=false"]
        H["Mixed H\nPropagated (any parent tainted)\nis_tainted=true"]
    end

    A -->|lineage| G
    B -->|lineage| G
    C -->|lineage| E
    D -->|lineage| F
    A -->|lineage| H
    E -->|lineage| H

    style C fill:#ffeeee,stroke:#cc0000
    style D fill:#ffeeee,stroke:#cc0000
    style E fill:#fff3ee,stroke:#ff8800
    style F fill:#fff3ee,stroke:#ff8800
    style H fill:#fff3ee,stroke:#ff8800
    style G fill:#eeffee,stroke:#00aa00
```

### 17.4 Lineage Tracking Graph

```mermaid
flowchart TD
    U["user_msg\nKind::Task\nid: a1b2..."]
    P["patch\nKind::AgentOutput\nid: c3d4..."]
    V["verdict\nKind::GateVerdict\nid: e5f6..."]
    E["episode\nKind::Episode\nid: g7h8..."]
    INS["insight\nKind::Insight\nid: i9j0..."]

    U -->|"lineage[0]"| P
    P -->|"lineage[0]"| V
    U -->|"lineage[1] (via derive_verdict)"| V
    V -->|"lineage[0]"| E
    P -->|"lineage[1]"| E
    E -->|"lineage[0]"| INS

    style U fill:#e6f3ff,stroke:#0066cc
    style P fill:#e6f3ff,stroke:#0066cc
    style V fill:#e6ffe6,stroke:#006600
    style E fill:#fff3e6,stroke:#cc6600
    style INS fill:#f3e6ff,stroke:#6600cc
```

### 17.5 Trait Composition Architecture

```mermaid
graph TB
    ENG[Engram\nThe Universal Datum]

    subgraph Persistence
        S[Store\nput/get/query/prune]
        CS[ColdStore\narchive/thaw/purge]
    end

    subgraph Quality
        SC[Scorer\nscore Datum]
        VF[Verify/Gate\nverify -> Verdict]
    end

    subgraph Routing
        RT[Route\nchoose + feedback]
        CM[Compose\nparts + budget]
    end

    subgraph Reactivity
        RC[React\nreact -> Vec&lt;Engram&gt;]
        BS[Bus\npublish/subscribe Pulse]
    end

    ENG --> S
    ENG --> SC
    ENG --> VF
    ENG --> RT
    ENG --> CM
    ENG --> RC
    S --> CS
    BS -.->|"graduation\n(Pulse -> Engram)"| S

    style ENG fill:#ffe6cc,stroke:#cc6600,font-weight:bold
```

### 17.6 Engram vs Pulse Duality

```mermaid
flowchart LR
    subgraph Durable["Durable Layer (Engrams)"]
        direction TB
        E1[Engram\nContent Hash\n7-axis Score\nLineage DAG\nDemurrage]
        STORE[Store\nHot / Cold]
        E1 <--> STORE
    end

    subgraph Ephemeral["Ephemeral Layer (Pulses)"]
        direction TB
        P1[Pulse\ntopic + seq\nNo Score\nRing Buffer]
        BUS[Bus\n~4096 entries]
        P1 <--> BUS
    end

    BUS -->|"Graduation\nfrom_pulse_synthetic()"| STORE
    STORE -->|"Projection\n(lossy broadcast)"| BUS

    DATUM["Datum&lt;'a&gt;\nEngram | Pulse\npolymorphic scoring"]

    E1 -.-> DATUM
    P1 -.-> DATUM

    style DATUM fill:#e6e6ff,stroke:#6666cc
```

---

## 18. Benchmarking

### 18.1 Hash Computation Throughput

BLAKE3 hash throughput depends on payload size and CPU features. Benchmark methodology: hash 1M random Engrams of varying body sizes, measuring wall time.

**Theoretical baselines (BLAKE3 team measurements)**:

| Platform | Single-thread | With SIMD | Multi-thread (8 cores) |
|---|---|---|---|
| x86_64 AVX-512 | ~1.3 GB/s | ~3.0 GB/s | ~24 GB/s |
| x86_64 AVX2 | ~800 MB/s | ~2.0 GB/s | ~16 GB/s |
| ARM64 NEON | ~600 MB/s | ~1.2 GB/s | ~9.6 GB/s |

**Engram-specific benchmark design** (targeting IronClaw MemoryDocument sizes):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

**Validation targets** (estimates to remeasure on IronClaw hardware):

| Payload Size | ns/op | Throughput |
|---|---|---|
| 64 bytes (tiny note) | ~50 ns | ~1.3 GB/s |
| 1 KB (typical note) | ~350 ns | ~2.9 GB/s |
| 10 KB (large doc) | ~3,200 ns | ~3.1 GB/s |
| 100 KB (file content) | ~32,000 ns | ~3.1 GB/s |

Treat these numbers as baseline targets only; local measurements should decide whether hashing is material on the write path.

### 18.2 Score Calculation Overhead

The `effective()` formula is a pure arithmetic computation with 6 floating-point multiplications and 4 conditional branches:

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

**Validation target**: ~2-5 ns per `effective()` call on modern CPUs. Re-measure before relying on this as a hot-path assumption.

### 18.3 Decay Simulation Performance

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

**Validation target**: HalfLife and Ebbinghaus calculations should remain small relative to retrieval and DB costs; benchmark locally because libm and CPU behavior vary.

### 18.4 Deduplication Hit Rate Analysis

Content-addressed deduplication eliminates redundant writes. Hit rate depends on workload:

| Workload Type | Expected Dedup Rate | Notes |
|---|---|---|
| Heartbeat re-writes (HEARTBEAT.md) | 0% (always new content) | Content changes each run |
| Identity documents (IDENTITY.md) | 80-95% | Rarely changes |
| Tool output (shell/file_read) | 20-50% | Repeated file reads |
| LLM-generated content | <5% | High entropy |
| Structured notes (USER.md) | 60-80% | Mostly stable |

**Storage efficiency** -- bytes saved per 1000 writes at 50% dedup rate and 1 KB average doc:
```
Storage saved = 1000 writes * 0.50 dedup rate * 1 KB = 500 KB
Index savings = 1000 * 0.50 * (sizeof(embedding) = 4 * 1536 bytes) = 3 MB
```

At 50% dedup rate, eliminating redundant embedding computations (1536-dim float vectors at 6 KB each) saves ~3 MB per 1000 writes -- more valuable than the raw content savings.

### 18.5 Storage Efficiency Analysis

Overhead breakdown per Engram vs. current IronClaw MemoryDocument:

| Component | Current MemoryDocument | Engram-enhanced | Delta |
|---|---|---|---|
| UUID id | 16 bytes | + ContentHash 32 bytes | +16 bytes |
| Metadata JSON | variable (~100 bytes) | + score fields (7 x 4 bytes = 28 bytes) | +28 bytes |
| No decay | 0 bytes | + decay variant + params (~20 bytes) | +20 bytes |
| No lineage | 0 bytes | + derived_from JSON array (~N x 64 chars) | +N x 64 bytes |
| No taint | 0 bytes | + source_type + is_tainted (~10 bytes) | +10 bytes |
| No balance | 0 bytes | + balance f64 (8 bytes) | +8 bytes |

**Total overhead per document**: ~82 bytes fixed + 64 bytes per lineage parent.

For a typical IronClaw deployment with 10,000 memory documents, the overhead is approximately:
- Fixed: 10,000 * 82 bytes = 820 KB
- Lineage (avg 2 parents): 10,000 * 2 * 64 bytes = 1.28 MB
- **Total**: ~2.1 MB overhead -- negligible compared to content storage.

### 18.6 Comparison with Flat Document Models

| Feature | Flat Document (current IronClaw) | Engram-enhanced |
|---|---|---|
| Deduplication | Existing version/content hash semantics | Optional canonical content hash index after migration analysis |
| Quality ranking | None (text similarity only) | 7-axis weighted scoring |
| Temporal relevance | Age only (updated_at) | Mathematical decay, configurable per-doc |
| Provenance | None | Author + trust + taint |
| Lineage | None | Full DAG with forensic audit |
| Safety filtering | None | Taint-based exclusion/penalization |
| Memory economics | Hard delete or keep-all | Demurrage + cold storage |

The flat model is simpler and has lower storage overhead. The Engram model can centralize scoring, lineage, and taint behavior that would otherwise drift across handlers, but only after the repository and DB contracts enforce those fields consistently.

---

## 19. Practical Examples

### 19.1 Memory Deduplication with BLAKE3

**Problem**: A user asks IronClaw to "remember that I prefer dark mode" multiple times across different sessions. Without deduplication, each write creates a new record.

**With Engram content-addressing**:

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

**Result**: 10 identical memory writes produce 1 stored document with `access_count=10` and `utility=1.0`. The document's effective weight increases with each confirmation, making it rank higher in search results.

### 19.2 Tracking Information Provenance Through Transformations

**Scenario**: Agent reads a web page, extracts a fact, synthesizes a recommendation.

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### 19.3 Decay-Weighted Memory Retrieval

**Scenario**: User asked about a project deadline 3 weeks ago. The deadline has passed. Should the memory still surface prominently?

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

**Practical retrieval**:

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### 19.4 Taint-Based Content Filtering

**Scenario**: User asks "What do you know about my API keys?" -- agent should not serve tainted (LLM-hallucinated) credential data.

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### 19.5 Attestation-Verified Knowledge Sharing

**Scenario**: Two IronClaw instances share a signed knowledge base. Instance B should verify that memories claimed to come from Instance A were actually produced by A, not forged.

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

---

## 20. IronClaw Integration Plan

IronClaw's current memory system (`src/workspace/document.rs`) stores documents as `MemoryDocument` structs with these fields: `id: Uuid`, `user_id: String`, `agent_id: Option<Uuid>`, `path: String`, `content: String`, `created_at: DateTime<Utc>`, `updated_at: DateTime<Utc>`, `metadata: serde_json::Value`. The system currently uses SHA-256 for `DocumentVersion.content_hash` (using the `sha2` crate) and hybrid search (FTS + vector embeddings via RRF) through four tools: `memory_search`, `memory_write`, `memory_read`, `memory_tree`.

Per IronClaw's architectural invariant: **"LLM data is never deleted."** All LLM output is retained. Decayed entries are archived, not removed.

### A. Enhanced MemoryDocument (Backward-Compatible)

**File**: `src/workspace/document.rs`

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### B. Database Migration (Both Backends)

**PostgreSQL** (`src/db/postgres/`):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

**libSQL/Turso** (`src/db/libsql/`):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### C. Content-Addressed Deduplication in memory_write

**File**: `src/tools/builtin/memory.rs` (or wherever `memory_write` is implemented)

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### D. Score-Weighted Memory Search Integration

**Integration with existing hybrid search** (`src/workspace/search.rs`):

```rust
/// Final ranking formula: RRF x effective_weight x taint_penalty
pub fn engram_weighted_rank(
    rrf_score: f64,
    doc: &MemoryDocument,
    taint_penalty: f64, // 0.0 = exclude, 1.0 = no penalty; typical: 0.3-0.5
) -> f64 {
    let effective = doc.effective_weight();
    let taint_factor = if doc.is_tainted { taint_penalty } else { 1.0 };
    rrf_score * effective * taint_factor
}
```

### E. Heartbeat Lifecycle Management

**Integration with `src/workspace/hygiene.rs`** (during heartbeat cycle, default 30 min):

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### F. Phased Rollout

1. **Phase 1** (1-2 weeks): Add `content_hash`, `score`, `source_type`, `is_tainted` columns + dedup in `memory_write`. Immediate deduplication and provenance value.
2. **Phase 2** (2-3 weeks): Add `decay` + `effective_weight()` computation + heartbeat lifecycle integration. Temporal dynamics.
3. **Phase 3** (2-3 weeks): Add `derived_from` + taint propagation. Full lineage tracking.
4. **Phase 4** (1-2 weeks): Integrate `effective_weight` into `memory_search` ranking. Enhanced retrieval.

---

## 21. Complexity Assessment

| Component | Estimated Lines | Risk | Notes |
|---|---|---|---|
| MemoryScore struct + effective() | 60 | Low | Pure arithmetic |
| DecayVariant enum + apply() | 80 | Low | Adapted from roko |
| MemoryDocument field additions | 60 | Low | All have defaults |
| compute_content_hash() + touch() + tick_demurrage() | 60 | Low | Straightforward |
| content-addressed dedup in memory_write | 80 | Low | Single extra DB lookup |
| infer_decay_from_path() heuristics | 30 | Low | Path-based switch |
| Taint propagation via source_type | 50 | Low | Simple flag inheritance |
| engram_weighted_rank() for search | 30 | Low | Multiplicative formula |
| engram_lifecycle_tick() heartbeat | 80 | Medium | Must handle archive edge cases |
| PostgreSQL migration | 40 | Low | Additive nullable columns |
| libSQL migration | 40 | Low | Same, libSQL syntax |
| DB repository updates (find_by_content_hash) | 60 | Low | New index + query |
| **Total** | **~670** | **Low** | |

**Risk assessment**: Medium. Start with a metadata overlay on `MemoryDocument` and caller-level ranking tests before adding columns. Any schema migration must go through the shared DB trait, implement both PostgreSQL and libSQL, preserve file-like memory semantics, and include rollback/backfill notes. The heartbeat lifecycle tick is the highest-risk behavior because it can hide useful documents if the archive threshold is wrong.

**Dependencies**: `blake3` is already available in the workspace. Do not replace existing SHA-256 `DocumentVersion.content_hash` semantics solely for speed; introduce any BLAKE3 content address as a separate field unless a migration plan proves compatibility.

---

## 22. References

### Neuroscience and Memory

- Ebbinghaus, H. (1885). *Uber das Gedachtnis: Untersuchungen zur experimentellen Psychologie*. Leipzig: Duncker & Humblot. English translation: *Memory: A Contribution to Experimental Psychology* (1913), Teachers College, Columbia University.
- Lashley, K.S. (1950). "In search of the engram." *Symposia of the Society for Experimental Biology* 4:454-482.
- Murre, J.M.J. and Dros, J. (2015). "Replication and Analysis of Ebbinghaus' Forgetting Curve." *PLOS ONE* 10(7):e0120644. doi:10.1371/journal.pone.0120644.
- Semon, R. (1904). *Die Mneme als erhaltendes Prinzip im Wechsel des organischen Geschehens*. Leipzig: Wilhelm Engelmann.
- Tonegawa, S., Liu, X., Ramirez, S., and Redondo, R. (2015). "Memory Engram Cells Have Come of Age." *Neuron* 87(5):918-931. doi:10.1016/j.neuron.2015.08.002. See also: Liu, X. et al. (2012). "Optogenetic stimulation of a hippocampal engram activates fear memory recall." *Nature* 484:381-385.

### Cryptography and Content-Addressing

- Benet, J. (2014). "IPFS - Content Addressed, Versioned, P2P File System." arXiv:1407.3561. [https://arxiv.org/abs/1407.3561](https://arxiv.org/abs/1407.3561).
- Merkle, R.C. (1979). "Method of providing digital signatures." U.S. Patent 4,309,569, filed September 5, 1979, issued January 5, 1982. See also: Merkle, R.C. (1987). "A Digital Signature Based on a Conventional Encryption Function." *Advances in Cryptology -- CRYPTO '87*, LNCS 293:369-378.
- O'Connor, J., Aumasson, J.-P., Neves, S., and Wilcox-O'Hearn, Z. (2020). "BLAKE3: one function, fast everywhere." Presented at Real World Crypto 2020. Specification: [https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf](https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf). See also: IETF draft: [https://www.ietf.org/archive/id/draft-aumasson-blake3-00.html](https://www.ietf.org/archive/id/draft-aumasson-blake3-00.html).
- Sanjuan, A.A., Spielman, E., and Pestana, P. (2020). "Merkle-CRDTs: Merkle-DAGs meet CRDTs." arXiv:2004.00107. [https://arxiv.org/abs/2004.00107](https://arxiv.org/abs/2004.00107).

### Information Theory and Appraisal

- Friston, K. (2010). "The free-energy principle: a unified brain theory?" *Nature Reviews Neuroscience* 11(2):127-138. doi:10.1038/nrn2787.
- Gruenwald, P.D. (2007). *The Minimum Description Length Principle*. MIT Press. ISBN 978-0262072816.
- Itti, L. and Baldi, P. (2005). "A principled approach to detecting surprising events in video." *IEEE Conference on Computer Vision and Pattern Recognition (CVPR)* 1:631-637.
- Kolmogorov, A.N. (1965). "Three approaches to the quantitative definition of information." *Problems of Information Transmission* 1(1):1-7.
- Miller, G.A. (1956). "The Magical Number Seven, Plus or Minus Two: Some Limits on Our Capacity for Processing Information." *Psychological Review* 63(2):81-97.
- Scherer, K.R. (2001). "Appraisal Considered as a Process of Multilevel Sequential Checking." In K.R. Scherer, A. Schorr, & T. Johnstone (Eds.), *Appraisal Processes in Emotion: Theory, Methods, Research* (pp. 92-120). Oxford University Press.
- Schmidhuber, J. (2010). "Formal Theory of Creativity, Fun, and Intrinsic Motivation (1990-2010)." *IEEE Transactions on Autonomous Mental Development* 2(3):230-247.

### Hyperdimensional Computing

- Kanerva, P. (2009). "Hyperdimensional Computing: An Introduction to Computing in Distributed Representation with High-Dimensional Random Vectors." *Cognitive Computation* 1(2):139-159.
- Kleyko, D., Rachkovskij, D.A., Osipov, E., and Rahimi, A. (2022). "A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures, Part I: Models and Data Transformations." *ACM Computing Surveys* 55(6):1-51. doi:10.1145/3538531.
- Plate, T.A. (2003). *Holographic Reduced Representations*. CSLI Publications. ISBN 978-1575864044.

### Economics

- Gesell, S. (1916). *Die Naturliche Wirtschaftsordnung durch Freiland und Freigeld*. English: *The Natural Economic Order* (1958 translation by Philip Pye). See also: [https://en.wikipedia.org/wiki/Demurrage_(currency)](https://en.wikipedia.org/wiki/Demurrage_(currency)).

### Software Engineering

- Kleppmann, M. (2017). *Designing Data-Intensive Applications*. O'Reilly Media. ISBN 978-1449373320.

### Taint Analysis

- Schwartz, E.J., Avgerinos, T., and Brumley, D. (2010). "All You Ever Wanted to Know About Dynamic Taint Analysis and Forward Symbolic Execution (but might have been afraid to ask)." *IEEE Symposium on Security and Privacy (S&P)* pp. 317-331.

---

## See Also

- **[Hyperdimensional Computing](./hyperdimensional-computing/README.md)** — full documentation of the HDC system whose `HdcFingerprint` is embedded in every fingerprinted Engram.
- **[context-memory/persistence-storage.md](../context-memory/persistence-storage.md)** — concrete database backends (PostgreSQL, libSQL) that implement the `Store` and `ColdStore` traits described in Section 14.
- **[Cognitive Architecture](./cognitive-architecture.md)** — how Engrams flow through Gamma (reactive search), Theta (reflective scoring), and Delta (consolidation / cold storage) tiers.
- **[Mathematical Primitives](./mathematical-primitives.md)** — the Ebbinghaus formula used in `Decay::Ebbinghaus` is implemented in the `roko-primitives` crate; TDA can detect loops in Engram lineage DAGs.
