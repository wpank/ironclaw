# Mathematical Primitives

**Source crate**: `roko-primitives` (`crates/roko-primitives/src/`)
**Priority**: LOW -- advanced math; useful for specialized analytics
**Relevant roko task identifiers**: TA-06 (manifolds), TA-09 (TDA), TA-10 (robust statistics), TA-13 (sheaves), TA-14 (tropical algebra)

---

## Table of Contents

1. [Overview and Motivation](#overview-and-motivation)
2. [Topological Data Analysis (TDA)](#1-topological-data-analysis-tda)
   - [The Problem TDA Solves](#the-problem-tda-solves)
   - [Takens Delay Embedding](#takens-delay-embedding)
   - [Simplicial Complexes and the Vietoris-Rips Construction](#simplicial-complexes-and-the-vietoris-rips-construction)
   - [Persistent Homology](#persistent-homology)
   - [Persistence Diagrams and Barcodes](#persistence-diagrams-and-barcodes)
   - [Persistence Landscapes](#persistence-landscapes)
   - [The Full TDA Pipeline in Code](#the-full-tda-pipeline-in-code)
   - [Distances Between Diagrams](#distances-between-diagrams)
   - [Practical Application: Agent Loop Detection](#practical-application-agent-loop-detection)
3. [Cellular Sheaves](#2-cellular-sheaves)
   - [What Sheaves Model](#what-sheaves-model)
   - [Stalks and Restriction Maps](#stalks-and-restriction-maps)
   - [The Coboundary Operator](#the-coboundary-operator)
   - [The Sheaf Laplacian](#the-sheaf-laplacian)
   - [Inconsistency Detection](#inconsistency-detection)
   - [Identifying the Outlier Oracle](#identifying-the-outlier-oracle)
   - [Spectral Analysis and Eigenvalues](#spectral-analysis-and-eigenvalues)
   - [Practical Application: Multi-Source Verification](#practical-application-multi-source-verification)
4. [Riemannian Geometry](#3-riemannian-geometry)
   - [Manifolds and the Cost Landscape](#manifolds-and-the-cost-landscape)
   - [The Metric Tensor](#the-metric-tensor)
   - [Christoffel Symbols and Connections](#christoffel-symbols-and-connections)
   - [Geodesics and the RK4 Solver](#geodesics-and-the-rk4-solver)
   - [Geodesic Distance](#geodesic-distance)
   - [Ricci Curvature](#ricci-curvature)
   - [Frechet Mean](#frechet-mean)
   - [Practical Application: Optimal Configuration Transitions](#practical-application-optimal-configuration-transitions)
5. [Tropical Algebra (Max-Plus Semiring)](#4-tropical-algebra-max-plus-semiring)
   - [What Is a Semiring](#what-is-a-semiring)
   - [The Max-Plus Semiring](#the-max-plus-semiring)
   - [Tropical Polynomials](#tropical-polynomials)
   - [Tropical Matrices](#tropical-matrices)
   - [Tropical Attention](#tropical-attention)
   - [Adversarial Distance](#adversarial-distance)
   - [Practical Application: Decision Boundary Analysis](#practical-application-decision-boundary-analysis)
6. [Robust Statistics](#5-robust-statistics)
   - [Why Standard Statistics Fail](#why-standard-statistics-fail)
   - [Trimmed Mean](#trimmed-mean)
   - [Median Absolute Deviation (MAD)](#median-absolute-deviation-mad)
   - [Hodges-Lehmann Estimator](#hodges-lehmann-estimator)
   - [Summary of Breakdown Points](#summary-of-breakdown-points)
   - [Practical Application: Reliable Metric Aggregation](#practical-application-reliable-metric-aggregation)
7. [Additional Primitives: HDC and Codebooks](#6-additional-primitives-hdc-and-codebooks)
8. [IronClaw Integration Plan](#ironclaw-integration-plan)
   - [Tier 1: Robust Statistics](#tier-1-immediate-value--robust-statistics)
   - [Tier 2: TDA for Execution Traces](#tier-2-interesting--tda-for-execution-traces)
   - [Tier 3: Sheaf Consistency](#tier-3-research-grade--sheaf-consistency-for-multi-source-verification)
   - [Tier 4: Riemannian and Tropical](#tier-4-long-term-research--riemannian-cost-optimization-and-tropical-decision-analysis)
9. [Complexity Assessment](#complexity-assessment)
10. [References](#references)

---

## Overview and Motivation

The `roko-primitives` crate contains a collection of pure mathematical tools spanning topology, differential geometry, abstract algebra, and robust statistics. These are not standard software engineering tools -- they are research-grade mathematical constructions adapted for software agent intelligence. The crate has zero internal workspace dependencies (`#![deny(unsafe_code)]`, no async, no I/O) and can be imported independently of the full roko platform.

The key question these tools answer: **How do you rigorously analyze the behavior of an AI agent and the data it processes, going beyond simple averages and counts?**

| Mathematical Domain | Core Question It Answers | AI Agent Use Case |
|---|---|---|
| **TDA** | What is the *shape* of this time series data? | Detect agent loops, convergence, regime changes |
| **Cellular Sheaves** | When multiple sources disagree, *which one* is the outlier? | Multi-source information consistency checking |
| **Riemannian Geometry** | What is the *optimal path* through non-linear cost space? | Find minimum-cost configuration transitions |
| **Tropical Algebra** | What are the exact *decision boundaries* of a piecewise-linear system? | Analyze routing logic, compute adversarial robustness |
| **Robust Statistics** | What is the *true center* and *true spread* of noisy data? | Reliable metric aggregation under noise/adversaries |

Each section below explains the mathematical concept from first principles (assuming the reader knows basic linear algebra but nothing about these specific topics), then shows the actual roko implementation with verified file paths and line numbers.

---

## 1. Topological Data Analysis (TDA)

**Source file**: `crates/roko-primitives/src/tda.rs` (575 lines)
**roko task**: TA-09
**roko docs**: `docs/v1/20-technical-analysis/10-predictive-geometry-and-resonant-patterns.md`, `docs/v2-depth/21-roadmap/06-advanced-geometry-and-integration.md`

### The Problem TDA Solves

Standard statistical analysis of a time series tells you its mean, variance, trend, and correlation with other series. But it misses *shape*. Consider two time series:

```
Series A:  ___/\___/\___     (two peaks, a valley)
Series B:  __________/       (gradual rise)
```

Both could have the same mean and variance. But their *topological* structures are completely different -- Series A has a loop-like structure in phase space (it returns to near its starting value), while Series B does not. TDA detects these shape differences.

The mathematical foundation is **algebraic topology** applied to point cloud data. Instead of working with continuous spaces (like a sphere or torus), TDA works with finite sets of points and builds topological structures on top of them. The key insight, established in Carlsson's foundational survey [1], is that topological invariants -- properties preserved under continuous deformation -- can be computed from discrete data and reveal structure invisible to traditional statistics.

### Takens Delay Embedding

**The idea**: A scalar time series `x(t)` is a 1-dimensional projection of a higher-dimensional dynamical system. Takens' Embedding Theorem (1981) [2] proves that you can reconstruct the topology of the original system from this 1D projection alone, by creating delay vectors.

**How it works**: Given a time series `[x_1, x_2, x_3, ..., x_n]`, an embedding dimension `d`, and a delay parameter `tau`, you construct `d`-dimensional points:

```
Point 0: [x(0),     x(tau),     x(2*tau),     ..., x((d-1)*tau)]
Point 1: [x(1),     x(1+tau),   x(1+2*tau),   ..., x(1+(d-1)*tau)]
Point 2: [x(2),     x(2+tau),   x(2+2*tau),   ..., x(2+(d-1)*tau)]
...
```

**Concrete example with d=3, tau=2**:

```
Input series:  [1, 2, 3, 4, 5, 6, 7, 8]

Point 0:  [x(0), x(2), x(4)] = [1, 3, 5]
Point 1:  [x(1), x(3), x(5)] = [2, 4, 6]
Point 2:  [x(2), x(4), x(6)] = [3, 5, 7]
Point 3:  [x(3), x(5), x(7)] = [4, 6, 8]
```

A linear series produces a straight line in the embedded space. A sinusoidal series produces an ellipse. A chaotic series produces a strange attractor. The key insight is that the *topology* of this point cloud reveals the *dynamics* of the underlying system.

**The roko implementation** (`tda.rs`, lines 141-158):

```rust
pub fn takens_embedding(series: &[f64], dim: usize, tau: usize) -> Vec<Vec<f64>> {
    if dim == 0 || tau == 0 || series.len() < (dim - 1) * tau + 1 {
        return Vec::new();
    }

    let n = series.len() - (dim - 1) * tau;
    let mut points = Vec::with_capacity(n);

    for i in 0..n {
        let mut point = Vec::with_capacity(dim);
        for d in 0..dim {
            point.push(series[i + d * tau]);
        }
        points.push(point);
    }

    points
}
```

The function returns `Vec::new()` for edge cases (zero dimension, zero delay, or insufficient data). The number of output points is `n = len(series) - (dim - 1) * tau`.

**Parameter guidance**:
- `dim`: Typically 2 or 3. Takens' theorem requires `d >= 2 * D + 1` where `D` is the true dimension of the attractor, but in practice 2-3 works for most time series.
- `tau`: The delay parameter. Too small makes adjacent points nearly identical; too large loses temporal correlation. A common heuristic is the first minimum of the mutual information function. Roko's docs note that a fixed `tau=1, d=3` is the naive starting point.

### Simplicial Complexes and the Vietoris-Rips Construction

**Background**: A simplicial complex is a generalization of a graph that can represent higher-dimensional relationships.

- A **0-simplex** is a point (vertex)
- A **1-simplex** is a line segment (edge) connecting two points
- A **2-simplex** is a filled triangle connecting three points
- A **3-simplex** is a filled tetrahedron, and so on

The **Vietoris-Rips complex** at scale parameter `epsilon` takes a point cloud and connects every pair of points that are within distance `epsilon` of each other. If all pairwise distances in a set of `k+1` points are within `epsilon`, they form a `k`-simplex.

```
                         epsilon = 0.5          epsilon = 1.5
Point cloud:
  A---B                  A---B                  A---B
  |                      |                      |\ /|
  C     D                C     D                C---D
                                                 (filled)

At small epsilon: disconnected points
At medium epsilon: edges and triangles appear
At large epsilon: everything fills in
```

**Mathematical definition**: The Vietoris-Rips complex VR(X, epsilon) on a finite metric space (X, d) at scale epsilon is the abstract simplicial complex where a subset sigma of X is a simplex if and only if d(x, y) <= epsilon for all x, y in sigma.

### Persistent Homology

**The key idea**: Instead of choosing a single scale `epsilon`, persistent homology tracks how topological features *change* as you sweep `epsilon` from 0 to infinity. Features that persist across a wide range of scales are "real" structure; features that appear and disappear quickly are noise.

**Homological features by dimension**:
- **H0** (dimension 0): Connected components. At `epsilon=0`, every point is its own component. As `epsilon` grows, components merge.
- **H1** (dimension 1): Loops (1-cycles). A loop is born when edges form a cycle and dies when the interior fills in with triangles.
- **H2** (dimension 2): Voids (cavities). Rare in low-dimensional data.

For a rigorous treatment of persistent homology and its stability properties, see Edelsbrunner and Harer [3].

**The roko implementation** (`tda.rs`, lines 233-324) computes Vietoris-Rips persistent homology:

1. **Compute pairwise distances** (lines 170-181):
```rust
fn distance_matrix(points: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = points.len();
    let mut dist = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let d = euclidean_distance(&points[i], &points[j]);
            dist[i][j] = d;
            dist[j][i] = d;
        }
    }
    dist
}
```

2. **Sort all edges by distance** (lines 243-249):
```rust
let mut edges: Vec<(f64, usize, usize)> = Vec::with_capacity(n * (n - 1) / 2);
for i in 0..n {
    for j in (i + 1)..n {
        edges.push((dist[i][j], i, j));
    }
}
edges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
```

3. **Track H0 features via Union-Find** (lines 254-283): Each point starts as its own connected component (born at `epsilon=0`). When two components merge at distance `d`, the younger one "dies" at `d`. The Union-Find data structure (lines 184-220) tracks this efficiently with path compression and union by rank.

4. **Approximate H1 features via cycle detection** (lines 289-321): When an edge is added between two already-connected vertices, it creates a cycle. The implementation checks for triangles (common neighbors) to estimate when loops are born and when they fill in.

### Persistence Diagrams and Barcodes

A **persistence diagram** is a multiset of points `(birth, death)` in the plane. Each point represents one topological feature.

```
  death
    |
  5 |         x          <- Long-lived feature (real structure)
    |
  3 |     x              <- Medium-lived feature
    |   x                <- Short-lived feature (noise)
  1 | x                  <- Short-lived feature (noise)
    |________________
    0  1  2  3  4  5  birth

Points near the diagonal (death ~ birth) = noise
Points far from the diagonal = genuine structure
```

An equivalent representation is the **barcode diagram**, where each feature is a horizontal bar from birth to death:

```
Feature 1: |====|                        (birth=0.0, death=1.0)
Feature 2: |==========|                  (birth=0.0, death=3.0)
Feature 3:    |====|                     (birth=0.5, death=1.5)
Feature 4: |========================|   (birth=0.0, death=5.0)  <- most significant
```

**The roko data types** (`tda.rs`, lines 20-89):

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PersistencePoint {
    pub birth: f64,
    pub death: f64,
    pub dimension: usize,  // 0 = connected component, 1 = loop, 2 = void
}

impl PersistencePoint {
    pub fn persistence(&self) -> f64 {
        self.death - self.birth  // lifetime of the feature
    }
}

#[derive(Debug, Clone, Default)]
pub struct PersistenceDiagram {
    pub points: Vec<PersistencePoint>,
}
```

The diagram supports filtering by dimension (`points_at_dim`), computing total persistence (sum of all lifetimes), and maximum persistence (the single most significant feature).

### Persistence Landscapes

**The problem with persistence diagrams**: They are multisets of points, not vectors. You cannot compute a "mean persistence diagram" or take a "standard deviation" in a meaningful way. Persistence landscapes, introduced by Bubenik [4], solve this by converting diagrams into functions that live in a Banach space (a complete normed vector space).

**How it works**: For each persistence point `(b, d)`, define a "tent function":

```
Lambda(t) = min(t - b, d - t)   if b <= t <= d
           = 0                    otherwise
```

This creates a triangle (tent) centered at the midpoint `(b+d)/2` with height `(d-b)/2`.

```
      /\
     /  \
    /    \      <- tent for (birth=1, death=5), peak at t=3, height=2
   /      \
  /        \
_/          \___
 1    3    5
```

The **level-k landscape** at each parameter value `t` is the k-th largest tent function value. Level 0 is the maximum over all tents. This converts the diagram into a piecewise-linear function that supports addition, subtraction, scaling, mean, and variance.

**The roko implementation** (`tda.rs`, lines 337-382):

```rust
pub fn persistence_landscape(
    diagram: &PersistenceDiagram,
    dim: usize,
    resolution: usize,
) -> Vec<f64> {
    let points: Vec<&PersistencePoint> = diagram.points_at_dim(dim);
    if points.is_empty() || resolution == 0 {
        return vec![0.0; resolution];
    }

    // Determine parameter range.
    let min_birth = points.iter().map(|p| p.birth).fold(f64::INFINITY, f64::min);
    let max_death = points.iter().map(|p| p.death).fold(f64::NEG_INFINITY, f64::max);

    if (max_death - min_birth).abs() < f64::EPSILON {
        return vec![0.0; resolution];
    }

    let step = (max_death - min_birth) / resolution as f64;
    let mut landscape = Vec::with_capacity(resolution);

    for k in 0..resolution {
        let t = min_birth + (k as f64 + 0.5) * step;

        let max_tent = points
            .iter()
            .map(|p| {
                if t >= p.birth && t <= p.death {
                    (t - p.birth).min(p.death - t)
                } else {
                    0.0
                }
            })
            .fold(0.0_f64, f64::max);

        landscape.push(max_tent);
    }

    landscape
}
```

The function samples the level-0 landscape at `resolution` evenly-spaced points and returns a vector of `f64` values. This vector can be compared with standard L2 distance.

### The Full TDA Pipeline in Code

The complete pipeline from time series to topological comparison, as demonstrated in the test at line 559:

```rust
// 1. Generate or obtain a time series
let series: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin()).collect();

// 2. Takens delay embedding: 1D series -> 2D point cloud
let embedded = takens_embedding(&series, 2, 5);

// 3. Vietoris-Rips persistent homology: point cloud -> persistence diagram
let diagram = vietoris_rips(&embedded, 1);  // max_dim=1 includes H0 and H1

// 4. Persistence landscape: diagram -> vector for comparison
let landscape = persistence_landscape(&diagram, 0, 50);

// 5. Compare with another landscape
let other_landscape = persistence_landscape(&other_diagram, 0, 50);
let distance = landscape_distance(&landscape, &other_landscape);
```

### Distances Between Diagrams

The crate provides two distance measures:

1. **Landscape L2 distance** (`tda.rs`, lines 384-391): Standard Euclidean distance between landscape vectors. Fast and suitable for statistical analysis.

```rust
pub fn landscape_distance(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    let sum_sq: f64 = (0..n).map(|i| (a[i] - b[i]).powi(2)).sum();
    let extra_a: f64 = a[n..].iter().map(|x| x.powi(2)).sum();
    let extra_b: f64 = b[n..].iter().map(|x| x.powi(2)).sum();
    (sum_sq + extra_a + extra_b).sqrt()
}
```

2. **Bottleneck distance** (`tda.rs`, lines 95-128): The maximum over all matched pairs of the L-infinity distance. This is a greedy approximation of the true bottleneck distance (which requires solving an optimal matching problem). It handles unmatched points by projecting them to the diagonal with distance `persistence / 2`.

### Practical Application: Agent Loop Detection

An AI agent executing tasks generates a time series of latencies, error counts, or resource usage. TDA on this time series can detect behavioral patterns invisible to simple statistics:

- **Cyclic behavior** (H1 features): The agent stuck in a retry loop produces a time series with strong H1 features (loops in phase space). The `max_persistence()` on H1 features exceeding a threshold signals this.
- **Convergence** (shrinking H0 features): An agent approaching a solution shows decreasing H0 feature lifetimes -- the point cloud in phase space is tightening.
- **Regime change** (bottleneck distance spike): When the bottleneck distance between successive persistence diagrams spikes, the system dynamics have fundamentally changed.

**Why TDA instead of simpler methods?** A fixed threshold on latency variance misses the case where the agent oscillates between two modes -- both with low variance but with a clear topological loop. TDA captures this shape.

---

## 2. Cellular Sheaves

**Source file**: `crates/roko-primitives/src/sheaf.rs` (789 lines)
**roko task**: TA-13
**roko docs**: `docs/v1/20-technical-analysis/14-sheaf-tropical-geometry.md`, `docs/v2-depth/21-roadmap/06-advanced-geometry-and-integration.md`

### What Sheaves Model

Imagine three weather stations reporting temperature:

```
Station A: 72 F
Station B: 73 F
Station C: 95 F   <- something is wrong here
```

Pairwise comparison tells you C disagrees with A and C disagrees with B. But with a more complex network of sensors measuring different quantities (temperature, humidity, pressure), pairwise comparisons miss *structural* inconsistencies -- cases where A and B agree, B and C agree, but A and C disagree in a way that is only detectable through the transitive chain.

A **cellular sheaf** is the mathematical framework for exactly this problem. Introduced in the computational setting by Curry [6] and developed into a spectral theory by Hansen and Ghrist [5], a cellular sheaf assigns a vector space to each node in a network (its "stalk" -- the local data space) and linear maps to each edge (its "restriction maps" -- the rules for comparing adjacent nodes). The sheaf Laplacian then measures global consistency in a single algebraic operation.

In roko's context, the "nodes" are different oracle subsystems (chain analysis, code review, research, etc.) that produce predictions, and the "edges" represent which pairs of oracles should agree. From `docs/v1/20-technical-analysis/14-sheaf-tropical-geometry.md`:

> "Roko's 9 TA subsystems each produce predictions that must be **locally consistent** -- the chain oracle's price prediction should cohere with the liquidity manifold's execution cost estimate."

### Stalks and Restriction Maps

**Stalks**: Each vertex (oracle/observer) has a stalk -- a vector space representing its prediction domain. For example, if Oracle A produces 4-dimensional predictions `[price, volume, gas, risk]`, its stalk is R^4.

**Restriction maps**: Each edge connecting two vertices has two restriction maps -- one from each endpoint's stalk into a shared "comparison space." These maps project each oracle's prediction into the dimensions where they can be meaningfully compared.

**Example**: Oracle A predicts `[price, volume, gas, risk]` (4D). Oracle B predicts `[price, risk]` (2D). The restriction map from A projects onto components 0 and 3 (price and risk). The restriction map from B is the identity (it already produces just price and risk). The consistency check verifies that A's projected `[price, risk]` matches B's `[price, risk]`.

**The roko implementation** (`sheaf.rs`, lines 38-123):

```rust
pub struct RestrictionMap {
    pub rows: usize,       // dimension of the edge stalk (comparison space)
    pub cols: usize,       // dimension of the vertex stalk
    pub data: Vec<f64>,    // row-major matrix data
}

impl RestrictionMap {
    /// Identity map: vertex and edge have the same dimension
    pub fn identity(dim: usize) -> Self { /* ... */ }

    /// Projection: select specific components from the vertex stalk
    /// e.g., projection(4, &[0, 3]) selects components 0 and 3
    pub fn projection(vertex_dim: usize, indices: &[usize]) -> Self { /* ... */ }

    /// Apply the restriction map to a vector: M * v
    pub fn apply(&self, v: &[f64]) -> Vec<f64> { /* ... */ }

    /// Compute the transpose of this map
    pub fn transpose(&self) -> RestrictionMap { /* ... */ }
}
```

The `CellularSheaf` struct (lines 146-216) holds stalk dimensions per vertex and edges with their restriction maps:

```rust
pub struct CellularSheaf {
    stalk_dims: HashMap<NodeId, usize>,
    edges: Vec<SheafEdge>,
}

impl CellularSheaf {
    pub fn add_vertex(&mut self, node: NodeId, dim: usize) { /* ... */ }
    pub fn add_edge(&mut self, src: NodeId, tgt: NodeId,
                    map_src: RestrictionMap, map_tgt: RestrictionMap) { /* ... */ }
    pub fn add_identity_edge(&mut self, src: NodeId, tgt: NodeId) { /* ... */ }
}
```

### The Coboundary Operator

The **coboundary operator** `delta` maps vertex sections (predictions at each node) to edge discrepancies (disagreements across each edge). For each edge `e` connecting vertices `v_i` and `v_j`:

```
(delta s)(e) = F_{e,tgt}(s(v_j)) - F_{e,src}(s(v_i))
```

Where `F_{e,src}` and `F_{e,tgt}` are the restriction maps. If all discrepancies are zero, the predictions are perfectly consistent. This operator is the discrete analogue of the exterior derivative in differential geometry -- it measures how local data fails to "glue" into a globally consistent section.

The roko implementation builds the full coboundary matrix (lines 261-294):

```rust
fn coboundary_matrix(&self) -> (Vec<f64>, usize, usize) {
    let nodes = self.ordered_nodes();
    let n_cols = self.total_vertex_dim();
    let n_rows: usize = self.edges.iter().map(|e| e.map_src.rows).sum();
    let mut matrix = vec![0.0; n_rows * n_cols];

    for edge in &self.edges {
        let edge_dim = edge.map_src.rows;
        let src_offset = self.stalk_offset(&nodes, edge.src);
        let tgt_offset = self.stalk_offset(&nodes, edge.tgt);

        for i in 0..edge_dim {
            // -F_{e,src}: negate source restriction
            for j in 0..src_dim {
                matrix[(row_offset + i) * n_cols + (src_offset + j)] -=
                    edge.map_src.data[i * edge.map_src.cols + j];
            }
            // +F_{e,tgt}: positive target restriction
            for j in 0..tgt_dim {
                matrix[(row_offset + i) * n_cols + (tgt_offset + j)] +=
                    edge.map_tgt.data[i * edge.map_tgt.cols + j];
            }
        }
    }

    (matrix, n_rows, n_cols)
}
```

### The Sheaf Laplacian

The **sheaf Laplacian** is `L_F = delta^T * delta`. It is a symmetric positive-semidefinite matrix whose properties reveal the consistency structure:

- `ker(L_F) = H^0(G, F)` = space of globally consistent sections
- `lambda_min(L_F) = 0` means a perfectly consistent global section exists
- `lambda_min(L_F) > 0` means no perfectly consistent section exists -- the predictions inherently disagree

From `docs/v1/20-technical-analysis/14-sheaf-tropical-geometry.md`:

> "The sheaf Laplacian generalizes the graph Laplacian by incorporating the restriction maps. Where the graph Laplacian diffuses scalar values, the sheaf Laplacian diffuses VECTOR values while preserving consistency structure."

The roko implementation (lines 301-317):

```rust
pub fn laplacian(&self) -> (Vec<f64>, usize) {
    let (delta, n_rows, n_cols) = self.coboundary_matrix();

    // L = delta^T * delta (n_cols x n_cols matrix)
    let mut lap = vec![0.0; n_cols * n_cols];
    for i in 0..n_cols {
        for j in 0..n_cols {
            let mut sum = 0.0;
            for k in 0..n_rows {
                sum += delta[k * n_cols + i] * delta[k * n_cols + j];
            }
            lap[i * n_cols + j] = sum;
        }
    }
    (lap, n_cols)
}
```

### Inconsistency Detection

The **inconsistency score** quantifies how much the predictions disagree:

```
score = ||delta s||^2 / ||s||^2
```

Where `s` is the flattened section (all predictions concatenated) and `delta s` is the coboundary. A score of 0 means perfect consistency; higher scores mean more disagreement. This ratio is scale-invariant -- multiplying all predictions by a constant does not change the score.

The roko implementation (lines 330-367):

```rust
pub fn inconsistency_score(&self, predictions: &HashMap<NodeId, Vec<f64>>) -> Option<f64> {
    // 1. Flatten predictions into a single vector
    let mut section = vec![0.0; n];
    for &node in &nodes {
        let pred = predictions.get(&node)?;
        // ... copy into section at correct offset ...
    }

    // 2. Compute ||s||^2
    let s_norm_sq: f64 = section.iter().map(|x| x * x).sum();

    // 3. Compute delta * s
    let (delta, n_rows, n_cols) = self.coboundary_matrix();
    let mut ds = vec![0.0; n_rows];
    // ... matrix-vector multiply ...

    // 4. Score = ||delta s||^2 / ||s||^2
    let ds_norm_sq: f64 = ds.iter().map(|x| x * x).sum();
    Some(ds_norm_sq / s_norm_sq)
}
```

### Identifying the Outlier Oracle

Beyond detecting inconsistency, the sheaf can identify *which* oracle is most responsible. The `most_inconsistent` function (lines 377-433) computes the per-vertex contribution to the total inconsistency via the Laplacian quadratic form:

```rust
pub fn most_inconsistent(
    &self,
    predictions: &HashMap<NodeId, Vec<f64>>,
) -> Option<(NodeId, f64)> {
    // 1. Compute L * s
    // 2. Per-vertex contribution: sum of (L*s)_i * s_i over stalk indices
    // 3. Return the vertex with highest contribution fraction
}
```

The test at line 675 demonstrates this:

```rust
// Oracle C is the outlier
let mut predictions = HashMap::new();
predictions.insert(0, vec![1.0, 1.0]);   // Oracle A
predictions.insert(1, vec![1.0, 1.0]);   // Oracle B
predictions.insert(2, vec![10.0, 10.0]); // Oracle C -- way off

let (node, fraction) = sheaf.most_inconsistent(&predictions).unwrap();
assert_eq!(node, 2);  // Oracle C identified as most inconsistent
```

### Spectral Analysis and Eigenvalues

The minimum eigenvalue of the Laplacian tells you whether *any* consistent section exists. The implementation (lines 464-529) uses the power method on a shifted matrix:

1. Find the largest eigenvalue `lambda_max` via power iteration on `L`
2. Form the shifted matrix `B = lambda_max * I - L`
3. Find the largest eigenvalue of `B` via power iteration
4. `lambda_min(L) = lambda_max(L) - lambda_max(B)`
5. Clamp to non-negative (the Laplacian is positive semidefinite)

For the typical sheaf dimensions in roko (4-12 dimensional), this converges in well under 200 iterations.

### Practical Application: Multi-Source Verification

When an AI agent gathers information from multiple sources (web search, file reading, LLM reasoning, memory recall), cellular sheaf cohomology can detect when these sources are globally inconsistent -- even when every pair of sources appears locally consistent. This is particularly valuable for detecting subtle contradictions in retrieved facts that propagate through transitive relationships.

---

## 3. Riemannian Geometry

**Source file**: `crates/roko-primitives/src/manifold.rs` (828 lines)
**roko task**: TA-06
**roko docs**: `docs/v1/20-technical-analysis/07-spectral-liquidity-manifolds.md`, `docs/v2-depth/21-roadmap/06-advanced-geometry-and-integration.md`

### Manifolds and the Cost Landscape

A **manifold** is a space that locally looks like ordinary flat Euclidean space but may be globally curved. The surface of the Earth is a 2D manifold embedded in 3D space: locally, your backyard looks flat, but globally the surface curves.

Roko models execution costs as a 4-dimensional manifold with axes:

```
Dimension 0: Slippage      (price impact of the action)
Dimension 1: Gas cost       (computational/transaction cost)
Dimension 2: Time           (latency)
Dimension 3: Opportunity    (cost of waiting / not acting)
```

Each point on this manifold represents a particular cost state. The manifold is *curved* because costs interact non-linearly -- doubling the trade size more than doubles the slippage, and gas costs spike non-linearly during congestion.

The mathematical framework follows do Carmo [8], with the statistical aspects of computing means on manifolds following Pennec [9] and Frechet [10].

From `docs/v1/20-technical-analysis/07-spectral-liquidity-manifolds.md`:

> "DeFi execution is not a simple price lookup. Every trade traverses a **liquidity landscape** where costs depend on pool depth, gas fees, timing, and opportunity costs. These costs vary non-linearly with trade size, time, and market conditions."

### The Metric Tensor

In ordinary Euclidean space, the distance between two nearby points is given by the Pythagorean theorem: `ds^2 = dx^2 + dy^2 + dz^2`. On a curved manifold, the distance formula generalizes to:

```
ds^2 = sum_{i,j} g_ij(x) * dx_i * dx_j
```

where `g_ij(x)` is the **metric tensor** -- a symmetric positive-definite matrix that can vary from point to point. The metric tensor tells you the "cost" of moving in each direction at each point.

For a 4D manifold, the metric tensor is a 4x4 matrix at each point. Diagonal entries control the cost of movement along each axis independently; off-diagonal entries create cross-coupling (moving in the slippage direction also incurs gas costs).

The roko implementation (lines 122-181) uses a callback-based metric:

```rust
pub struct MetricTensor {
    metric_fn: Box<dyn Fn(&Point) -> Mat4 + Send + Sync>,
}
```

The `execution_cost` metric (lines 158-166) defines a position-dependent cost surface:

```rust
pub fn execution_cost() -> Self {
    Self::new(|x: &Point| {
        let slippage = 1.0 + x[0] * x[0];    // Quadratic: slippage grows superlinearly
        let gas = 1.0 + x[1].abs();            // Linear: gas grows linearly
        let time = 1.0 + x[2].abs();           // Linear: time grows linearly
        let opportunity = 1.0 + x[3] * x[3];   // Quadratic: opportunity cost grows superlinearly
        mat4_diag(&[slippage, gas, time, opportunity])
    })
}
```

This means that at the origin `[0,0,0,0]`, all costs are 1.0 (equal weight). But at `[2,3,4,5]`, slippage costs 5x more than at the origin (because `1 + 4 = 5`), making the manifold curved.

The crate also provides hand-rolled 4x4 matrix utilities (lines 43-116): identity, diagonal, and Gauss-Jordan inverse. No external linear algebra dependency.

### Christoffel Symbols and Connections

**Christoffel symbols** `Gamma^k_ij` describe how the coordinate basis vectors change as you move along the manifold. They are the "connection coefficients" that appear in the geodesic equation. Conceptually, they encode: "if I am at position `x` and moving in direction `i`, how does the coordinate system twist in direction `k` due to the curvature?"

The formula is:

```
Gamma^k_ij = (1/2) * g^{kl} * (d_i g_{jl} + d_j g_{il} - d_l g_{ij})
```

where `g^{kl}` is the inverse metric and `d_i g_{jl}` is the partial derivative of the metric component `g_{jl}` with respect to coordinate `x_i`. This is a standard formula from Riemannian geometry (see do Carmo [8], Chapter 2). The three terms in parentheses encode how the metric changes in each coordinate direction; the `1/2` factor ensures the resulting connection is torsion-free (the Levi-Civita connection).

The roko implementation (lines 213-248) computes these via central finite differences:

```rust
pub fn christoffel(metric: &MetricTensor, point: &Point, h: f64)
    -> Option<ChristoffelSymbols>
{
    let g_inv = metric.inverse_at(point)?;

    // Compute partial derivatives: dg[i][j][l] = d(g_jl)/d(x_i)
    let mut dg = [[[0.0; DIM]; DIM]; DIM];
    for i in 0..DIM {
        let mut p_plus = *point;
        let mut p_minus = *point;
        p_plus[i] += h;
        p_minus[i] -= h;
        let g_plus = metric.at(&p_plus);
        let g_minus = metric.at(&p_minus);
        for j in 0..DIM {
            for l in 0..DIM {
                dg[i][j][l] = (g_plus[j][l] - g_minus[j][l]) / (2.0 * h);
            }
        }
    }

    // Gamma^k_ij = 1/2 g^{kl} (dg[i][j][l] + dg[j][i][l] - dg[l][i][j])
    let mut gamma = [[[0.0; DIM]; DIM]; DIM];
    for k in 0..DIM {
        for i in 0..DIM {
            for j in 0..DIM {
                let mut sum = 0.0;
                for l in 0..DIM {
                    sum += g_inv[k][l] * (dg[i][j][l] + dg[j][i][l] - dg[l][i][j]);
                }
                gamma[k][i][j] = 0.5 * sum;
            }
        }
    }

    Some(gamma)
}
```

**Key properties verified by tests**:
- On a flat (constant) metric, all Christoffel symbols are zero.
- On the `execution_cost` metric, the slippage axis has non-zero `Gamma^0_00` because the metric varies quadratically with `x_0`.
- Christoffel symbols are symmetric in the lower indices: `Gamma^k_ij = Gamma^k_ji` (torsion-free connection).

### Geodesics and the RK4 Solver

A **geodesic** is the shortest path between two points on a curved manifold -- the analogue of a straight line in flat space. On the Earth's surface, geodesics are great circles.

The geodesic equation is a system of second-order ODEs:

```
d^2 x^k / dt^2 + Gamma^k_ij * (dx^i/dt) * (dx^j/dt) = 0
```

This is the Euler-Lagrange equation for the length functional on the manifold. The roko solver converts it to a first-order system (`dx/dt = v`, `dv^k/dt = -Gamma^k_ij v^i v^j`) and integrates with 4th-order Runge-Kutta (lines 284-322):

```rust
pub fn geodesic_rk4(
    metric: &MetricTensor,
    start: Point,
    velocity: Point,
    steps: usize,
    dt: f64,
    h: f64,
) -> Vec<GeodesicPoint> {
    let mut path = Vec::with_capacity(steps + 1);
    let mut x = start;
    let mut v = velocity;

    for _ in 0..steps {
        // Standard RK4: compute k1, k2, k3, k4
        let (k1x, k1v) = geodesic_deriv(metric, &x, &v, h);
        let (x2, v2) = step_state(&x, &v, &k1x, &k1v, 0.5 * dt);
        let (k2x, k2v) = geodesic_deriv(metric, &x2, &v2, h);
        let (x3, v3) = step_state(&x, &v, &k2x, &k2v, 0.5 * dt);
        let (k3x, k3v) = geodesic_deriv(metric, &x3, &v3, h);
        let (x4, v4) = step_state(&x, &v, &k3x, &k3v, dt);
        let (k4x, k4v) = geodesic_deriv(metric, &x4, &v4, h);

        // Weighted average of slopes
        for i in 0..DIM {
            x[i] += dt / 6.0 * (k1x[i] + 2.0 * k2x[i] + 2.0 * k3x[i] + k4x[i]);
            v[i] += dt / 6.0 * (k1v[i] + 2.0 * k2v[i] + 2.0 * k3v[i] + k4v[i]);
        }

        path.push(GeodesicPoint { position: x, velocity: v });
    }
    path
}
```

**On a flat metric**, the geodesic is a straight line (velocity stays constant). **On a curved metric**, the velocity changes -- the path bends to follow the curvature. The tests verify both behaviors.

**Application**: The geodesic from current system state to desired state is the *optimal* (minimum-cost) transition path. Instead of naively moving in a straight line through cost space, the geodesic curves to avoid high-cost regions.

### Geodesic Distance

The `approx_geodesic_distance` function (lines 371-407) approximates the geodesic distance by integrating the metric along a straight-line path:

```rust
pub fn approx_geodesic_distance(
    metric: &MetricTensor, a: &Point, b: &Point, segments: usize,
) -> f64 {
    // For each segment along the straight line from a to b:
    //   1. Evaluate the metric at the midpoint of the segment
    //   2. Compute ds^2 = g_ij * tangent_i * tangent_j
    //   3. Accumulate sqrt(ds^2) * segment_length
}
```

This is exact for flat metrics and an upper bound for curved metrics (since the straight line is never shorter than the geodesic). With `segments=100`, the approximation is quite accurate for most practical cases.

### Ricci Curvature

The **Ricci scalar** `R` is a single number that summarizes the curvature at a point. It is obtained by a double contraction of the Riemann curvature tensor:

- `R > 0`: Locally sphere-like. Geodesics converge. In a cost context: market self-corrects.
- `R < 0`: Locally saddle-like. Geodesics diverge. Perturbations amplify.
- `R = 0`: Flat. Linear cost model. Geodesics are straight lines.

The computation (lines 424-473) follows the standard differential geometry procedure:

1. Compute Christoffel symbols at the point
2. Compute derivatives of Christoffel symbols (via finite differences of the Christoffel computation itself -- requiring 8 additional metric evaluations per dimension)
3. Build the Riemann curvature tensor:
   ```
   R^l_ijk = d_i Gamma^l_jk - d_j Gamma^l_ik + Gamma^l_im Gamma^m_jk - Gamma^l_jm Gamma^m_ik
   ```
4. Contract to the Ricci tensor: `R_ij = R^k_ikj`
5. Contract with the inverse metric: `R = g^{ij} R_{ij}`

### Frechet Mean

The **Frechet mean** [10] is the intrinsic average on a curved manifold. It minimizes `sum_i d^2(x, x_i)` where `d` is geodesic distance. On flat space, the Frechet mean equals the arithmetic mean. On curved space, it differs -- it accounts for the curvature when computing "center."

The roko implementation (lines 494-556) uses iterative gradient descent initialized with the Euclidean mean:

```rust
pub fn frechet_mean(
    metric: &MetricTensor, points: &[Point], max_iter: usize, tol: f64,
) -> (Point, usize) {
    // Initialize with Euclidean mean
    // Iteratively:
    //   1. Compute tangent vector (average of log_mean(p_i))
    //   2. Apply inverse metric to get Riemannian gradient
    //   3. Step along gradient with shrinking step size: 1 / (1 + iter * 0.1)
    //   4. Check convergence (Euclidean distance between iterates < tol)
}
```

The step size shrinks as `1 / (1 + iter * 0.1)` to ensure convergence. This follows the general approach of Pennec [9] for computing intrinsic statistics on Riemannian manifolds.

### Practical Application: Optimal Configuration Transitions

An AI agent managing model configurations (temperature, max tokens, context window, etc.) faces a cost landscape where changing parameters has non-linear costs. The geodesic on the configuration manifold gives the minimum-disruption transition path from current settings to desired settings, accounting for the fact that some parameter changes interact non-linearly.

---

## 4. Tropical Algebra (Max-Plus Semiring)

**Source file**: `crates/roko-primitives/src/tropical.rs` (698 lines)
**roko task**: TA-14
**roko docs**: `docs/v1/20-technical-analysis/14-sheaf-tropical-geometry.md`

### What Is a Semiring

A **semiring** is an algebraic structure with two operations (analogous to addition and multiplication) that satisfy the standard distributive law, but unlike a ring, there is no requirement for additive inverses (subtraction is not always possible).

The familiar integers with `(+, *)` form a ring. Tropical algebra replaces these operations to create a different algebraic structure that has surprising connections to optimization, neural networks, and graph algorithms [11].

### The Max-Plus Semiring

In the **tropical semiring** (also called the max-plus algebra):

```
Tropical addition:       a (+) b = max(a, b)
Tropical multiplication: a (*) b = a + b         (ordinary addition!)
Tropical zero:           -infinity               (max(x, -inf) = x for all x)
Tropical one:            0                        (x + 0 = x for all x)
```

This is not arbitrary -- it captures the essential structure of optimization problems. When you compute `max(cost_A + bonus_A, cost_B + bonus_B)`, you are performing a tropical inner product.

**Semiring laws hold** (all verified by tests in the crate):
- Associativity: `max(max(a,b),c) = max(a,max(b,c))` and `(a+b)+c = a+(b+c)`
- Commutativity: `max(a,b) = max(b,a)` and `a+b = b+a`
- Distributivity: `a + max(b,c) = max(a+b, a+c)`
- Identities: `max(a, -inf) = a` and `a + 0 = a`
- Zero absorbs: `-inf + a = -inf` (tropical zero is absorbing under multiplication)

The roko implementation (lines 38-99) uses Rust's operator overloading:

```rust
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct TropicalF64(pub f64);

impl TropicalF64 {
    pub const ZERO: Self = Self(f64::NEG_INFINITY);  // additive identity
    pub const ONE: Self = Self(0.0);                  // multiplicative identity
}

impl Add for TropicalF64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0.max(rhs.0))  // tropical addition = max
    }
}

impl Mul for TropicalF64 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Self::ZERO;  // -inf + anything = -inf
        }
        Self(self.0 + rhs.0)    // tropical multiplication = standard addition
    }
}
```

**Tropical exponentiation**: `a^n = n * a` in standard arithmetic (since tropical multiplication is standard addition, repeated `n` times is just multiplication by `n`). The `trop_pow` method implements this (lines 71-77).

### Tropical Polynomials

A **tropical polynomial** is a max over affine functions:

```
p(x) = max_i (c_i + a_{i,1} * x_1 + a_{i,2} * x_2 + ...)
```

Each term is an affine function `c_i + a_i . x`. The polynomial evaluates to the maximum over all terms. This is exactly what a ReLU neural network computes at each layer -- making tropical algebra the natural language for analyzing neural networks [12].

**Worked example** (from the test at line 503):

```
p(x) = max(2 + 1*x, 5 + (-1)*x)

Two affine functions:
  f1(x) = 2 + x     (slope +1, intercept 2)
  f2(x) = 5 - x     (slope -1, intercept 5)

At x = 0:   max(2, 5) = 5     <- f2 wins
At x = 1.5: max(3.5, 3.5) = 3.5  <- boundary (both tie)
At x = 4:   max(6, 1) = 6     <- f1 wins
```

The **tropical hypersurface** -- where two or more terms tie for the maximum -- is the decision boundary. At `x = 1.5`, both terms give 3.5, so this is the boundary between the "f1 region" and the "f2 region."

The roko implementation (`tropical.rs`, lines 144-221):

```rust
pub struct TropicalPolynomial {
    pub terms: Vec<TropicalTerm>,
}

impl TropicalPolynomial {
    pub fn evaluate(&self, point: &[TropicalF64]) -> TropicalF64 {
        let mut result = TropicalF64::ZERO;  // -infinity
        for term in &self.terms {
            let mut term_value = term.coefficient;
            for (exp, x) in term.exponents.iter().zip(point.iter()) {
                term_value = term_value * x.trop_pow(*exp);
            }
            result = result + term_value;  // tropical addition = max
        }
        result
    }

    /// Which term achieves the maximum (which "linear piece" is active)
    pub fn active_term(&self, point: &[TropicalF64]) -> Option<usize> { /* ... */ }
}
```

### Tropical Matrices

Tropical matrix multiplication replaces the standard `sum(A_ik * B_kj)` with `max_k(A_ik + B_kj)`:

```
Standard: C_ij = sum_k (A_ik * B_kj)
Tropical: C_ij = max_k (A_ik + B_kj)
```

**Worked example** (from the test at line 574):

```
A = [[1, 3],    B = [[5, 6],
     [2, 4]]         [7, 8]]

C_00 = max(1+5, 3+7) = max(6, 10) = 10
C_01 = max(1+6, 3+8) = max(7, 11) = 11
C_10 = max(2+5, 4+7) = max(7, 11) = 11
C_11 = max(2+6, 4+8) = max(8, 12) = 12
```

The tropical identity matrix has 0 on the diagonal and `-inf` elsewhere (since `max(x + 0, y + (-inf)) = x` for the diagonal element).

The roko implementation (`tropical.rs`, lines 227-321) provides `TropicalMatrix` with `mul`, `mul_vec`, and utility constructors.

**Connection to shortest paths**: Tropical matrix multiplication corresponds to the shortest-path relaxation step in the Bellman-Ford and Floyd-Warshall algorithms. Raising a tropical adjacency matrix to the n-th power gives the shortest paths of length at most n.

### Tropical Attention

Standard softmax attention computes `softmax(QK^T / sqrt(d)) V`. The tropical limit (as temperature goes to zero) replaces softmax with hardmax:

```
tropical_attention(q, keys, values) = max_j(q . k_j + v_j)
```

This selects the single key with the highest dot product (plus value bias) rather than blending all keys. The result is piecewise-linear and exactly interpretable.

From `docs/v1/20-technical-analysis/14-sheaf-tropical-geometry.md`:

> "Tropical attention directly approximates dynamic programming algorithms (shortest paths, Viterbi, CKY parsing). This creates a principled bridge between symbolic planning and neural scoring."

The roko implementation (`tropical.rs`, lines 341-360):

```rust
pub fn tropical_attention(q: &[f64], keys: &[Vec<f64>], values: &[f64])
    -> (f64, usize)
{
    let mut best_val = f64::NEG_INFINITY;
    let mut best_idx = 0;

    for (j, key) in keys.iter().enumerate() {
        let dot: f64 = q.iter().zip(key.iter()).map(|(a, b)| a * b).sum();
        let score = dot + values[j];
        if score > best_val {
            best_val = score;
            best_idx = j;
        }
    }
    (best_val, best_idx)
}
```

A batch version (`tropical_attention_batch`, lines 368-377) processes multiple queries against the same keys.

### Adversarial Distance

The **adversarial distance** for a tropical polynomial measures how much you need to perturb the input to cross a decision boundary (flip which term is active). It equals half the gap between the two highest-scoring terms:

```
adversarial_distance = (best_score - second_best_score) / 2
```

At the decision boundary, this is exactly zero. Far from the boundary, it is large. This is a direct consequence of the piecewise-linear structure: the minimum L-infinity perturbation needed to change the active term is exactly half the score gap.

The roko implementation (`tropical.rs`, lines 387-416):

```rust
pub fn adversarial_distance(poly: &TropicalPolynomial, point: &[TropicalF64])
    -> Option<f64>
{
    // 1. Evaluate each term at the point
    let mut scores: Vec<f64> = poly.terms.iter().map(|term| { /* ... */ }).collect();
    // 2. Sort scores descending
    scores.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    // 3. Return (scores[0] - scores[1]) / 2
    if scores.len() >= 2 && scores[0].is_finite() && scores[1].is_finite() {
        Some((scores[0] - scores[1]) / 2.0)
    } else {
        None
    }
}
```

From `docs/v1/20-technical-analysis/14-sheaf-tropical-geometry.md`:

> "Zhang et al. (2018) and subsequent work show that: 1) Decision boundaries of ReLU-based oracles are tropical hypersurfaces, 2) Adversarial examples live on or near these hypersurfaces, 3) The NUMBER of linear regions correlates with adversarial robustness."

### Practical Application: Decision Boundary Analysis

An AI agent's routing logic (which model to use, which tool to invoke) is often piecewise-linear -- a series of comparisons and score aggregations. Modeling this routing as a tropical polynomial makes the decision boundaries explicit. The adversarial distance at the current input tells you how robust the current routing decision is: if the distance is small, a slight perturbation could flip the decision, warranting higher confidence thresholds.

---

## 5. Robust Statistics

**Source file**: `crates/roko-primitives/src/robust_stats.rs` (169 lines)
**roko task**: TA-10

### Why Standard Statistics Fail

The arithmetic mean is not robust. A single outlier can move it arbitrarily:

```
Clean data:      [1, 2, 3, 4, 5]       mean = 3.0
One outlier:     [1, 2, 3, 4, 10000]    mean = 2002.0
```

Standard deviation is equally fragile. These are "breakdown point 0" estimators -- a single corrupted observation can make them arbitrarily wrong.

The **breakdown point** of an estimator is the fraction of observations that can be arbitrarily corrupted before the estimator becomes unbounded. Higher breakdown point = more robust. This concept was formalized by Hampel [14] and extended in the landmark work of Huber [13], who established the foundational theory of M-estimators.

### Trimmed Mean

**What it is and why it matters**: The trimmed mean provides a compromise between the efficiency of the arithmetic mean (optimal for clean Gaussian data) and the robustness of the median (50% breakdown point but statistically inefficient). By discarding a controlled fraction of extremes, you get a location estimator that is both reasonably efficient and provably robust.

**Formula**: Given sorted observations `x_(1) <= x_(2) <= ... <= x_(n)` and trim fraction `alpha`:
```
trimmed_mean = (1 / (n - 2*floor(n*alpha))) * sum_{i=floor(n*alpha)+1}^{n-floor(n*alpha)} x_(i)
```

**Breakdown point**: Equal to `alpha`. With `alpha = 0.1`, up to 10% of observations can be corrupted.

**The roko implementation** (`robust_stats.rs`, lines 18-36):

```rust
pub fn trimmed_mean(values: &[f64], trim_pct: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let trim_pct = trim_pct.clamp(0.0, 0.499);
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted.len();
    let trim_count = (n as f64 * trim_pct).floor() as usize;
    let lo = trim_count;
    let hi = n.saturating_sub(trim_count);
    if lo >= hi {
        return None;
    }
    let trimmed = &sorted[lo..hi];
    let sum: f64 = trimmed.iter().sum();
    Some(sum / trimmed.len() as f64)
}
```

**Worked example** (from test at line 107):

```
values = [-1000.0, 2.0, 3.0, 4.0, 1000.0]
trim_pct = 0.2

Sorted: [-1000, 2, 3, 4, 1000]
trim_count = floor(5 * 0.2) = 1
Remaining: [2, 3, 4]
Mean: 3.0   (outliers at both ends removed)
```

### Median Absolute Deviation (MAD)

**What it is and why it matters**: Standard deviation measures dispersion but is as fragile as the mean. MAD replaces both the mean and the standard deviation with their median-based counterparts, achieving the maximum possible breakdown point of 50%.

**Formula**:
```
MAD = 1.4826 * median(|x_i - median(x)|)
```

1. Compute the median of the data
2. Compute absolute deviations from the median
3. Take the median of those deviations
4. Multiply by 1.4826 (the consistency factor `1 / Phi^{-1}(3/4)` that makes MAD an unbiased estimator of the standard deviation for normally distributed data)

**Breakdown point**: 50% -- the best possible. Up to half the data can be corrupted.

**The roko implementation** (`robust_stats.rs`, lines 46-54):

```rust
pub fn mad(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let med = median(values)?;
    let abs_devs: Vec<f64> = values.iter().map(|x| (x - med).abs()).collect();
    let med_dev = median(&abs_devs)?;
    Some(med_dev * 1.4826)
}
```

**Worked example** (from test at line 121):

```
values = [1, 2, 3, 4, 5]
median = 3
abs_devs = [2, 1, 0, 1, 2]
median(abs_devs) = 1
MAD = 1 * 1.4826 = 1.4826

Compare: standard deviation of [1,2,3,4,5] = sqrt(2) ~ 1.414
The MAD is close but more robust to contamination.
```

### Hodges-Lehmann Estimator

**What it is and why it matters**: The Hodges-Lehmann estimator [15] is the median of all pairwise averages. It achieves a higher statistical efficiency than the median for symmetric distributions while maintaining strong robustness (29.3% breakdown point). For symmetric distributions, it is approximately 96% as efficient as the arithmetic mean -- nearly as good in clean data, but much better in contaminated data.

**Formula**: For observations `x_1, ..., x_n`:
```
HL = median{ (x_i + x_j) / 2 : 1 <= i <= j <= n }
```

**Breakdown point**: 29.3%.

**Computational complexity**: `O(n^2)` to generate all `n*(n+1)/2` pairwise averages, then `O(n^2 log n)` to find their median.

**The roko implementation** (`robust_stats.rs`, lines 62-74):

```rust
pub fn hodges_lehmann(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let n = values.len();
    let mut pairwise: Vec<f64> = Vec::with_capacity(n * (n + 1) / 2);
    for i in 0..n {
        for j in i..n {
            pairwise.push((values[i] + values[j]) / 2.0);
        }
    }
    median(&pairwise)
}
```

**Worked example** (from test at line 149):

```
values = [1, 2, 3, 4, 1000]  (one outlier)

All pairwise averages (including self-pairs):
  (1+1)/2=0.5,  (1+2)/2=1.5,  (1+3)/2=2,  (1+4)/2=2.5,  (1+1000)/2=500.5,
  (2+2)/2=2,    (2+3)/2=2.5,  (2+4)/2=3,  (2+1000)/2=501,
  (3+3)/2=3,    (3+4)/2=3.5,  (3+1000)/2=501.5,
  (4+4)/2=4,    (4+1000)/2=502,
  (1000+1000)/2=1000

Sorted: [0.5, 1.5, 2, 2, 2.5, 2.5, 3, 3, 3.5, 4, 500.5, 501, 501.5, 502, 1000]
Median (8th of 15): 3.0

The outlier barely affects the result. Compare with arithmetic mean: 202.0
```

### Summary of Breakdown Points

| Estimator | Breakdown Point | Complexity | Best For |
|---|---|---|---|
| Arithmetic Mean | 0% | O(n) | Clean data only |
| Trimmed Mean (10%) | 10% | O(n log n) | Light contamination |
| Median | 50% | O(n) | Heavy contamination (location) |
| MAD | 50% | O(n log n) | Heavy contamination (scale) |
| Hodges-Lehmann | 29.3% | O(n^2 log n) | Moderate contamination with higher efficiency |

### Practical Application: Reliable Metric Aggregation

Every numeric metric in an AI agent system -- response latency, token usage, relevance scores, cost estimates -- is subject to noise and occasional extreme values (network timeouts, malformed inputs, model hallucinations). Using `trimmed_mean` instead of `mean` for these aggregations provides resilience against individual outliers without discarding too much data. Using `mad` instead of `std_dev` ensures that the computed spread reflects the typical behavior, not the worst case.

---

## 6. Additional Primitives: HDC and Codebooks

The crate also contains primitives that are documented in the crate's own README but are worth mentioning for completeness.

### HdcVector (`hdc.rs`, 718 lines)

A 10,240-bit binary vector (`[u64; 160]`, 1,280 bytes, `Copy`, stack-allocated) for hyperdimensional computing. Three core operations:

- **Bind** (XOR): Associates two concepts. Involutory: `bind(bind(a, b), b) = a`.
- **Bundle** (majority vote): Superposition of multiple concepts.
- **Similarity** (normalized Hamming distance): `[0.0, 1.0]`, where random vectors score ~0.5 and identical score 1.0.

Performance: `~5 ns` for bind (160 XORs), `~50 ns` for similarity (XOR + hardware POPCNT).

### Codebook and PatternStore (`codebook.rs`, 544 lines)

Deterministic symbol allocation (`Codebook`), role-filler binding (`role_bind`/`unbind`), and pattern matching (`PatternStore`) built on `HdcVector`. Cross-domain resonance detection identifies when patterns from different domains share structural similarity beyond the chance threshold of `0.526` (approximately 3 standard deviations above random for 10,240-bit vectors).

### PadVector (`pad.rs`, 130 lines)

A Pleasure-Arousal-Dominance vector for affect modeling. Three `f64` dimensions in `[-1.0, 1.0]` with clamping, decay, delta application, and cosine similarity.

### InferenceTier (`tier.rs`, 152 lines)

Three-tier model routing: T0 (suppress, no LLM call), T1 (Haiku-class), T2 (Opus or Sonnet based on vitality threshold at 0.3). Pure stateless function.

---

## IronClaw Integration Plan

### Tier 1: Immediate Value -- Robust Statistics

**Where**: `src/estimation/`, `src/evaluation/`, `src/workspace/`
**Effort**: ~100-200 lines
**Impact**: High -- directly improves quality of all numeric aggregation
**Risk**: Zero -- pure math, no side effects, no async

Replace every `mean()` and `std_dev()` computation over LLM output metrics with robust alternatives:

```rust
// Before: one slow LLM response skews the average
let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;

// After: outliers are handled gracefully
use roko_primitives::robust_stats::{trimmed_mean, mad};

let avg_latency = trimmed_mean(&latencies, 0.1)
    .unwrap_or(0.0);
let spread = mad(&latencies)
    .unwrap_or(0.0);

// Use MAD for anomaly detection: flag values > median + 3 * MAD
let median_val = median(&latencies).unwrap_or(0.0);
let threshold = median_val + 3.0 * spread;
let anomalous = latencies.iter().any(|&x| x > threshold);
```

**Specific integration points**:
- `src/estimation/`: Cost/time/value estimation with EMA learning. Replace raw EMA inputs with trimmed means to prevent outlier spikes from corrupting the exponential moving average.
- `src/evaluation/`: Success evaluation. Use MAD to detect when a metric has genuinely changed vs. when it is just noisy. A shift greater than 3 MAD is statistically significant.
- `src/workspace/`: Memory search scoring. Use Hodges-Lehmann when aggregating relevance scores from multiple embedding providers or search backends.

**Implementation sketch for `src/estimation/`**:

```rust
// In the estimation module, wrap the EMA update:
pub fn robust_ema_update(
    current_ema: f64,
    recent_observations: &[f64],
    alpha: f64,
    trim_pct: f64,
) -> f64 {
    // Use trimmed mean of recent observations instead of raw last value
    let robust_input = trimmed_mean(recent_observations, trim_pct)
        .unwrap_or(current_ema);
    current_ema * (1.0 - alpha) + robust_input * alpha
}
```

### Tier 2: Interesting -- TDA for Execution Traces

**Where**: `src/observability/`, `src/agent/`
**Effort**: ~500-800 lines
**Impact**: Medium -- detects agent behavior patterns that simple metrics miss

The agent loop generates a time series of execution characteristics (latency per turn, tool call count per step, token usage per response). Applying the TDA pipeline to these traces can detect:

1. **Loop detection**: The agent stuck in a retry cycle produces a time series with strong H1 features (loops in phase space). Detect `max_persistence()` above a threshold on H1 features.

2. **Convergence detection**: An agent approaching a solution shows decreasing H0 feature lifetimes (the point cloud in phase space is tightening).

3. **Regime change detection**: When the bottleneck distance between successive persistence diagrams spikes, the system dynamics have fundamentally changed.

```rust
use roko_primitives::tda::{takens_embedding, vietoris_rips, persistence_landscape};

/// Analyze recent agent turns for behavioral anomalies.
pub fn detect_agent_loop(turn_latencies: &[f64]) -> AgentBehavior {
    if turn_latencies.len() < 20 {
        return AgentBehavior::InsufficientData;
    }

    // 1. Embed the 1D latency series into 2D phase space
    let embedded = takens_embedding(turn_latencies, 2, 3);
    if embedded.is_empty() {
        return AgentBehavior::InsufficientData;
    }

    // 2. Compute persistence diagram
    let diagram = vietoris_rips(&embedded, 1);

    // 3. Check for loops (H1 features with high persistence)
    let h1_features = diagram.points_at_dim(1);
    let max_h1_persistence = h1_features.iter()
        .map(|p| p.persistence())
        .fold(0.0_f64, f64::max);

    if max_h1_persistence > LOOP_THRESHOLD {
        return AgentBehavior::PossibleLoop {
            loop_strength: max_h1_persistence,
        };
    }

    // 4. Check convergence via shrinking H0 spread
    let h0_features = diagram.points_at_dim(0);
    let avg_h0_persistence = h0_features.iter()
        .map(|p| p.persistence())
        .sum::<f64>() / h0_features.len().max(1) as f64;

    if avg_h0_persistence < CONVERGENCE_THRESHOLD {
        return AgentBehavior::Converging;
    }

    AgentBehavior::Normal
}
```

### Tier 3: Research-Grade -- Sheaf Consistency for Multi-Source Verification

**Where**: Multi-source information verification in workspace memory
**Effort**: ~1,000+ lines for integration
**Impact**: Low frequency but high value -- catches contradictions that pairwise comparisons miss

When the workspace accumulates knowledge from different tools (web search, file reading, LLM reasoning), sheaf cohomology could detect when these sources are globally inconsistent -- even when every pair of sources appears locally consistent.

```rust
use roko_primitives::sheaf::{CellularSheaf, RestrictionMap};
use std::collections::HashMap;

/// Check consistency of information from multiple sources.
/// Each source provides a vector of confidence-weighted fact assertions.
pub fn check_source_consistency(
    sources: &[(SourceId, Vec<f64>)],
) -> ConsistencyResult {
    let dim = sources[0].1.len();
    let mut sheaf = CellularSheaf::new();

    // Add each source as a vertex with the assertion dimension
    for (i, (_, _)) in sources.iter().enumerate() {
        sheaf.add_vertex(i as u32, dim);
    }

    // Connect all pairs with identity edges (same assertion space)
    for i in 0..sources.len() {
        for j in (i + 1)..sources.len() {
            sheaf.add_identity_edge(i as u32, j as u32);
        }
    }

    // Build prediction map
    let mut predictions = HashMap::new();
    for (i, (_, facts)) in sources.iter().enumerate() {
        predictions.insert(i as u32, facts.clone());
    }

    // Check global consistency
    let score = sheaf.inconsistency_score(&predictions);
    match score {
        Some(s) if s > 0.1 => {
            let (outlier, fraction) = sheaf.most_inconsistent(&predictions)
                .unwrap_or((0, 0.0));
            ConsistencyResult::Inconsistent {
                score: s,
                outlier_source: sources[outlier as usize].0,
                outlier_fraction: fraction,
            }
        }
        _ => ConsistencyResult::Consistent,
    }
}
```

### Tier 4: Long-Term Research -- Riemannian Cost Optimization and Tropical Decision Analysis

These are research-grade applications documented extensively in the roko docs:

- **Riemannian geometry** for finding optimal model configuration transitions (the geodesic from current performance to target performance on the cost manifold). This requires defining a meaningful metric tensor over IronClaw's configuration space -- potentially using observed performance data to learn the local cost structure.

- **Tropical algebra** for analyzing the decision boundaries of the agent's routing logic (which model to use, which tool to invoke) and computing exact adversarial distances. This is most useful when the routing logic is expressible as a piecewise-linear function, which is the case for score-based tool selection.

---

## Complexity Assessment

| Component | Lines of Code | Integration Effort | Value | Risk |
|---|---|---|---|---|
| Robust statistics | ~170 | Low | High | Zero (pure math, no side effects) |
| TDA pipeline | ~575 | Moderate | Medium | Low (pure math, O(n^2) for Rips) |
| Cellular sheaves | ~789 | Moderate-High | Niche but high | Low (pure math, eigenvalue convergence) |
| Riemannian geometry | ~828 | High | Research-grade | Medium (numerical stability of RK4, finite differences) |
| Tropical algebra | ~698 | Medium | Research-grade | Low (exact arithmetic) |
| HDC + Codebook | ~718 + ~544 | Already integrated in roko | Core infrastructure | Zero |
| **Total crate** | **~4,682** | -- | -- | -- |

**Dependencies**: `serde`, `serde_json`, `uuid`, optionally `rkyv`. Pure `#![deny(unsafe_code)]` Rust. No external linear algebra or numerical libraries. Dev dependencies include `proptest` for property-based testing and `criterion` for benchmarks.

---

## References

[1] Carlsson, G. (2009). Topology and data. *Bulletin of the American Mathematical Society*, 46(2), 255--308. The foundational survey that established TDA as a field, demonstrating how persistent homology extracts shape features from data.

[2] Takens, F. (1981). Detecting strange attractors in turbulence. In Rand, D. & Young, L.-S. (Eds.), *Dynamical Systems and Turbulence, Warwick 1980* (Lecture Notes in Mathematics, Vol. 898, pp. 366--381). Springer. Proves that delay embeddings reconstruct attractor topology from scalar time series.

[3] Edelsbrunner, H. & Harer, J. (2010). *Computational Topology: An Introduction*. American Mathematical Society. The standard textbook on persistent homology and its algorithmic foundations.

[4] Bubenik, P. (2015). Statistical topological data analysis using persistence landscapes. *Journal of Machine Learning Research*, 16(3), 77--102. Introduces persistence landscapes as a vectorization of persistence diagrams amenable to statistical analysis.

[5] Hansen, J. & Ghrist, R. (2019). Toward a spectral theory of cellular sheaves. *Journal of Applied and Computational Topology*, 3, 315--358. Develops the sheaf Laplacian and its spectral properties for consistency analysis on networks.

[6] Curry, J. (2014). *Sheaves, Cosheaves and Applications*. Ph.D. dissertation, University of Pennsylvania. Advisor: Robert W. Ghrist. Develops the computational theory of cellular sheaves for topological data analysis, sensor networks, and signal processing.

[7] Robinson, M. (2014). *Topological Signal Processing*. Springer (Mathematical Engineering series). Applies sheaf theory to signal processing, establishing sheaves as a canonical data structure for sensor integration.

[8] do Carmo, M. P. (1992). *Riemannian Geometry*. Birkhauser (Mathematics: Theory & Applications). Translated by Francis Flaherty. The standard graduate textbook on Riemannian geometry covering metric tensors, connections, geodesics, and curvature.

[9] Pennec, X. (2006). Intrinsic statistics on Riemannian manifolds: Basic tools for geometric measurements. *Journal of Mathematical Imaging and Vision*, 25(1), 127--154. Develops the Frechet mean, Riemannian covariance, and other statistical tools on manifolds.

[10] Frechet, M. (1948). Les elements aleatoires de nature quelconque dans un espace distancie. *Annales de l'Institut Henri Poincare*, 10, 215--310. The original definition of the intrinsic mean in general metric spaces as the minimizer of expected squared distance.

[11] Maclagan, D. & Sturmfels, B. (2015). *Introduction to Tropical Geometry* (Graduate Studies in Mathematics, Vol. 161). American Mathematical Society. The standard graduate textbook on tropical geometry covering tropical varieties, polyhedral structures, and connections to algebraic geometry.

[12] Zhang, L., Naitzat, G. & Lim, L.-H. (2018). Tropical geometry of deep neural networks. In *Proceedings of the 35th International Conference on Machine Learning* (ICML 2018), PMLR 80, 5824--5832. Proves that feedforward ReLU networks compute tropical rational functions, connecting decision boundaries to tropical hypersurfaces.

[13] Huber, P. J. (1964). Robust estimation of a location parameter. *The Annals of Mathematical Statistics*, 35(1), 73--101. The foundational work on M-estimators and minimax robust estimation.

[14] Hampel, F. R. (1974). The influence curve and its role in robust estimation. *Journal of the American Statistical Association*, 69(346), 383--393. Introduces the influence function as a tool for measuring the local robustness of statistical estimators.

[15] Hodges, J. L. & Lehmann, E. L. (1963). Estimates of location based on rank tests. *The Annals of Mathematical Statistics*, 34(2), 598--611. Introduces the median of pairwise averages as a robust, highly efficient location estimator.

[16] Alfarra, M., Bibi, A., Torber, H., Hein, M., & Ghanem, B. (2022). On the decision boundaries of neural networks: A tropical geometry perspective. *IEEE Transactions on Pattern Analysis and Machine Intelligence*, 44(12), 9072--9085. Characterizes neural network decision boundaries as subsets of tropical hypersurfaces formed by zonotope convex hulls.

[17] Absil, P.-A., Mahony, R. & Sepulchre, R. (2008). *Optimization Algorithms on Matrix Manifolds*. Princeton University Press. Extends standard optimization (steepest descent, conjugate gradient, trust-region) to Riemannian manifolds, applicable to constrained optimization on positive-definite matrices and other matrix spaces.
