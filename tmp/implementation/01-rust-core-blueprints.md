# Rust Core Blueprints

This document provides compact, complete-enough Rust sketches for the most
portable algorithms in the Roko analysis: HDC vectors, content-addressed
signals, decay, robust statistics, and LinUCB routing.

The code is written as IronClaw-native blueprint code. It intentionally avoids
`roko-*` dependencies.

## HDC Vector Core

```rust
pub const HDC_BITS: usize = 10_240;
pub const HDC_WORDS: usize = HDC_BITS / 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HdcVector {
    words: [u64; HDC_WORDS],
}

impl HdcVector {
    pub fn zero() -> Self {
        Self { words: [0; HDC_WORDS] }
    }

    pub fn seeded(domain: u64, name_hash: u64) -> Self {
        let mut state = domain ^ name_hash ^ 0x9e37_79b9_7f4a_7c15;
        let mut words = [0u64; HDC_WORDS];
        for word in &mut words {
            state = splitmix64(state);
            *word = state;
        }
        Self { words }
    }

    pub fn bind(self, rhs: Self) -> Self {
        let mut out = [0u64; HDC_WORDS];
        for i in 0..HDC_WORDS {
            out[i] = self.words[i] ^ rhs.words[i];
        }
        Self { words: out }
    }

    pub fn permute(self, shift_bits: u32) -> Self {
        let shift_words = (shift_bits as usize / 64) % HDC_WORDS;
        let shift_inner = shift_bits % 64;
        let mut out = [0u64; HDC_WORDS];
        for i in 0..HDC_WORDS {
            let a = self.words[(i + HDC_WORDS - shift_words) % HDC_WORDS];
            let b = self.words[(i + HDC_WORDS - shift_words - 1) % HDC_WORDS];
            out[i] = if shift_inner == 0 {
                a
            } else {
                (a << shift_inner) | (b >> (64 - shift_inner))
            };
        }
        Self { words: out }
    }

    pub fn hamming_distance(self, rhs: Self) -> u32 {
        self.words
            .iter()
            .zip(rhs.words.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum()
    }

    pub fn similarity(self, rhs: Self) -> f64 {
        1.0 - (self.hamming_distance(rhs) as f64 / HDC_BITS as f64)
    }
}

pub fn bundle(vectors: &[HdcVector]) -> HdcVector {
    if vectors.is_empty() {
        return HdcVector::zero();
    }

    let mut out = [0u64; HDC_WORDS];
    for bit in 0..HDC_BITS {
        let word = bit / 64;
        let mask = 1u64 << (bit % 64);
        let ones = vectors.iter().filter(|v| v.words[word] & mask != 0).count();
        if ones * 2 >= vectors.len() {
            out[word] |= mask;
        }
    }
    HdcVector { words: out }
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}
```

Implementation notes:

- `HDC_BITS = 10_240` gives 160 `u64` words, or 1280 bytes per vector.
- Similarity around `0.5` is random. Thresholds above `0.55` are already far
  into the tail for independent vectors; production thresholds should be tuned
  on actual memory/code corpora.
- `bundle()` is intentionally simple. For streaming updates, use a
  `Vec<i16>` vote accumulator instead of rebundling every vector.

## Content-Addressed Signal

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signal {
    pub id: [u8; 32],
    pub kind: SignalKind,
    pub body: serde_json::Value,
    pub score: Score7,
    pub parents: Vec<[u8; 32]>,
    pub taint: Vec<Taint>,
    pub decay: DecayPolicy,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SignalKind {
    Message,
    ToolCall,
    ToolResult,
    GateVerdict,
    Metric,
    Memory,
    Plan,
    Episode,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Score7 {
    pub confidence: f64,
    pub novelty: f64,
    pub utility: f64,
    pub reputation: f64,
    pub precision: f64,
    pub salience: f64,
    pub coherence: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Taint {
    Unverified,
    External,
    UserGenerated,
    LlmGenerated,
    Sensitive,
    Ephemeral,
    Derived,
    Disputed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DecayPolicy {
    None,
    Ttl { expires_at_ms: i64 },
    HalfLife { half_life_secs: f64 },
    Ebbinghaus { stability: f64, last_reinforced_ms: i64 },
}

impl Signal {
    pub fn new(
        kind: SignalKind,
        body: serde_json::Value,
        score: Score7,
        parents: Vec<[u8; 32]>,
        taint: Vec<Taint>,
        decay: DecayPolicy,
        created_at_ms: i64,
    ) -> Self {
        let id = compute_signal_id(&kind, &body, &parents);
        Self { id, kind, body, score, parents, taint, decay, created_at_ms }
    }

    pub fn effective_weight(&self, now_ms: i64) -> f64 {
        let decay = match self.decay {
            DecayPolicy::None => 1.0,
            DecayPolicy::Ttl { expires_at_ms } => {
                if now_ms >= expires_at_ms { 0.0 } else { 1.0 }
            }
            DecayPolicy::HalfLife { half_life_secs } => {
                let age_secs = ((now_ms - self.created_at_ms).max(0) as f64) / 1000.0;
                0.5_f64.powf(age_secs / half_life_secs.max(1.0))
            }
            DecayPolicy::Ebbinghaus { stability, last_reinforced_ms } => {
                let age_secs = ((now_ms - last_reinforced_ms).max(0) as f64) / 1000.0;
                (-age_secs / stability.max(1.0)).exp()
            }
        };
        decay * (
            self.score.confidence * 0.25
                + self.score.utility * 0.25
                + self.score.salience * 0.20
                + self.score.coherence * 0.15
                + self.score.reputation * 0.15
        )
    }
}

fn compute_signal_id(
    kind: &SignalKind,
    body: &serde_json::Value,
    parents: &[[u8; 32]],
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(format!("{kind:?}").as_bytes());
    hasher.update(&serde_json::to_vec(body).unwrap_or_default());
    for parent in parents {
        hasher.update(parent);
    }
    *hasher.finalize().as_bytes()
}
```

IronClaw adaptation:

- Store `id` as a binary digest or hex string depending on backend conventions.
- Add migrations to both PostgreSQL and libSQL if this becomes durable state.
- For workspace memory, route writes through the existing memory service facade
  rather than introducing a second memory store.

## Robust Statistics

```rust
pub fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.total_cmp(b));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        Some((values[mid - 1] + values[mid]) / 2.0)
    } else {
        Some(values[mid])
    }
}

pub fn trimmed_mean(values: &mut [f64], trim: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.total_cmp(b));
    let trim_count = ((values.len() as f64) * trim.clamp(0.0, 0.49)).floor() as usize;
    let kept = &values[trim_count..values.len() - trim_count];
    Some(kept.iter().sum::<f64>() / kept.len() as f64)
}

pub fn mad(values: &[f64]) -> Option<f64> {
    let mut center_input = values.to_vec();
    let center = median(&mut center_input)?;
    let mut deviations: Vec<f64> = values.iter().map(|v| (v - center).abs()).collect();
    median(&mut deviations).map(|raw| raw * 1.4826)
}
```

Use cases:

- `src/estimation/` cost and duration summaries.
- LLM provider latency summaries.
- Gate pass-rate regression baselines.
- Conductor threshold initialization.

## LinUCB Router Core

```rust
#[derive(Clone, Debug)]
pub struct LinUcbArm {
    // A starts as lambda * I. This sketch stores dense row-major matrices.
    a: Vec<f64>,
    b: Vec<f64>,
    dim: usize,
}

impl LinUcbArm {
    pub fn new(dim: usize, lambda: f64) -> Self {
        let mut a = vec![0.0; dim * dim];
        for i in 0..dim {
            a[i * dim + i] = lambda;
        }
        Self { a, b: vec![0.0; dim], dim }
    }

    pub fn score(&self, x: &[f64], alpha: f64) -> f64 {
        let a_inv = invert_small_matrix(&self.a, self.dim);
        let theta = mat_vec(&a_inv, &self.b, self.dim);
        let exploit = dot(&theta, x);
        let explore = dot(x, &mat_vec(&a_inv, x, self.dim)).max(0.0).sqrt();
        exploit + alpha * explore
    }

    pub fn update(&mut self, x: &[f64], reward: f64, discount: f64) {
        for v in &mut self.a {
            *v *= discount;
        }
        for v in &mut self.b {
            *v *= discount;
        }
        for r in 0..self.dim {
            for c in 0..self.dim {
                self.a[r * self.dim + c] += x[r] * x[c];
            }
            self.b[r] += reward * x[r];
        }
    }
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn mat_vec(a: &[f64], x: &[f64], dim: usize) -> Vec<f64> {
    let mut out = vec![0.0; dim];
    for r in 0..dim {
        for c in 0..dim {
            out[r] += a[r * dim + c] * x[c];
        }
    }
    out
}

fn invert_small_matrix(a: &[f64], dim: usize) -> Vec<f64> {
    // Blueprint only: production should use a tested Cholesky or Gauss-Jordan
    // implementation with singularity handling and regression tests.
    let mut m = vec![0.0; dim * dim * 2];
    for r in 0..dim {
        for c in 0..dim {
            m[r * 2 * dim + c] = a[r * dim + c];
        }
        m[r * 2 * dim + dim + r] = 1.0;
    }
    for pivot in 0..dim {
        let p = m[pivot * 2 * dim + pivot].max(1e-12);
        for c in 0..2 * dim {
            m[pivot * 2 * dim + c] /= p;
        }
        for r in 0..dim {
            if r == pivot {
                continue;
            }
            let factor = m[r * 2 * dim + pivot];
            for c in 0..2 * dim {
                m[r * 2 * dim + c] -= factor * m[pivot * 2 * dim + c];
            }
        }
    }
    let mut inv = vec![0.0; dim * dim];
    for r in 0..dim {
        for c in 0..dim {
            inv[r * dim + c] = m[r * 2 * dim + dim + c];
        }
    }
    inv
}
```

Reward formula for initial deployment:

```text
reward = 0.55 * quality_pass
       + 0.20 * latency_score
       + 0.20 * cost_score
       + 0.05 * reliability_score
```

Where `cost_score = clamp(1 - actual_cost / baseline_cost, 0, 1)` and
`latency_score = clamp(1 - actual_latency / latency_budget, 0, 1)`.

