[← Back to HDC Overview](./README.md)

# Applications — 6 Use Cases with Worked Examples

This document covers the six application areas implemented in the roko codebase, each with complete source code and concrete similarity values.

---

## Application 1: Knowledge Fingerprinting

The `KnowledgeHdcEncoder` encodes knowledge entries (Engrams) as HDC vectors. Every knowledge entry gets a 10,240-bit fingerprint at ingestion time, enabling content-addressed similarity search without any external embedding model.

*Source: `crates/roko-neuro/src/hdc.rs`*

### Memory Fingerprinting Flow

```mermaid
flowchart TD
    subgraph WRITE["Write Path: memory_write(content, tags, path)"]
        direction TB
        W1["content: &str\ntags: &[String]\npath: &str"]
        W2["text_hv(content)\n↓ normalize → FNV-1a → splitmix64\nHdcVector (1,280 bytes)"]
        W3["tags.iter().map(|t| from_seed(t))\n↓ bundle(tag_hvs)\ntag composite HdcVector"]
        W4["role_hv('path').bind(text_hv(path))\npath role-filler binding"]
        W5["bundle([content_hv, tags_hv, path_hv])\nfinal fingerprint HdcVector"]
        W6["fingerprint.to_bytes()\n→ store as BLOB in documents.hdc_fingerprint"]

        W1 --> W2
        W1 --> W3
        W1 --> W4
        W2 --> W5
        W3 --> W5
        W4 --> W5
        W5 --> W6
    end

    subgraph DEDUP["Deduplication Check (before commit)"]
        D1["load existing hdc_fingerprints\nfrom DB (lazy, cached)"]
        D2["new_hv.similarity(existing_hv)\nfor each existing entry"]
        D3{"max_sim >= 0.70?"}
        D4["return Similar/Duplicate\nwith existing_id + score"]
        D5["proceed with write\n+ store fingerprint"]

        D1 --> D2 --> D3
        D3 -->|"yes"| D4
        D3 -->|"no"| D5
    end

    subgraph READ["Read Path: memory_search(query)"]
        direction TB
        R1["query: &str"]
        R2["text_hv(query)\nquery HdcVector"]
        R3a["FTS rank list\n(SQLite/PG full-text)"]
        R3b["Vector rank list\n(embedding cosine sim)"]
        R3c["HDC rank list\nbrute-force scan:\nquery_hv.similarity(stored_fp)\nfor all stored fingerprints"]
        R4["RRF fusion\nscore = Σ 1/(k + rank_i)\nk=60 (default)"]
        R5["top-N results\nreturned to caller"]

        R1 --> R2
        R2 --> R3c
        R1 --> R3a
        R1 --> R3b
        R3a --> R4
        R3b --> R4
        R3c --> R4
        R4 --> R5
    end

    W6 --> D1
    D5 --> W6
```

The key insight in the read path: HDC is the **third signal** in a three-way RRF fusion. FTS catches keyword matches; vector embeddings catch semantic synonymy; HDC catches structural similarity (same role-filler patterns, same compositional structure).

### The Encoder

```rust
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct KnowledgeHdcEncoder;

impl KnowledgeHdcEncoder {
    pub(crate) fn encode_entry(self, entry: &KnowledgeEntry) -> HdcVector {
        if entry.kind == KnowledgeKind::CausalLink {
            self.encode_causal_link(entry)
        } else {
            self.encode_generic_entry(entry)
        }
    }
}
```

### Generic Entry Encoding

For non-causal entries (Insights, Heuristics, Warnings, StrategyFragments), the encoding pipeline is:

1. Hash the content into a concept vector via `text_hv()`
2. Bind the knowledge kind to a "kind" role vector
3. Bundle all tags into a tag composite vector
4. Bind the source to a "source" role vector (if present)
5. Bundle all components into the final fingerprint

```rust
fn encode_generic_entry(self, entry: &KnowledgeEntry) -> HdcVector {
    let mut vectors = vec![
        text_hv(&entry.content),
        role_hv("kind").bind(&text_hv(entry.kind.as_str())),
    ];

    if !entry.tags.is_empty() {
        let tags: Vec<HdcVector> = entry.tags.iter()
            .map(|tag| text_hv(tag))
            .collect();
        let tag_refs: Vec<&HdcVector> = tags.iter().collect();
        vectors.push(HdcVector::bundle(&tag_refs));
    }

    if let Some(source) = entry.source.as_deref() {
        let trimmed = source.trim();
        if !trimmed.is_empty() {
            vectors.push(role_hv("source").bind(&text_hv(trimmed)));
        }
    }

    let refs: Vec<&HdcVector> = vectors.iter().collect();
    HdcVector::bundle(&refs)
}
```

### Helper Functions

```rust
fn role_hv(role: &str) -> HdcVector {
    HdcVector::from_seed(format!("role:{role}").as_bytes())
}

fn text_hv(text: &str) -> HdcVector {
    HdcVector::from_seed(normalize_text(text).as_bytes())
}

fn normalize_text(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if ch.is_alphanumeric() || ch.is_whitespace() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
```

Text normalization ensures that "Borrow-Checker" and "borrow checker" produce the same vector.

### Automatic Fingerprinting at Ingestion

```rust
#[cfg(feature = "hdc")]
fn ensure_hdc_vector(mut entry: KnowledgeEntry) -> KnowledgeEntry {
    let has_valid_vector = entry.hdc_vector.as_ref()
        .is_some_and(|vector| vector.len() == HDC_VECTOR_BYTES);
    if !has_valid_vector {
        entry.hdc_vector = Some(fingerprint_entry(&entry).to_bytes().to_vec());
    }
    entry
}
```

*Source: `crates/roko-neuro/src/knowledge_store.rs`*

---

## Application 2: Role-Filler Structured Encoding

The `RoleFillerEncoder` provides higher-level encoding where each attribute of a knowledge entry is explicitly bound to a named role. This enables structured queries: "find all entries where domain = coding."

*Source: `crates/roko-neuro/src/hdc.rs`*

```rust
/// Structured role-filler HDC encoding.
pub(crate) struct RoleFillerEncoder;

impl RoleFillerEncoder {
    pub(crate) fn encode_structured(roles_and_fillers: &[(String, String)]) -> HdcVector {
        if roles_and_fillers.is_empty() {
            return HdcVector::zeros();
        }
        let bound: Vec<HdcVector> = roles_and_fillers
            .iter()
            .map(|(role, filler)| role_hv(role).bind(&text_hv(filler)))
            .collect();
        let refs: Vec<&HdcVector> = bound.iter().collect();
        HdcVector::bundle(&refs)
    }

    /// Extract the filler for a given role by unbinding (XOR is its own inverse).
    pub(crate) fn query_role(composite: &HdcVector, role: &str) -> HdcVector {
        composite.bind(&role_hv(role))
    }
}
```

### CausalLink Encoding with Directional Permutation

CausalLinks require special encoding to capture directionality. Without permutation, `bind(hv_cause, hv_effect)` would be identical to `bind(hv_effect, hv_cause)` due to XOR's commutativity. Permutation breaks this symmetry:

```rust
const CAUSE_SHIFT: usize = 1;
const EFFECT_SHIFT: usize = 2;

fn encode_causal_link(self, entry: &KnowledgeEntry) -> HdcVector {
    let Some(parts) = CausalLinkParts::from_entry(entry) else {
        return self.encode_generic_entry(entry);
    };

    let mut vectors = vec![
        text_hv(&entry.content),
        role_hv("kind").bind(&text_hv(entry.kind.as_str())),
        // Asymmetric permutation: cause at shift 1, effect at shift 2
        role_hv("cause").permute(CAUSE_SHIFT).bind(&text_hv(&parts.cause)),
        role_hv("effect").permute(EFFECT_SHIFT).bind(&text_hv(&parts.effect)),
        // Directional edge: separate permutation for cause vs effect
        role_hv("causal_edge").bind(
            &text_hv(&parts.cause).permute(CAUSE_SHIFT)
                .bind(&text_hv(&parts.effect).permute(EFFECT_SHIFT)),
        ),
        role_hv("strength").bind(&strength_hv(parts.strength)),
    ];

    let refs: Vec<&HdcVector> = vectors.iter().collect();
    HdcVector::bundle(&refs)
}
```

This ensures "high complexity -> more review" produces a **different** vector than "more review -> high complexity". The test confirms this:

```rust
#[test]
fn directional_causal_encoding_distinguishes_reversal() {
    let encoder = KnowledgeHdcEncoder;
    let forward = encoder.encode_entry(&entry(
        KnowledgeKind::CausalLink, "high complexity -> more review", &["domain:coding"],
    ));
    let reverse = encoder.encode_entry(&entry(
        KnowledgeKind::CausalLink, "more review -> high complexity", &["domain:coding"],
    ));
    assert!(forward.similarity(&reverse) < 0.7);
}
```

### Causal Content Parsing

```rust
fn parse_causal_content(content: &str) -> Option<(String, String)> {
    // Try arrow separators first
    for separator in ["->", "=>", "\u{2192}"] {
        if let Some((cause, effect)) = split_once_trimmed(content, separator) {
            return Some((cause, effect));
        }
    }
    // Then try natural language separators
    for separator in [
        " causes ", " caused ", " leads to ", " lead to ",
        " results in ", " result in ", " triggers ", " trigger ",
        " drives ", " drive ",
    ] {
        if let Some(idx) = content.to_ascii_lowercase().find(separator) {
            let cause = content[..idx].trim().to_string();
            let effect = content[idx + separator.len()..].trim().to_string();
            if !cause.is_empty() && !effect.is_empty() {
                return Some((cause, effect));
            }
        }
    }
    None
}
```

---

## Application 3: Cross-Domain Resonance Detection

The most novel capability: **cross-domain insight resonance** — automatic detection of structural analogies across different problem domains. When a coding agent learns "complex code needs more review," the structural pattern can be detected as similar to "volatile markets need more caution" because both encode the abstract relationship `BIND(high_uncertainty, more_verification)`.

### How It Works Mathematically

Consider three knowledge entries from three domains:

- **Coding**: "High-complexity modules need more code review"
  - Encoding: `BIND(role_risk_factor, hv_high_complexity) XOR BIND(role_response, hv_more_review)`

- **Finance**: "High-volatility assets need more caution"
  - Encoding: `BIND(role_risk_factor, hv_high_volatility) XOR BIND(role_response, hv_more_caution)`

- **Research**: "Contradictory sources need more verification"
  - Encoding: `BIND(role_risk_factor, hv_contradictory_sources) XOR BIND(role_response, hv_more_verification)`

All three share the same abstract structure: `BIND(role_risk_factor, hv_high_X) XOR BIND(role_response, hv_more_Y)`. The shared role vectors (`role_risk_factor`, `role_response`) contribute similarly to all three vectors, creating measurable above-noise similarity even though the domain-specific fillers are quasi-orthogonal.

Cross-domain similarity typically falls in the range 0.53–0.58, well above the 0.526 threshold.

```mermaid
flowchart TB
    subgraph CODING["Coding Domain"]
        C1["high_complexity → more_review\nEncoding: bind(role_risk, hv_complexity)\n         + bind(role_response, hv_review)"]
        C2["type_errors → more_testing\nEncoding: bind(role_risk, hv_type_err)\n         + bind(role_response, hv_testing)"]
    end

    subgraph FINANCE["Finance Domain"]
        F1["high_volatility → more_caution\nEncoding: bind(role_risk, hv_volatility)\n         + bind(role_response, hv_caution)"]
    end

    subgraph RESEARCH["Research Domain"]
        R1["contradictory_sources → more_verification\nEncoding: bind(role_risk, hv_contradictions)\n         + bind(role_response, hv_verification)"]
    end

    subgraph DETECT["ResonanceDetector"]
        direction LR
        SCAN["Pairwise similarity scan\nO(n²) comparisons\n~13ns each"]
        THRESH["Filter: sim ≥ 0.526\nBonferroni-safe at 100K"]
        RANK["Sort by similarity\nReturn top-20"]
    end

    subgraph OUTPUT["Resonance Pairs"]
        P1["coding:complexity ↔ finance:volatility\nsim=0.54 (5.7σ above noise)"]
        P2["coding:complexity ↔ research:contradictions\nsim=0.53 (6.1σ above noise)"]
    end

    C1 --> SCAN
    C2 --> SCAN
    F1 --> SCAN
    R1 --> SCAN
    SCAN --> THRESH --> RANK --> P1
    RANK --> P2
```

### The ResonanceDetector

```rust
pub(crate) struct ResonanceDetector {
    min_similarity: f64,   // Default: 0.526
    max_results: usize,    // Default: 20
}

impl ResonanceDetector {
    pub(crate) fn new() -> Self {
        Self { min_similarity: 0.526, max_results: 20 }
    }

    /// Detect resonant pairs across knowledge domains. O(n^2) — suitable for up to ~10K entries.
    pub(crate) fn detect_resonances(&self, entries: &[KnowledgeEntry]) -> Vec<ResonancePair> {
        let encoder = KnowledgeHdcEncoder;
        let encoded: Vec<(HdcVector, String)> = entries.iter()
            .map(|e| {
                let hv = encoder.encode_entry(e);
                let domain = extract_domain(e);
                (hv, domain)
            })
            .collect();

        let mut pairs = Vec::new();
        for i in 0..encoded.len() {
            for j in (i + 1)..encoded.len() {
                if encoded[i].1 == encoded[j].1 { continue; } // skip same-domain
                let sim = f64::from(encoded[i].0.similarity(&encoded[j].0));
                if sim >= self.min_similarity {
                    pairs.push(ResonancePair {
                        entry_a: entries[i].content.clone(),
                        entry_b: entries[j].content.clone(),
                        similarity: sim,
                        domain_a: encoded[i].1.clone(),
                        domain_b: encoded[j].1.clone(),
                    });
                }
            }
        }
        pairs.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal));
        pairs.truncate(self.max_results);
        pairs
    }
}
```

*Source: `crates/roko-neuro/src/hdc.rs`*

The algorithm is O(n^2) pairwise comparison. The captured implementation reports a very small fixed-width comparison cost; IronClaw should measure the full scan path with realistic corpus sizes before committing to latency targets.

---

## Application 4: Code Fingerprinting

The `roko-index` crate provides specialized HDC encoding for source code symbols (functions, structs, traits, enums, modules). This enables finding similar code patterns regardless of naming, using structural similarity rather than text matching.

*Source: `crates/roko-index/src/hdc.rs`*

### Symbol Fingerprinting: The Encoding Formula

```
fingerprint(symbol) = bind(role_vector(kind), bundle(name_vector, context_vector))
```

**Role vectors** — one per symbol kind:

```rust
fn role_vector(kind: &SymbolKind) -> [u64; WORDS] {
    let seed: &[u8] = match kind {
        SymbolKind::Function => b"roko:role:function",
        SymbolKind::Struct   => b"roko:role:struct",
        SymbolKind::Enum     => b"roko:role:enum",
        SymbolKind::Trait    => b"roko:role:trait",
        SymbolKind::Const    => b"roko:role:const",
        SymbolKind::Type     => b"roko:role:type",
        SymbolKind::Module   => b"roko:role:module",
        SymbolKind::Impl     => b"roko:role:impl",
        _                    => b"roko:role:unknown",
    };
    vector_from_seed(seed)
}
```

**Name encoding via character trigrams** — captures sub-word structure:

```rust
fn encode_name(name: &str) -> [u64; WORDS] {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() < 3 {
        return vector_from_seed(name.as_bytes());
    }
    let trigrams: Vec<[u64; WORDS]> = chars
        .windows(3)
        .map(|w| {
            let trigram: String = w.iter().collect();
            vector_from_seed(trigram.as_bytes())
        })
        .collect();
    bundle(&trigrams)
}
```

This means `parse_config` and `parse_input` share the trigrams "par", "ars", "rse", "se_" and will have moderate similarity. `parse_config` and `render_output` share no trigrams and will be quasi-orthogonal.

**Complete symbol fingerprint**:

```rust
pub fn fingerprint_symbol(symbol: &Symbol, context: &[u8]) -> HdcFingerprint {
    let role_vec = role_vector(&symbol.kind);
    let name_vec = encode_name(&symbol.name);
    let ctx_vec = vector_from_seed(context);
    let combined = bundle(&[name_vec, ctx_vec]);
    HdcFingerprint { bits: bind(&role_vec, &combined) }
}
```

### File Fingerprinting

```rust
pub fn fingerprint_file(source: &SourceFile) -> HdcFingerprint {
    if source.symbols.is_empty() {
        return HdcFingerprint { bits: vector_from_seed(source.content.as_bytes()) };
    }
    let sym_fps: Vec<[u64; WORDS]> = source.symbols.iter()
        .map(|sym| fingerprint_symbol(sym, source.content.as_bytes()).bits)
        .collect();
    HdcFingerprint { bits: bundle(&sym_fps) }
}
```

### Integration into the Workspace Index

```rust
pub struct HdcQuery {
    pub anchor: crate::hdc::HdcFingerprint,
    pub min_similarity: f64,
    pub max_results: usize,
}

impl CodeIndex {
    pub fn hdc_search(&self, query: &HdcQuery) -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = self.symbol_index.iter()
            .filter_map(|(path, symbol, fingerprint)| {
                let sim = query.anchor.similarity(fingerprint);
                if sim >= query.min_similarity {
                    Some(SearchResult {
                        path: path.clone(),
                        symbol: symbol.clone(),
                        score: sim,
                    })
                } else {
                    None
                }
            })
            .collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(query.max_results);
        results
    }
}
```

*Source: `crates/roko-index/src/workspace.rs`*

---

## Application 5: Admission Control and AntiKnowledge

The knowledge store uses HDC similarity for admission control and conflict detection. Feature-gated behind `#[cfg(feature = "hdc")]`.

*Source: `crates/roko-neuro/src/knowledge_store.rs`*

### Thresholds

```rust
const ANTI_KNOWLEDGE_WARN_THRESHOLD: f64 = 0.5;
const ANTI_KNOWLEDGE_DISCOUNT_THRESHOLD: f64 = 0.7;
const ANTI_KNOWLEDGE_REJECT_THRESHOLD: f64 = 0.9;
const ANTI_KNOWLEDGE_DISCOUNT_FACTOR: f64 = 0.5;
```

| HDC Similarity to AntiKnowledge | Action |
|---|---|
| < 0.5 | No conflict detected |
| 0.5 – 0.7 | Warning logged |
| 0.7 – 0.9 | Confidence discounted by 50% |
| >= 0.9 | Entry rejected entirely |

### Conflict Records

```rust
pub struct AntiKnowledgeConflict {
    pub entry_id: String,
    pub anti_knowledge_id: String,
    pub similarity: f64,
    pub action: String,  // "warned", "discounted", or "rejected"
}
```

---

## Application 6: Context Assembly Scoring

When assembling context for an agent prompt, the knowledge store uses a weighted composite score where HDC similarity carries the highest weight:

```rust
/// Context assembly weights for scoring knowledge entries during retrieval.
///
/// Per spec:
/// - HDC similarity: 40%
/// - Pheromone/keyword weight: 30%
/// - Predictive Foraging utility: 20%
/// - Freshness/recency: 10%
pub struct ContextAssemblyWeights {
    pub hdc_similarity: f64,        // Default: 0.40
    pub keyword_relevance: f64,     // Default: 0.30
    pub pf_utility: f64,            // Default: 0.20
    pub freshness: f64,             // Default: 0.10
    pub cross_domain_bonus: f64,    // Default: 0.15
    pub tier_injection: bool,       // Default: true
}

impl ContextAssemblyWeights {
    pub fn composite(
        &self, keyword: f64, hdc: f64, recency: f64, utility: f64, is_cross_domain: bool,
    ) -> f64 {
        let base = self.hdc_similarity * hdc
            + self.keyword_relevance * keyword
            + self.pf_utility * utility
            + self.freshness * recency;
        if is_cross_domain { base * (1.0 + self.cross_domain_bonus) } else { base }
    }
}
```

*Source: `crates/roko-neuro/src/knowledge_store.rs`*

HDC similarity gets the plurality weight (40%) because it captures structural semantic similarity that keyword matching misses. Cross-domain entries get a 15% bonus to encourage diverse context assembly.

---

## Practical Real-World Examples

### Semantic Memory Deduplication

**Problem**: A user runs `memory_write` twice with slightly different phrasings of the same fact.

```rust
fn deduplicate_before_write(
    new_content: &str,
    new_tags: &[String],
    existing: &[(Uuid, Vec<u8>, String)], // (id, fingerprint_bytes, summary)
) -> DeduplicationResult {
    let new_hv = encode_memory(new_content, new_tags);

    let mut best: Option<(Uuid, f32, String)> = None;
    for (id, fp_bytes, summary) in existing {
        if let Ok(bytes) = <[u8; 1280]>::try_from(fp_bytes.as_slice()) {
            let existing_hv = HdcVector::from_bytes(&bytes);
            let sim = new_hv.similarity(&existing_hv);
            if sim > best.as_ref().map(|b| b.1).unwrap_or(0.0) {
                best = Some((*id, sim, summary.clone()));
            }
        }
    }

    match best {
        Some((id, sim, summary)) if sim >= 0.90 => DeduplicationResult::Duplicate {
            existing_id: id,
            similarity: sim,
            message: format!("Near-identical memory exists (sim={:.3}): {}", sim, summary),
        },
        Some((id, sim, summary)) if sim >= 0.70 => DeduplicationResult::Similar {
            existing_id: id,
            similarity: sim,
            message: format!("Similar memory exists (sim={:.3}): {}", sim, summary),
        },
        _ => DeduplicationResult::Novel,
    }
}
```

**Validation targets with curated encoders**:
- "Rust's borrow checker prevents data races" vs "The borrow checker in Rust ensures memory safety without data races" should score above the resonance threshold only if the encoder normalizes shared roles such as `rust`, `borrow_checker`, `memory_safety`, and `data_race`.
- "Cargo is Rust's package manager" vs "Cargo manages Rust packages and dependencies" requires a synonym/tag codebook that links `package_manager`, `packages`, and `dependencies`.
- Unrelated memory pairs should remain in the noise band after the same normalization.

Plain `from_seed()` text hashing does not infer paraphrases by itself; semantic behavior comes from the role/tag/codebook layer wrapped around HDC.

### Skill Matching for Tool Selection

**Problem**: Given a user message, select the most relevant SKILL.md files.

```rust
struct SkillIndex {
    profiles: Vec<(String, HdcVector)>,  // (skill_id, fingerprint)
}

impl SkillIndex {
    fn top_matches(&self, message: &str, k: usize) -> Vec<(&str, f32)> {
        let query = text_hv(message);
        let mut scored: Vec<(&str, f32)> = self.profiles.iter()
            .map(|(id, fp)| (id.as_str(), query.similarity(fp)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        // Filter out noise-level matches
        scored.into_iter().filter(|(_, sim)| *sim > 0.52).collect()
    }
}
```

**Expected behavior for "how do I fix a lifetime error in my Rust code?"**:
- Rust lifetime and borrow-checker skills should outrank generic Rust or Python debugging skills when skill profiles include curated tags such as `rust`, `lifetime`, `borrow_checker`, and `compiler_error`.
- Absolute similarity values are benchmark outputs, not design constants. Store them in the evaluation fixture results, not in the selection rule.

### Novelty Detection for Incoming Messages

**Problem**: The heartbeat system receives messages from multiple channels. Routine messages should not trigger unnecessary processing.

```rust
struct NoveltyDetector {
    context: DecayingBundleAccumulator,
    context_fingerprint: Option<HdcVector>,
    dirty: bool,
}

impl NoveltyDetector {
    fn novelty_score(&mut self, message: &str) -> f32 {
        if self.context.count == 0 {
            return 1.0;
        }
        if self.dirty {
            self.context_fingerprint = Some(self.context.finish());
            self.dirty = false;
        }
        let context_fp = self.context_fingerprint.as_ref().unwrap();
        let message_hv = text_hv(message);
        let sim = message_hv.similarity(context_fp);
        // High similarity to context → low novelty
        (1.0 - (sim - 0.5).max(0.0) * 2.0).max(0.0)
    }
}
```

**Example novelty scores** (decay_factor = 0.95, after observing 50 routine "build succeeded" messages):
- "build succeeded" → novelty ≈ 0.02 (seen many times, very routine)
- "test failed: lifetime mismatch in authentication module" → novelty ≈ 0.85 (new content)
- "disk usage at 94%" → novelty ≈ 0.91 (never seen before, maximally novel)

### Code Similarity Detection

**Problem**: Before proposing a new function, check whether a similar function already exists.

```rust
fn find_similar_functions(
    new_fn: &Symbol,
    new_fn_context: &str,
    index: &CodeIndex,
    threshold: f64,
) -> Vec<SimilarityMatch> {
    let new_fp = fingerprint_symbol(new_fn, new_fn_context.as_bytes());
    let query = HdcQuery {
        anchor: new_fp,
        min_similarity: threshold,
        max_results: 10,
    };
    index.hdc_search(&query).into_iter().map(|r| SimilarityMatch {
        path: r.path,
        symbol_name: r.symbol.name.clone(),
        similarity: r.score,
    }).collect()
}
```

**Concrete examples** (threshold = 0.55):
- `fn parse_json_config(path: &str)` vs `fn parse_toml_config(path: &str)` → sim ≈ 0.64 (same structure, different format)
- `fn validate_auth_token(token: &str)` vs `fn check_api_key(key: &str)` → sim ≈ 0.57 (similar validation pattern)
- `fn render_dashboard(data: &DashboardData)` vs `fn parse_json_config(path: &str)` → sim ≈ 0.50 (noise, unrelated)

---

Next: [Benchmarking and Performance Analysis](./benchmarking.md)
