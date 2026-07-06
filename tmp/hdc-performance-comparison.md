# HDC Performance Comparison

Comparing IronClaw's Hyperdimensional Computing (HDC) fingerprinting against typical embedding-based approaches for memory deduplication and similarity search.

## Storage Footprint

| Approach | Vector Size | Notes |
|----------|-------------|-------|
| **HDC fingerprint** | **1,280 bytes** | 10,240-bit binary vector (`[u64; 160]`) |
| OpenAI text-embedding-3-small (1536d) | 6,144 bytes | 1536 x float32 |
| OpenAI text-embedding-3-large (3072d) | 12,288 bytes | 3072 x float32 |
| Cohere embed-v3 (1024d) | 4,096 bytes | 1024 x float32 |
| Voyage-2 (1024d) | 4,096 bytes | 1024 x float32 |

HDC fingerprints are **4.8x smaller** than typical 1536-dimensional embeddings. For 100K memory entries:

- HDC: ~122 MB
- 1536d float32: ~586 MB

## Compute Characteristics

Use this table as a comparison frame, not a promise. Local HDC numbers must be
filled in from `cargo bench -p ironclaw_hdc` and the workspace smoke tests on
the machine/configuration being evaluated.

| Property | HDC | Embedding APIs |
|----------|-----|----------------|
| **Compute location** | Pure CPU, local | GPU (remote API call) |
| **Latency per encode** | Measure with `bench_encode_text_1kb` / `bench_encode_document_1kb_5tags` | Often dominated by network + provider inference |
| **Similarity comparison** | Measure with `bench_similarity` / `bench_hamming` | Measure dot product or ANN query in the deployed stack |
| **Batch throughput** | Measured locally; no provider rate limit | Rate-limited by provider/API quota |
| **GPU required** | No | Yes (for provider inference) |
| **Network required** | No | Yes |
| **API key required** | No | Yes |

## Scan Performance

Brute-force scanning (no index structure needed):

| Corpus Size | HDC top-k scan | ANN index query | Notes |
|-------------|----------------|-----------------|-------|
| 1K vectors | Benchmark locally | Benchmark deployed ANN stack | Both are usually small enough for interactive diagnostics |
| 10K vectors | Benchmark locally | Benchmark deployed ANN stack | Watch p95 scan latency before enabling live gating |
| 100K vectors | Benchmark locally | Benchmark deployed ANN stack | ANN likely wins latency; HDC has simpler maintenance |
| 1M vectors | Requires an index/cache decision | Benchmark deployed ANN stack | Brute-force HDC is probably not the right default |

HDC's maintenance advantage is that a flat scan does not need HNSW graph
building, IVF training, or index rebuilds after inserts. That advantage should
be weighed against measured p50/p95 scan latency for the actual workspace size.

## Determinism

| Property | HDC | Neural Embeddings |
|----------|-----|-------------------|
| **Same input -> same output** | Deterministic under a fixed encoder/version | Varies by model version, API state |
| **Cross-platform consistency** | Identical (platform-independent math) | Depends on floating point implementation |
| **Model versioning** | Encoder version is local and testable | Output changes when provider updates model |
| **Reproducibility** | Byte-for-byte golden tests are practical | Approximate at best |

HDC fingerprints use FNV-1a hashing and SplitMix64 PRNG with integer
operations. The same input should produce the same fingerprint across platforms
as long as the encoder and seeded-vector algorithm stay version-stable.

## Offline Operation

| Capability | HDC | Embedding APIs |
|------------|-----|----------------|
| Works without internet | Yes | No |
| Works without API keys | Yes | No |
| Works in air-gapped environments | Yes | No |
| Works during provider outages | Yes | No |
| Startup time | Instant | Connection establishment |
| Cost per operation | No provider bill; local CPU/storage cost still exists | Provider billing per token/request |

For IronClaw's use case (personal AI assistant), offline capability is critical. The workspace memory system must function regardless of network state.

## Complementary Strengths

HDC and neural embeddings detect different kinds of similarity:

| Dimension | HDC | Neural Embeddings |
|-----------|-----|-------------------|
| **Structural/syntactic** | Strong -- byte trigrams capture surface form | Weak -- abstracts away surface form |
| **Semantic/conceptual** | Moderate -- tag/path signals help | Strong -- trained on meaning |
| **Exact phrase matching** | Strong -- trigram overlap is high | Weak -- may miss exact matches |
| **Paraphrase detection** | Weak -- different words = different trigrams | Strong -- understands meaning |
| **Cross-language** | None -- byte-level encoding | Multilingual models handle this |

### When HDC Is a Good Fit

- Detecting copy-paste duplicates with minor edits
- Finding files that share structural patterns (same codebase conventions)
- Catching accidental re-saves or trivial rewrites
- Comparing paths and metadata (not just content)

### When Embeddings Are a Better Fit

- Finding semantically related content with different wording
- Cross-document concept linking ("Rust ownership" is related to "C++ RAII")
- Understanding intent behind queries

### IronClaw's Approach: Both

IronClaw can use HDC as a local deterministic structure signal and embeddings
as a semantic enrichment layer:

1. **Write path**: HDC fingerprint computed locally when shadow/dedup mode is
   enabled; write-path overhead must be measured before promotion.
2. **Search path**: HDC similarity can annotate existing search results; live
   fusion remains fixture- and metric-gated.
3. **Heartbeat path**: Decaying HDC accumulator can classify novelty locally; no
   embedding provider call is needed for the HDC classification itself.

This gives IronClaw two complementary signals: local deterministic structure
matching, plus semantic retrieval when an embedding provider is configured.

## Memory Encoding Details

IronClaw's document fingerprint combines three signals with weighted bundling:

```
doc_fingerprint = weighted_bundle(
    content_hv * 5,  // byte-trigram encoding of document body
    tag_hv * 3,      // role-bound tag vectors
    path_hv * 2,     // position-encoded path segments
)
```

This means two documents are similar when they share:
- Textual content (strongest signal, weight 5)
- Tags/labels (weight 3)
- File path structure (weight 2)

The weighting ensures content matters most, but metadata can still contribute to
the fingerprint. Current live workspace writes pass `tags: &[]`; tag-aware
backfill/live-write normalization must be resolved before using tag-sensitive
quality claims.
