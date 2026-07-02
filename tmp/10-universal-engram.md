# Universal Engram: Rich Memory Primitives for Cognitive AI Systems

**Source crate**: `roko-core` (`crates/roko-core/`)
**Priority**: HIGH -- richer memory entries with multi-axis scoring and decay
**Roko source files verified against**:
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
17. [IronClaw Integration Plan](#17-ironclaw-integration-plan)
18. [Complexity Assessment](#18-complexity-assessment)
19. [References](#19-references)

---

## 1. Introduction -- What Is an Engram?

The term "engram" originates from neuroscience. Richard Semon coined it in 1904 (Semon 1904) to describe the hypothetical physical trace that a memory leaves in the brain -- a biophysical change in neural tissue that encodes experience. For decades the engram was theoretical: Karl Lashley spent thirty years searching for it (Lashley 1950), ultimately concluding that memories were not localized to any single region. The concept was vindicated in 2015 when Susumu Tonegawa's group at MIT identified specific neurons ("engram cells") in the hippocampus and amygdala whose artificial reactivation could recall or suppress a fear memory (Tonegawa et al. 2015, *Science* 348(6238):1007-1013).

In the Roko AI agent system, an **Engram** is the digital analog: a content-addressed, scored, decaying unit of cognition. It is the single data type that represents everything the system knows, observes, decides, and produces. Where biological engrams encode memories as patterns of synaptic weights, Roko's engrams encode knowledge as structured data with explicit quality scores, decay functions, provenance records, and lineage graphs.

The key properties that make an Engram a rich memory primitive rather than a simple record:

- **Content-addressed identity**: An Engram is identified by a BLAKE3 hash of its content, not by an arbitrary ID. Identical content produces identical identifiers. This gives deduplication, integrity verification, and linkability for free.
- **Multi-dimensional scoring**: Seven independent axes (confidence, novelty, utility, reputation, precision, salience, coherence) that collapse into a single effective score via a multiplicative formula. Zero confidence or zero reputation structurally produces zero score -- invalid or untrusted information cannot be prioritized.
- **Temporal decay**: Every Engram has a decay function -- none, exponential half-life, hard TTL cutoff, or Ebbinghaus forgetting curve -- that causes its effective weight to diminish over time. The system's memory is not a static database; it is a living substrate where information competes for representation based on recency, utility, and reinforcement.
- **Provenance and taint**: Every Engram knows who produced it, how trusted that producer is, and whether the data is tainted (from an unverified source, generated by an LLM, flagged by a user, or inherited from a tainted parent). Taint propagates through the lineage DAG.
- **Lineage DAG**: Every Engram records which parent Engrams it was derived from, forming a directed acyclic graph that enables causal replay, impact analysis, and forensic audit of any decision.

For IronClaw, adopting the Engram pattern would transform its current flat memory system (`src/workspace/document.rs`, `MemoryDocument` with path/content/metadata) into one with mathematical decay, multi-axis quality scores, content-addressed deduplication, lineage tracking, and taint propagation.

---

## 2. Why a Universal Data Type for Knowledge

Classical software architectures use many types: tasks, events, messages, requests, responses, records, logs. Each type has its own schema, its own storage, its own lifecycle. Adding a new capability means adding a new type, a new store, a new API. This proliferation creates a combinatorial explosion of adapters, converters, and integration surfaces.

Roko takes a fundamentally different approach. There is exactly **one data type** -- the **Engram** -- and a small set of traits that operate on it. Every event, every piece of data, every agent output, every gate verdict, every knowledge entry, every prediction, every tool trace is an Engram. This design choice has three profound consequences:

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

```rust
/// The universal datum of the Roko system.
///
/// # Identity
///
/// An engram's identity is its [`ContentHash`], computed from its kind, body,
/// author, and tags (see [`Engram::content_hash`]). Score and decay are
/// **excluded** from the hash -- they can change without changing identity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Engram {
    /// Content-addressed identity (computed from kind + body + author + tags).
    pub id: ContentHash,

    /// HDC fingerprint plus encoder metadata used for similarity and clustering.
    /// Remains optional so callers can construct engrams before a substrate
    /// has finalized fingerprinting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<HdcFingerprint>,

    /// What kind of engram this is (Task, Episode, GateVerdict, Pheromone, ...).
    pub kind: Kind,

    /// The engram's payload (Empty, Text, Json, or Bytes).
    pub body: Body,

    /// Unix milliseconds when this engram was first emitted.
    pub created_at_ms: i64,

    /// How this engram's weight decays over time (None, HalfLife, Ttl, Ebbinghaus).
    pub decay: Decay,

    /// Producer attribution and trust (author + trust score + taint classification).
    pub provenance: Provenance,

    /// Quality score at emission time (may be recomputed by scorers).
    pub score: Score,

    /// ContentHashes of engrams this derived from (forms a DAG for auditing
    /// and autocatalytic metrics).
    pub lineage: Vec<ContentHash>,

    /// Arbitrary string metadata (ordered for stable hashing via BTreeMap).
    pub tags: BTreeMap<String, String>,

    /// Optional cryptographic proof of origin (Ed25519 signature + optional
    /// on-chain witness).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attestation: Option<Attestation>,

    /// Optional emotional metadata associated with this engram
    /// (PAD vector -- Pleasure, Arousal, Dominance).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emotional_tag: Option<EmotionalTag>,

    /// Demurrage balance in [0.0, 1.0]. Decays over time; refreshed on access.
    /// Defaults to 1.0 for new engrams.
    #[serde(default = "default_balance")]
    pub balance: f64,
}
```

### 3.1 Field Partitioning -- What Gets Hashed vs. What Doesn't

This is a critical architectural decision. The Engram's fields are partitioned into two sets:

**Identity fields (hashed -- changing these creates a new Engram)**:
- `kind` -- the semantic type
- `body` -- the payload
- `provenance.author` -- who produced this
- `provenance.taint` -- whether the data is tainted (via `is_tainted()` boolean)
- `lineage` -- parent ContentHashes
- `tags` -- all key-value pairs (BTreeMap guarantees deterministic ordering)

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

```rust
pub struct EngramBuilder {
    kind: Kind,
    body: Body,                            // default: Body::Empty (via Body::empty())
    created_at_ms: Option<i64>,            // default: current wall-clock time
    decay: Decay,                          // default: Decay::None
    provenance: Provenance,                // default: Provenance::default() -- trusted, author="roko"
    score: Score,                          // default: Score::NEUTRAL
    lineage: Vec<ContentHash>,             // default: empty
    tags: BTreeMap<String, String>,        // default: empty
    fingerprint: Option<HdcFingerprint>,   // default: None
    attestation: Option<Attestation>,      // default: None
    emotional_tag: Option<EmotionalTag>,   // default: None
    balance: f64,                          // default: 1.0
}
```

Usage examples:

```rust
use roko_core::{Body, Engram, Kind, Decay, Provenance, Score};

// Simple task Engram
let task = Engram::builder(Kind::Task)
    .body(Body::text("implement login"))
    .tag("priority", "high")
    .build();

// Pheromone with decay
let pheromone = Engram::builder(Kind::Pheromone)
    .body(Body::text("high gas prices detected"))
    .decay(Decay::HalfLife { half_life_ms: 14_400_000 }) // 4 hours
    .provenance(Provenance::agent("chain_monitor"))
    .tag("type", "opportunity")
    .build();

// Gate verdict derived from a task Engram (lineage tracking)
let verdict = task.derive(Kind::GateVerdict, Body::text("compilation passed"))
    .score(Score::new(1.0, 0.0, 1.0, 1.0))
    .build();
// verdict.lineage == [task.id]
```

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

```rust
/// A 32-byte content-addressed identifier (BLAKE3 digest).
///
/// Two signals with identical canonical encoding share the same `ContentHash`.
/// The hash is computed over the signal's body and its identity fields, but
/// **not** its score or decay -- those can change without changing identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(#[serde(with = "hex_bytes")] pub [u8; 32]);

impl ContentHash {
    /// Compute a content hash from arbitrary bytes.
    pub fn of(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }

    /// Hex-encoded representation (64 chars).
    pub fn to_hex(&self) -> String { ... }

    /// Short form for logs/display (first 8 hex chars).
    pub fn short(&self) -> String {
        format!("{:02x}{:02x}{:02x}{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3])
    }

    /// Parse a hex string into a ContentHash. Returns None for malformed input.
    pub fn from_hex(s: &str) -> Option<Self> { ... }
}
```

### 4.2 Why BLAKE3 Over SHA-256

BLAKE3 was designed by Jack O'Connor, Jean-Philippe Aumasson, Samuel Neves, and Zooko Wilcox-O'Hearn and announced at Real World Crypto 2020 (O'Connor et al. 2020, "BLAKE3: one function, fast everywhere," [BLAKE3 specification](https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf)). The choice of BLAKE3 over SHA-256 is motivated by three properties:

1. **Speed**: BLAKE3 uses a Merkle tree internal structure and SIMD optimizations to achieve approximately 5-14x faster throughput than SHA-256 on modern hardware. On a single core with AVX-512, BLAKE3 achieves over 1 GB/s; with multi-threading it scales near-linearly with core count.
2. **Streaming**: BLAKE3 supports incremental hashing natively via its `Hasher` type, which matters when Engrams contain large payloads (file contents, compiled artifacts). You can feed data in chunks without buffering the entire input.
3. **Keyed mode**: BLAKE3 supports keyed hashing for MAC computation (`blake3::keyed_hash(key, data)`), useful for attestation without requiring a separate HMAC construction.

These advantages over SHA-256 come without any reduction in security: BLAKE3 provides 128-bit collision resistance (256-bit output), which is the same effective collision security as SHA-256.

### 4.3 The Exact Hash Computation

From `crates/roko-core/src/engram.rs` (lines 113-134), the `content_hash()` method feeds identity fields into a BLAKE3 hasher with pipe delimiters between fields:

```rust
impl Engram {
    pub fn content_hash(&self) -> ContentHash {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.kind.identity_key().as_bytes());
        hasher.update(b"|");
        hasher.update(&self.body.canonical_bytes());
        hasher.update(b"|");
        hasher.update(self.provenance.author.as_bytes());
        hasher.update(b"|");
        hasher.update(&[u8::from(self.provenance.is_tainted())]);
        hasher.update(b"|");
        for h in &self.lineage {
            hasher.update(&h.0);
        }
        hasher.update(b"|");
        for (k, v) in &self.tags {
            hasher.update(k.as_bytes());
            hasher.update(b"=");
            hasher.update(v.as_bytes());
            hasher.update(b";");
        }
        ContentHash(*hasher.finalize().as_bytes())
    }
}
```

**Key details**:
- Fields are separated by `|` pipe delimiters to prevent ambiguity
- Tags use `key=value;` format with semicolons
- `BTreeMap` guarantees lexicographic key order, making the hash deterministic regardless of insertion order
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

```rust
/// A multi-dimensional quality score for a signal.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Score {
    /// [0..1] -- how confident are we this signal is correct/valid?
    pub confidence: f32,
    /// [0..1] -- how novel is this signal compared to prior signals?
    pub novelty: f32,
    /// [0..inf) -- how useful has this signal proven historically?
    pub utility: f32,
    /// [0..inf) -- reputation of the signal's author at emission time.
    pub reputation: f32,
    /// [0..1] -- how exact or narrowly applicable is this signal?
    #[serde(default)]
    pub precision: f32,
    /// [0..1] -- how much extra ranking weight should this signal receive?
    #[serde(default)]
    pub salience: f32,
    /// [0..1] -- how internally consistent is the evidence?
    #[serde(default)]
    pub coherence: f32,
}
```

### 5.1 The Four Primary Axes

#### Confidence -- [0, 1]

**What it measures**: How sure are we that this Engram is correct, valid, or truthful?

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

#### Reputation -- [0, infinity)

**What it measures**: How trustworthy is the Engram's producer at the time the Engram was created?

**Critical property**: Zero reputation produces zero effective score (like zero confidence). Untrusted sources are structurally excluded.

**Default values by provenance** (from `crates/roko-core/src/provenance.rs` lines 297-347):
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

```rust
impl Score {
    pub fn effective(&self) -> f32 {
        if !self.is_finite() {
            return 0.0;
        }
        let salience_factor = if self.salience == 0.0 {
            1.0
        } else {
            0.5 + 0.5 * self.salience
        };
        let coherence_factor = if self.coherence == 0.0 {
            1.0
        } else {
            0.5 + 0.5 * self.coherence
        };
        finite_non_negative(
            self.confidence
                * (1.0 + self.novelty)
                * (1.0 + self.utility)
                * self.reputation
                * salience_factor
                * coherence_factor,
        )
    }
}
```

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

**Design rationale**: The extended factors are opt-in soft damping. When salience and coherence are zero (the default for `Score::new()`), the 6-factor formula reduces exactly to the original 4-factor formula: `confidence x (1 + novelty) x (1 + utility) x reputation`. Precision remains excluded because it describes applicability narrowness, not quality. The choice of seven dimensions connects to Miller's (1956) "magical number seven" channel capacity (*Psychological Review* 63(2):81-97) and Scherer's (2001) component process model of appraisal which uses 5-7 evaluation dimensions (*Applied AI* 15:5-131).

**Formula properties**:

| Property | Guarantee | Why It Matters |
|---|---|---|
| `confidence = 0` --> `effective = 0` | Zero confidence kills the score | Invalid information is never prioritized |
| `reputation = 0` --> `effective = 0` | Zero reputation kills the score | Untrusted sources are structurally excluded |
| `novelty = 0` --> multiplier = 1.0 | No penalty for routine information | Routine is normal, not bad |
| `novelty = 1` --> multiplier = 2.0 | Novel information gets 2x priority | Surprise drives attention |
| `utility = 0` --> multiplier = 1.0 | New Engrams start at baseline | No penalty for lack of history |
| `utility = n` --> multiplier = `(1+n)` | Utility accumulates multiplicatively | Frequently-useful Engrams dominate |
| Non-finite input --> 0.0 | NaN/Inf safety | Numeric robustness guaranteed |

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

```rust
/// How a signal's weight diminishes over time.
///
/// `Decay::apply(age_ms)` returns a multiplier in `[0.0, 1.0]` that scales
/// the signal's score. A fresh signal has multiplier `1.0`; a fully-decayed
/// signal has multiplier `0.0`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Decay {
    /// No decay -- signal weight is permanent.
    None,

    /// Exponential half-life: weight = 0.5 ^ (age / half_life_ms).
    HalfLife { half_life_ms: u64 },

    /// Hard cutoff: full weight until ttl_ms, then zero.
    Ttl { ttl_ms: u64 },

    /// Ebbinghaus forgetting curve: weight = exp(-age / (strength * scale_ms)).
    Ebbinghaus { strength: f32, scale_ms: u64 },
}
```

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

**Design note** (from the source comments): The TTL is a **relative duration** (milliseconds since emission), not an absolute timestamp. `Decay::apply()` takes a relative `age_ms` parameter. Absolute deadlines are handled at a higher layer: the emitter computes `ttl_ms = deadline - now` at construction time.

**Use cases**: Strict validity windows (offers, bounties), session-scoped data, tool output whose validity is strictly time-bound.

### 6.4 Decay::Ebbinghaus -- Forgetting Curve

**Mathematical formulation**:

```
weight(age) = exp(-age_ms / (strength * scale_ms))
            = e^(-age_ms / (strength * scale_ms))
```

Where:
- `strength` (float, [0..infinity)) is a retention multiplier. Higher = signal persists longer.
- `scale_ms` (integer) is the base time unit in milliseconds.
- The product `strength * scale_ms` gives the effective time constant `tau`.

**At the time constant** (age = strength * scale_ms = tau): `weight = 1/e ~ 0.368`.

**Implementation** (from `crates/roko-core/src/decay.rs` lines 107-114):

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

**Ebbinghaus with tier shaping**: In roko's Neuro subsystem, the effective half-life combines a knowledge-type base with a validation tier multiplier:

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

Used by garbage collection and pruning routines. Also note that negative ages (clock skew) return `1.0` -- the implementation handles this at line 87-89.

---

## 7. Demurrage -- Attention Economics

Beyond the four decay variants, Roko defines a **demurrage** system that models knowledge retention as an attention economy. The concept is borrowed from Silvio Gesell's 1916 *The Natural Economic Order* (Gesell 1916), which proposed that money should carry a holding cost ("demurrage") to encourage circulation. Just as Gesellian demurrage taxes idle money to prevent hoarding, Roko's demurrage taxes idle knowledge to ensure active validation.

From `crates/roko-core/src/demurrage.rs` (lines 9-32):

```rust
/// Time-decay tax on stored value -- ensures active validation.
///
/// Implementors must be refreshed (validated, used) to maintain
/// their balance. Neglected items naturally fade.
///
/// The decay formula is: balance *= (1 - rate)^elapsed_hours
pub trait Demurrage {
    /// Current attention/value balance in [0.0, 1.0].
    fn balance(&self) -> f64;

    /// Hourly decay rate in [0.0, 1.0].
    fn demurrage_rate(&self) -> f64;

    /// Apply time-based decay for elapsed_hours of inactivity.
    fn tick(&mut self, elapsed_hours: f64);

    /// Replenish the balance (capped at 1.0) after active validation.
    fn replenish(&mut self, amount: f64);

    /// Returns true when the balance has fallen below the usable threshold.
    fn is_depleted(&self) -> bool {
        self.balance() < 0.1
    }
}
```

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

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    // Agent runtime
    ProcessSpawn, ProcessExit, AgentMessage, AgentOutput,
    TokenUsage, ApprovalRequested,

    // Verification
    GateVerdict, TestResult, CompileDiagnostic,

    // Tasks & plans
    Task, Plan, PlanPhase,

    // Context assembly
    PromptSection, ContextPack, Prompt,

    // Routing & learning
    RouterChoice, RouterFeedback,

    // Memory
    Episode, PlaybookRule, Skill,

    // Compound (multiple kinds simultaneously)
    Compound(Vec<Kind>),

    // Observability
    Metric, ExperimentResult, ToolInvocation, ToolHealthDegraded,

    // Chain participation
    Insight, Pheromone, Bounty, Transaction, Service, Prediction,

    // Extension (user-defined)
    Custom(String),
}
```

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

```rust
impl Engram {
    /// Emit a derived engram -- new kind/body, but tracks this engram as lineage.
    pub fn derive(&self, kind: Kind, body: Body) -> EngramBuilder {
        EngramBuilder::new(kind)
            .body(body)
            .lineage([self.id])
            .provenance(Provenance::agent("derived"))
    }

    /// Emit a derived gate verdict engram with explicit verdict defaults.
    /// Preserves the parent's tag set, carries forward the full known lineage
    /// chain, and applies the Decay::GATE_VERDICT contract.
    pub fn derive_verdict(&self, body: Body) -> EngramBuilder {
        let mut builder = EngramBuilder::new(Kind::GateVerdict)
            .body(body)
            .decay(Decay::GATE_VERDICT)
            .lineage(self.derived_lineage())
            .provenance(Provenance::agent("derived"));
        for (key, value) in &self.tags {
            builder = builder.tag(key.clone(), value.clone());
        }
        builder
    }
}
```

The `derive_verdict()` method is particularly notable: it carries forward the **entire** known lineage chain (via `derived_lineage()`, which appends `self.id` to all existing lineage entries, deduplicating), preserves all parent tags (with child tags overriding on collision), and applies the standard `Decay::GATE_VERDICT` (24-hour half-life).

### 10.2 DAG Traversal

The lineage DAG can be traversed by querying the Store for each parent ContentHash (BFS traversal with cycle detection via `HashSet`):

```rust
async fn trace_lineage(
    store: &dyn Store,
    engram: &Engram,
) -> Vec<Engram> {
    let mut ancestors = Vec::new();
    let mut queue: VecDeque<ContentHash> = engram.lineage.iter().copied().collect();
    let mut seen = HashSet::new();
    while let Some(id) = queue.pop_front() {
        if !seen.insert(id) { continue; }
        if let Ok(Some(parent)) = store.get(&id).await {
            queue.extend(parent.lineage.iter().copied());
            ancestors.push(parent);
        }
    }
    ancestors
}
```

---

## 11. Provenance and Taint Propagation

Every Engram carries a `Provenance` record that answers three questions: who produced this, how trusted are they, and is the data tainted?

### 11.1 The Provenance Struct

From `crates/roko-core/src/provenance.rs` (lines 263-292):

```rust
/// Who produced a signal and how trustworthy they are.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// Identifier of the producer (agent role, user email, chain address).
    pub author: String,
    /// Trust score [0..1] at time of emission.
    pub trust: f32,
    /// Typed taint classification. Taint::Clean means untainted.
    #[serde(default)]
    pub taint: Taint,
    /// Deprecated legacy structured taint metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taint_info: Option<TaintInfo>,
    /// Optional session or run ID that produced this signal.
    pub session: Option<String>,
}
```

### 11.2 The Taint Enum

The `Taint` enum (lines 21-67) has 9 variants. Each variant is `#[non_exhaustive]` so new taint reasons can be added without breaking downstream matches:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Taint {
    Clean,                                           // no taint
    LlmHallucination { detail: String },             // LLM-generated, may hallucinate
    ToolFailure { detail: String },                  // tool call failed
    UserFlagged { detail: String },                  // human explicitly flagged
    StaleData { threshold_ms: i64 },                 // exceeded freshness window
    UnverifiedSource { detail: String },             // unverified external source
    Propagated { detail: String,                     // inherited from upstream
                 inherited_from: Option<ContentHash> },
    UserInput { detail: String },                    // user-provided input
    Custom(String),                                  // application-specific
}
```

The taint system implements a form of **dynamic taint analysis** (Schwartz et al. 2010, "All You Ever Wanted to Know About Dynamic Taint Analysis and Forward Symbolic Execution," *IEEE S&P*). Data from untrusted sources is tagged at the boundary and the tag propagates through derivation chains via the `Propagated` variant.

### 11.3 Taint Propagation Rules

**Taint affects the content hash**: The taint boolean (`is_tainted()`) is included in the content hash computation at line 121 of `engram.rs`. This means that the same content from a tainted source and a clean source produces **different** ContentHashes -- you cannot substitute a tainted Engram for a clean one.

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

```rust
impl Provenance {
    pub fn trusted(author: impl Into<String>) -> Self {
        Self { author: author.into(), trust: 1.0, taint: Taint::Clean, taint_info: None, session: None }
    }
    pub fn agent(author: impl Into<String>) -> Self {
        Self { author: author.into(), trust: 0.75, taint: Taint::Clean, taint_info: None, session: None }
    }
    pub fn external(author: impl Into<String>) -> Self {
        let author = author.into();
        Self { taint: Taint::UnverifiedSource { detail: format!("source {}", author) },
               taint_info: None, author, trust: 0.1, session: None }
    }
    pub fn user(author: impl Into<String>) -> Self {
        let author = author.into();
        Self { taint: Taint::UserInput { detail: format!("source {}", author) },
               taint_info: None, author, trust: 0.5, session: None }
    }
}
```

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

```rust
pub fn sign(engram: &Engram, key: &SigningKey) -> Attestation {
    let hash = engram.content_hash();
    let signature = key.sign(&hash.0);
    Attestation {
        signature: Ed25519Signature(signature.to_bytes()),
        public_key: PublicKey(key.verifying_key().to_bytes()),
        chain_attestation: None,
    }
}

pub fn verify(engram: &Engram, attestation: &Attestation) -> bool {
    let Ok(public_key) = VerifyingKey::from_bytes(&attestation.public_key.0) else {
        return false;
    };
    let signature = Signature::from_bytes(&attestation.signature.0);
    public_key.verify(&engram.content_hash().0, &signature).is_ok()
}
```

**Key design decisions**:
- Attestations are **excluded** from the content hash, so an Engram can be attested after creation without changing its ID
- The `witness_hash()` method excludes `chain_attestation` so the anchored proof stays stable before and after on-chain publication
- Chain attestation (`ChainAttestation { chain_id, tx_hash, block_number }`) provides timestamped proof-of-existence on a blockchain

---

## 13. HDC Fingerprinting and Semantic Search

Every Engram can carry an HDC (Hyperdimensional Computing) fingerprint for fast similarity search. HDC, also called Vector Symbolic Architecture (VSA), represents information as high-dimensional binary vectors and uses simple algebraic operations (XOR, majority vote, cyclic shift) for associative memory and similarity search (Kanerva 2009, "Hyperdimensional Computing: An Introduction to Computing in Distributed Representation with High-Dimensional Random Vectors," *Cognitive Computation* 1(2):139-159; Kleyko et al. 2022, "A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures, Part I," *ACM Computing Surveys* 55(6):1-51; Plate 2003, *Holographic Reduced Representations*, CSLI Publications).

From `crates/roko-core/src/engram.rs` (lines 17-27):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HdcFingerprint {
    /// The semantic fingerprint vector for this engram.
    pub vector: HdcVector,      // 10,240-bit binary vector (1,280 bytes)
    /// Monotonic version of the encoder used to derive vector.
    pub encoder_version: u32,
}
```

Three algebraic operations on fingerprints (lines 270-291):

| Operation | HDC Implementation | Engram Meaning |
|---|---|---|
| **Bind** | XOR of HDC vectors | Associate two Engrams (key-value pair) |
| **Bundle** | Majority vote | Create cluster centroid / composite |
| **Permute** | Cyclic bit shift | Encode temporal ordering |

```rust
impl Engram {
    pub fn bind(&self, other: &Engram) -> Option<HdcVector> {
        Some(self.fingerprint?.vector.bind(&other.fingerprint?.vector))
    }
    pub fn bundle(engrams: &[Engram]) -> Option<HdcVector> {
        let mut vectors = Vec::with_capacity(engrams.len());
        for engram in engrams { vectors.push(engram.fingerprint?.vector); }
        let refs = vectors.iter().collect::<Vec<_>>();
        Some(HdcVector::bundle(&refs))
    }
    pub fn at_position(&self, position: usize) -> Option<HdcVector> {
        Some(self.fingerprint?.vector.permute(position))
    }
}
```

---

## 14. The Core Traits

The entire Roko system is built from the Engram and traits defined in `crates/roko-core/src/traits.rs`. These traits define the complete operational surface:

### 14.1 Store (lines 37-80)
Stores and queries Engrams. All storage backends implement this trait. Key methods: `put()` (idempotent), `get()`, `query()`, `query_similar()` (HDC), `prune()`.

### 14.2 ColdStore (lines 101-151)
Archival store for aged-out Engrams. Methods: `archive()`, `thaw()`, `purge_before()`. Migration flow: `Store (hot) --age_out()--> ColdStore (cold/archive)`.

### 14.3 Score trait (lines 167-198)
Rates Engrams along the 7 axes. Pure functions of `(engram, context)`. Polymorphic over Engrams and Pulses via `Datum` dispatch.

### 14.4 Verify (Gate) (lines 214-226)
Verifies Engrams against ground truth (compile, run tests, simulate transactions). Returns a `Verdict`.

### 14.5 Route (lines 242-267)
Selects one Engram from many candidates. Learns via `feedback()`. Implementations include `StaticRouter`, `LinUCBRouter` (contextual bandit), `CascadeRouter`, `WeightedRouter`.

### 14.6 Compose (lines 285-320)
Combines multiple Engrams into one under a `Budget`. Primary use case: assembling prompts from PromptSection Engrams under a token budget.

### 14.7 React (Policy) (lines 339-365)
Watches streams of Engrams/Pulses and emits interventions. The reactive/behavioral layer: conductor watchers, circuit breakers, episode logging, pheromone reactions.

### 14.8 Bus (lines 385-395)
Publish/subscribe transport for ephemeral Pulses. `publish()` returns a monotonic sequence number; `subscribe()` takes a `TopicFilter`.

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
let patch = user_msg.derive(Kind::AgentOutput, Body::text("--- a/auth.rs\n+++ b/auth.rs\n..."))
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
// verdict inherits tags from patch (e.g., any plan_id)
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
t=168h    ...        Balance < 0.1 threshold? No. But if it were:
                       -> Move to ColdStore (archive, not delete)
                       -> Can be thawed if lineage traversal needs it
```

**Step 6 -- Taint propagation**

If the user task referenced external data:

```rust
let external_data = Engram::builder(Kind::Insight)
    .body(Body::text("Auth library v2.3 has CVE-2024-1234"))
    .provenance(Provenance::external("security_feed"))  // trust=0.1, tainted
    .build();

// Anything derived from external_data inherits taint:
let analysis = external_data.derive(Kind::Episode, Body::text("User may be vulnerable"))
    .provenance(Provenance::agent("analyzer").with_taint(
        Taint::Propagated {
            detail: "inherited from external source".into(),
            inherited_from: Some(external_data.id),
        }
    ))
    .build();
// analysis.provenance.is_tainted() == true
// analysis.id is DIFFERENT from what it would be if taint were clean
// (because is_tainted() is included in the content hash)
```

---

## 17. IronClaw Integration Plan

IronClaw's current memory system stores documents as `MemoryDocument` structs (defined in `src/workspace/document.rs`) with these fields: `id: Uuid`, `user_id: String`, `agent_id: Option<Uuid>`, `path: String`, `content: String`, `created_at: DateTime<Utc>`, `updated_at: DateTime<Utc>`, `metadata: serde_json::Value`. Memory chunks (`MemoryChunk`) are used for search indexing. The system uses hybrid search (FTS + vector embeddings via RRF) through four tools: `memory_search`, `memory_write`, `memory_read`, `memory_tree`.

Per IronClaw's architectural invariant: **"LLM data is never deleted."** All LLM output is retained. Decayed entries are archived, not removed.

### A. Enhanced Memory Entries

**Where**: `src/workspace/document.rs` -- enrich `MemoryDocument`

```rust
// Current IronClaw MemoryDocument (from src/workspace/document.rs)
pub struct MemoryDocument {
    pub id: Uuid,
    pub user_id: String,
    pub agent_id: Option<Uuid>,
    pub path: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

// Enhanced with Engram concepts (backward-compatible via defaults)
pub struct MemoryDocument {
    // Existing fields (unchanged)
    pub id: Uuid,
    pub user_id: String,
    pub agent_id: Option<Uuid>,
    pub path: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,

    // NEW: Content-addressed identity (BLAKE3 of content)
    pub content_hash: Option<[u8; 32]>,

    // NEW: Multi-axis scoring (from roko Score, simplified to 4 primary axes)
    pub confidence: f32,            // [0, 1] -- default 0.5 (neutral)
    pub utility: f32,               // [0, inf) -- default 0.0 (new)
    pub novelty: f32,               // [0, 1] -- default 0.0
    pub reputation: f32,            // [0, inf) -- default 1.0 (trusted)

    // NEW: Decay (from roko Decay enum)
    pub decay_variant: DecayVariant, // None | HalfLife | Ttl | Ebbinghaus

    // NEW: Usage tracking (for demurrage / Ebbinghaus strength updates)
    pub access_count: u32,           // default 0
    pub last_accessed: DateTime<Utc>,
    pub balance: f64,                // [0, 1] -- demurrage balance, default 1.0

    // NEW: Lineage (from roko lineage DAG)
    pub derived_from: Vec<String>,   // Parent content hashes (hex strings)

    // NEW: Taint (from roko Provenance/Taint)
    pub source_type: SourceType,     // User | LLM | External | Derived | Tool
    pub is_tainted: bool,            // default false
}

pub enum DecayVariant {
    None,
    HalfLife { half_life: Duration },
    Ttl { expires_at: DateTime<Utc> },
    Ebbinghaus { strength: f64, scale: Duration },
}

pub enum SourceType {
    User,       // trust 0.5
    Agent,      // trust 0.75
    LLM,        // trust 0.5, tainted (potential hallucination)
    External,   // trust 0.1, tainted
    Derived,    // inherits from parents
    Tool,       // trust 0.75
    Verified,   // trust 1.0 (gate-checked)
}
```

### B. Effective Weight Computation

```rust
impl MemoryDocument {
    pub fn effective_weight(&self, now: DateTime<Utc>) -> f64 {
        let age_ms = (now - self.created_at).num_milliseconds() as f64;
        let decay_factor = self.decay_variant.current_strength(age_ms);
        let score = self.confidence as f64
            * (1.0 + self.novelty as f64)
            * (1.0 + self.utility as f64)
            * self.reputation as f64;
        score * decay_factor
    }
}

impl DecayVariant {
    pub fn current_strength(&self, age_ms: f64) -> f64 {
        if age_ms <= 0.0 { return 1.0; }
        match self {
            Self::None => 1.0,
            Self::HalfLife { half_life } => {
                let hl = half_life.num_milliseconds() as f64;
                if hl <= 0.0 { return 0.0; }
                0.5_f64.powf(age_ms / hl)
            },
            Self::Ttl { expires_at } => {
                let now = Utc::now();
                if now < *expires_at { 1.0 } else { 0.0 }
            },
            Self::Ebbinghaus { strength, scale } => {
                let s = scale.num_milliseconds() as f64;
                if s <= 0.0 || *strength <= 0.0 { return 0.0; }
                (-age_ms / (*strength * s)).exp()
            },
        }
    }
}
```

### C. Intelligent Memory Lifecycle Management

**Where**: `src/workspace/mod.rs` -- integrate with heartbeat system

```
Periodically (during heartbeat cycle, default every 30 minutes):
  For each memory document:
    weight = document.effective_weight(now)

    if weight < EVICTION_THRESHOLD (e.g., 0.01):
      if document.utility > HIGH_UTILITY_THRESHOLD (e.g., 3.0):
        // High-utility but decayed -- promote to longer decay
        document.decay_variant = DecayVariant::HalfLife { half_life: 30.days() }
      else:
        // Archive (mark as archived, NOT delete -- per "LLM data never deleted")
        document.metadata["archived_at"] = now.to_rfc3339()
        document.metadata["archived_weight"] = weight

    // Demurrage tick: reduce balance based on time since last access
    elapsed_hours = (now - document.last_accessed).num_hours() as f64
    document.balance *= (1.0 - DEMURRAGE_RATE).powf(elapsed_hours)
    // DEMURRAGE_RATE = 0.01 per hour (configurable)

    // Depleted entries are candidates for cold storage
    if document.balance < 0.1:
      move_to_cold_storage(document)  // Archive row, not delete
```

### D. Content-Addressed Deduplication

**Where**: `memory_write` tool implementation

```rust
fn memory_write(path: &str, content: &str, user_id: &str) -> Result<()> {
    let content_hash = blake3::hash(content.as_bytes());

    // Check for existing document with same content hash
    if let Some(existing) = store.find_by_content_hash(&content_hash)? {
        // Same content already exists -- merge metadata
        existing.access_count += 1;
        existing.last_accessed = Utc::now();
        existing.balance = 1.0;  // Refresh demurrage balance (touch)
        // If the new write targets a different path, record as alias
        if existing.path != path {
            let aliases: Vec<String> = existing.metadata.get("aliases")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            if !aliases.contains(&path.to_string()) {
                // ... add alias
            }
        }
        return store.update(existing);
    }

    // New content -- create document with Engram-enriched fields
    store.insert(MemoryDocument {
        content_hash: Some(*content_hash.as_bytes()),
        confidence: 0.5,   // neutral until scored
        utility: 0.0,      // no history yet
        novelty: 0.0,      // not yet compared
        reputation: 1.0,   // default trusted
        balance: 1.0,      // fresh
        ..MemoryDocument::new(user_id, None, path)
    })
}
```

### E. Lineage Tracking

**Where**: Workspace memory, integrated with agent tool calls

```
Memory A: "User prefers Rust" (source: user message, source_type: User)
    content_hash: a1b2c3d4...

Memory B: "User's project uses async/await" (source: code analysis, source_type: Tool)
    content_hash: e5f6a7b8...

--> Derived Memory C: "User likely familiar with tokio"
    derived_from: ["a1b2c3d4...", "e5f6a7b8..."]
    source_type: Derived
    is_tainted: false  (both parents are clean)

Memory D: "External API says tokio 1.39 has a bug" (source: web_fetch)
    source_type: External
    is_tainted: true

--> Derived Memory E: "User may be affected by tokio bug"
    derived_from: [hash(C), hash(D)]
    source_type: Derived
    is_tainted: true  (inherits from tainted parent D)
```

### F. Source Tainting

| Source | Source Type | Default Trust | Tainted? |
|---|---|---|---|
| User says something | User | 0.5 | true (UserInput) |
| LLM generates content | LLM | 0.5 | true (potential hallucination) |
| Web fetch / HTTP tool | External | 0.1 | true (UnverifiedSource) |
| Tool call succeeds | Tool | 0.75 | false |
| Tool call fails | Tool | 0.1 | true (ToolFailure) |
| Derived from tainted parent | Derived | min(parent_trust) | true (Propagated) |
| Gate-verified (test passes) | Verified | 1.0 | false (Clean) |

### G. Database Migration

New columns to add to memory tables (both PostgreSQL and libSQL, per IronClaw's dual-backend requirement):

```sql
ALTER TABLE memory_documents ADD COLUMN content_hash BLOB;        -- 32 bytes, BLAKE3
ALTER TABLE memory_documents ADD COLUMN confidence REAL DEFAULT 0.5;
ALTER TABLE memory_documents ADD COLUMN utility REAL DEFAULT 0.0;
ALTER TABLE memory_documents ADD COLUMN novelty REAL DEFAULT 0.0;
ALTER TABLE memory_documents ADD COLUMN reputation REAL DEFAULT 1.0;
ALTER TABLE memory_documents ADD COLUMN decay_type TEXT DEFAULT 'none';
ALTER TABLE memory_documents ADD COLUMN decay_params TEXT;         -- JSON for variant params
ALTER TABLE memory_documents ADD COLUMN access_count INTEGER DEFAULT 0;
ALTER TABLE memory_documents ADD COLUMN last_accessed TEXT;
ALTER TABLE memory_documents ADD COLUMN balance REAL DEFAULT 1.0;
ALTER TABLE memory_documents ADD COLUMN derived_from TEXT;         -- JSON array of parent hashes
ALTER TABLE memory_documents ADD COLUMN source_type TEXT DEFAULT 'agent';
ALTER TABLE memory_documents ADD COLUMN is_tainted BOOLEAN DEFAULT FALSE;

CREATE INDEX idx_memory_content_hash ON memory_documents(content_hash);
CREATE INDEX idx_memory_balance ON memory_documents(balance);
CREATE INDEX idx_memory_source_type ON memory_documents(source_type);
CREATE INDEX idx_memory_is_tainted ON memory_documents(is_tainted);
```

### H. Integration with Existing Memory Search

The `memory_search` tool currently uses hybrid search (FTS + vector embeddings via RRF). The Engram-enriched fields enable **score-weighted ranking**:

```
final_rank = rrf_score * effective_weight * (1.0 if !is_tainted else TAINT_PENALTY)

where:
  rrf_score        = existing reciprocal rank fusion score
  effective_weight = confidence * (1 + novelty) * (1 + utility) * reputation * decay_factor
  TAINT_PENALTY    = 0.5 (configurable -- tainted results ranked lower, not excluded)
```

This means frequently-accessed, high-utility, recently-created memories rank higher than stale, unverified, low-utility ones -- even if the raw text similarity is the same.

---

## 18. Complexity Assessment

| Component | Estimated Lines | Notes |
|---|---|---|
| Score struct with 4 primary axes | 150 | Adapted from roko, with `effective()` formula |
| DecayVariant enum with 4 variants | 100 | Adapted from roko, using `chrono::Duration` |
| Content hash + dedup in memory_write | 100 | BLAKE3 hashing + dedup check |
| MemoryDocument field additions | 50 | New fields with defaults |
| Demurrage tick logic | 80 | Balance decay + touch on access |
| Lineage tracking | 100 | DAG construction via derived_from field |
| Taint propagation | 100 | SourceType enum + propagation rules |
| Effective weight computation | 50 | Score x decay scalar |
| Search ranking integration | 80 | Weight-adjusted RRF scoring |
| Database migration (both backends) | 50 | SQL schema changes |
| Heartbeat lifecycle integration | 100 | Periodic eviction/archival check |
| **Total** | **~1,000** | |

**Risk**: Low -- enriches existing data model without breaking changes. All new fields have sensible defaults (confidence=0.5, utility=0.0, decay=None, balance=1.0) so existing code continues to work unmodified. The migration is additive (new nullable columns with defaults).

**Dependencies**: `blake3` crate (check if already a transitive dependency; if not, it is lightweight -- pure Rust, no C dependencies required).

**Phased rollout**:
1. **Phase 1**: Add content_hash, confidence, utility fields + dedup in memory_write. Immediate value.
2. **Phase 2**: Add DecayVariant + effective_weight computation + heartbeat integration.
3. **Phase 3**: Add lineage tracking (derived_from) + taint propagation.
4. **Phase 4**: Integrate effective_weight into memory_search ranking.

---

## 19. References

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
