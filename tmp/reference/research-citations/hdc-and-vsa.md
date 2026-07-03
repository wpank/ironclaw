# HDC And VSA Citations

Research context for hyperdimensional computing, vector symbolic architectures,
and similarity search. Captured dimensionalities and performance claims should
be treated as design inputs until verified locally.

## Selected References

| Reference | Contribution | Local relevance |
|---|---|---|
| Kanerva (1988), _Sparse Distributed Memory_ | Content-addressable memory in high-dimensional spaces. | Foundation for HDC-style memory and noisy retrieval. |
| Kanerva (2009), "Hyperdimensional Computing" | Bind, bundle, and permute operations over high-dimensional vectors. | Vocabulary for HDC memory and code-search sketches. |
| Kleyko et al. (2022), HDC/VSA survey | Broad survey of representations, encodings, and applications. | Best starting point before choosing an HDC implementation. |
| Johnson and Lindenstrauss (1984) | Random projection distance-preservation result. | Use carefully when reasoning about embedding projection. |
| Charikar (2002), SimHash | Binary locality-sensitive hashing for angular similarity. | Candidate dedupe/search index technique. |
| Plate (1994/2003), HRR | Distributed compositional representations. | Context for VSA binding alternatives. |
| Rachkovskij (2001) | Binary sparse distributed codes. | Basis for binary-vector operations. |
| Levy and Gayler (2008) | VSA framing for structured symbolic representation. | Connects HDC operations to agent memory structures. |
| Frady et al. (2020), Resonator Networks | Factoring high-dimensional representations. | Future optimization idea; not required for an MVP. |
| Malkov and Yashunin (2020), HNSW | Approximate nearest-neighbor graph search. | Practical vector-search baseline or complement to HDC. |

## Use In IronClaw

- Use HDC as a candidate similarity signal, not proof of semantic identity.
- Keep exact content hashes and source provenance for hard memory decisions.
- Benchmark any HDC index against existing workspace search before enforcing it.

Navigation: [README](README.md) | [Memory and Learning](memory-and-learning.md) |
[Math and Statistics](math-and-statistics.md)
