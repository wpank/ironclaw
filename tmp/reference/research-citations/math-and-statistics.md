# Math And Statistics Citations

Research context for topology, projection, changepoints, calibration, control,
forgetting, and approximate search.

## Selected References

| Reference | Contribution | Local relevance |
|---|---|---|
| Takens (1981) | Delay embeddings for dynamical systems. | Research context for time-series shape analysis. |
| Carlsson (2009) | Topological data analysis overview. | Background for TDA anomaly sketches. |
| Bubenik (2015), Bauer (2021) | Persistence landscapes and efficient persistence computation. | Candidate analysis tools, not default runtime dependencies. |
| Johnson and Lindenstrauss (1984) | Distance-preserving random projection. | Useful when reasoning about embeddings and binary projections. |
| Nemhauser et al. (1978) | Submodular optimization. | Greedy context selection analogy. |
| Peters (2019), Kelly (1956) | Ergodicity and log-growth allocation. | Budget-sizing analogy; validate before implementation. |
| Adams and MacKay (2007), Killick et al. (2012) | Online and offline changepoint detection. | Provider/gate drift candidates. |
| Page (1954), Roberts (1959) | CUSUM and EWMA process control. | Lightweight monitor candidates. |
| Guo et al. (2017), Vovk et al. (2005) | Calibration and conformal prediction. | Confidence needs measured calibration. |
| Ebbinghaus (1885), Richards and Frankland (2017) | Forgetting and active memory decay. | Retrieval weighting and stale-memory handling. |
| Charikar (2002), Malkov and Yashunin (2020) | SimHash and HNSW approximate search. | Search-index implementation options. |

## Use In IronClaw

- Prefer simple, measurable statistics before complex topology or control
  machinery.
- Keep thresholds fixture-defined and rollout-controlled.
- Treat mathematical guarantees as valid only when local assumptions match.

Navigation: [README](README.md) | [Verification and Safety](verification-and-safety.md) |
[HDC and VSA](hdc-and-vsa.md)
