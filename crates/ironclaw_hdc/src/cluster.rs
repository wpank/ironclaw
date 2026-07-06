//! K-medoids (PAM) clustering over HDC vectors using Hamming distance.
//!
//! Provides deterministic clustering of labeled hypervectors into k groups,
//! using farthest-first initialization and the PAM (Partitioning Around Medoids)
//! swap procedure. All operations are deterministic (no RNG).
//!
//! # Example
//!
//! ```
//! use ironclaw_hdc::cluster::{k_medoids, Cluster};
//! use ironclaw_hdc::vector::HdcVector;
//!
//! let vectors: Vec<(usize, HdcVector)> = (0..10)
//!     .map(|i| (i, HdcVector::from_seed("demo", format!("item_{i}").as_bytes())))
//!     .collect();
//!
//! let clusters = k_medoids(&vectors, 3).unwrap();
//! assert_eq!(clusters.len(), 3);
//! ```

use crate::error::HdcError;
use crate::vector::HdcVector;

/// Maximum number of assignment/update iterations before stopping.
const MAX_ITERATIONS: usize = 100;

/// Result of k-medoids clustering.
#[derive(Debug, Clone)]
pub struct Cluster<Id> {
    /// The ID of the medoid (representative element).
    pub medoid_id: Id,
    /// IDs of all members including the medoid.
    pub members: Vec<Id>,
    /// Average distance of non-medoid members to the medoid (0.0 = perfect, 0.5 = random).
    /// Excludes the medoid's self-distance (always 0) to avoid under-reporting.
    pub inertia: f64,
}

/// Compute distance between two HDC vectors: `1.0 - similarity`.
///
/// Returns 0.0 for identical vectors and ~0.5 for random independent vectors.
fn distance(a: HdcVector, b: HdcVector) -> f64 {
    1.0 - a.similarity(b)
}

/// Select initial medoid indices using farthest-first traversal.
///
/// Deterministic: always starts from index 0 and greedily picks the point
/// farthest from all existing medoids.
fn farthest_first_init(vectors: &[HdcVector], k: usize) -> Vec<usize> {
    let n = vectors.len();
    let mut medoid_indices = Vec::with_capacity(k);

    // First medoid is always index 0.
    medoid_indices.push(0);

    // Track the minimum distance from each point to any selected medoid.
    let mut min_dist_to_medoid = vec![f64::MAX; n];

    for _ in 1..k {
        // Update min distances with the most recently added medoid.
        let last_medoid_vec = vectors[*medoid_indices.last().unwrap_or(&0)];
        for (i, min_d) in min_dist_to_medoid.iter_mut().enumerate() {
            let d = distance(vectors[i], last_medoid_vec);
            if d < *min_d {
                *min_d = d;
            }
        }

        // Pick the point with maximum min-distance to any existing medoid.
        // Skip points already selected as medoids.
        let mut best_idx = 0;
        let mut best_dist = -1.0_f64;
        for (i, &d) in min_dist_to_medoid.iter().enumerate() {
            if !medoid_indices.contains(&i) && d > best_dist {
                best_dist = d;
                best_idx = i;
            }
        }
        medoid_indices.push(best_idx);
    }

    medoid_indices
}

/// Assign each vector to the nearest medoid, returning a vector of cluster indices.
fn assign_to_nearest(vectors: &[HdcVector], medoid_indices: &[usize]) -> Vec<usize> {
    vectors
        .iter()
        .map(|v| {
            let mut best_cluster = 0;
            let mut best_dist = f64::MAX;
            for (ci, &mi) in medoid_indices.iter().enumerate() {
                let d = distance(*v, vectors[mi]);
                if d < best_dist {
                    best_dist = d;
                    best_cluster = ci;
                }
            }
            best_cluster
        })
        .collect()
}

/// For a given cluster, find the member that minimizes total distance to all
/// other members. Returns the index (into `vectors`) of the best medoid.
fn find_best_medoid(vectors: &[HdcVector], member_indices: &[usize]) -> usize {
    if member_indices.len() <= 1 {
        return member_indices[0];
    }

    let mut best_idx = member_indices[0];
    let mut best_total = f64::MAX;

    for &candidate in member_indices {
        let total: f64 = member_indices
            .iter()
            .map(|&other| distance(vectors[candidate], vectors[other]))
            .sum();
        if total < best_total {
            best_total = total;
            best_idx = candidate;
        }
    }

    best_idx
}

/// Run PAM k-medoids clustering on a set of labeled HDC vectors.
///
/// Uses farthest-first initialization (deterministic, no RNG).
/// Distance metric: `1.0 - similarity` (normalized Hamming distance).
///
/// # Errors
/// - `HdcError::EmptyInput` if `vectors` is empty
/// - `HdcError::InvalidDedupConfig` if `k == 0`
///
/// # Panics
/// Never panics. If `k > vectors.len()`, clamps to `vectors.len()`.
/// Empty clusters (possible when medoids coincide) are omitted from the
/// result, so the returned count may be less than the effective k.
pub fn k_medoids<Id: Clone + PartialEq>(
    vectors: &[(Id, HdcVector)],
    k: usize,
) -> Result<Vec<Cluster<Id>>, HdcError> {
    if vectors.is_empty() {
        return Err(HdcError::EmptyInput {
            context: "k_medoids requires at least one vector",
        });
    }
    if k == 0 {
        return Err(HdcError::InvalidDedupConfig {
            reason: "k must be at least 1".to_string(),
        });
    }

    // Clamp k to n: can't have more clusters than vectors.
    let effective_k = k.min(vectors.len());

    // Extract just the vectors for internal computation.
    let vecs: Vec<HdcVector> = vectors.iter().map(|(_, v)| *v).collect();

    // --- Initialization ---
    let mut medoid_indices = farthest_first_init(&vecs, effective_k);

    // --- Iterative refinement ---
    let mut assignments = assign_to_nearest(&vecs, &medoid_indices);

    for _ in 0..MAX_ITERATIONS {
        // Update step: find best medoid within each cluster.
        let mut new_medoid_indices = Vec::with_capacity(effective_k);
        for (ci, &existing_medoid) in medoid_indices.iter().enumerate() {
            let members: Vec<usize> = assignments
                .iter()
                .enumerate()
                .filter(|&(_, a)| *a == ci)
                .map(|(i, _)| i)
                .collect();

            if members.is_empty() {
                // Keep the existing medoid if the cluster is empty (can happen
                // transiently when multiple medoids coincide).
                new_medoid_indices.push(existing_medoid);
            } else {
                new_medoid_indices.push(find_best_medoid(&vecs, &members));
            }
        }

        // Re-assign with updated medoids.
        let new_assignments = assign_to_nearest(&vecs, &new_medoid_indices);

        // Check for convergence.
        if new_assignments == assignments && new_medoid_indices == medoid_indices {
            medoid_indices = new_medoid_indices;
            assignments = new_assignments;
            break;
        }

        medoid_indices = new_medoid_indices;
        assignments = new_assignments;
    }

    // --- Build output clusters (skip empty clusters) ---
    let mut clusters = Vec::with_capacity(effective_k);
    for (ci, &medoid_vec_idx) in medoid_indices.iter().enumerate() {
        let member_indices: Vec<usize> = assignments
            .iter()
            .enumerate()
            .filter(|&(_, a)| *a == ci)
            .map(|(i, _)| i)
            .collect();

        // Drop clusters that ended up empty (can happen when multiple
        // medoids coincide on the same vector).
        if member_indices.is_empty() {
            continue;
        }

        let medoid_vec = vecs[medoid_vec_idx];

        // Compute inertia: average distance of non-medoid members to the
        // medoid.  Excluding the medoid's self-distance (always 0) avoids
        // under-reporting the true average intra-cluster spread.
        let non_medoid_count = member_indices.len() - 1;
        let inertia = if non_medoid_count == 0 {
            0.0
        } else {
            let total_dist: f64 = member_indices
                .iter()
                .filter(|&&i| i != medoid_vec_idx)
                .map(|&i| distance(vecs[i], medoid_vec))
                .sum();
            total_dist / non_medoid_count as f64
        };

        let members: Vec<Id> = member_indices
            .iter()
            .map(|&i| vectors[i].0.clone())
            .collect();

        clusters.push(Cluster {
            medoid_id: vectors[medoid_vec_idx].0.clone(),
            members,
            inertia,
        });
    }

    Ok(clusters)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::BundleAccumulator;

    /// Create a vector near the given base by bundling the base with heavy weight
    /// and adding light noise. The result has high similarity to the base.
    fn make_near_vector(base: HdcVector, noise_seed: &[u8]) -> HdcVector {
        let noise = HdcVector::from_seed("noise", noise_seed);
        let mut acc = BundleAccumulator::new();
        acc.add_weighted(&base, 10);
        acc.add_weighted(&noise, 1);
        acc.finalize()
    }

    #[test]
    fn distance_identical_is_zero() {
        let v = HdcVector::from_seed("test", b"dist");
        assert_eq!(distance(v, v), 0.0);
    }

    #[test]
    fn distance_random_near_half() {
        let a = HdcVector::from_seed("test", b"a");
        let b = HdcVector::from_seed("test", b"b");
        let d = distance(a, b);
        assert!(
            (0.45..0.55).contains(&d),
            "expected distance near 0.5, got {d}"
        );
    }

    #[test]
    fn farthest_first_selects_k_distinct() {
        let vecs: Vec<HdcVector> = (0..10)
            .map(|i| HdcVector::from_seed("ff", format!("{i}").as_bytes()))
            .collect();
        let medoids = farthest_first_init(&vecs, 3);
        assert_eq!(medoids.len(), 3);
        // All distinct indices.
        assert_ne!(medoids[0], medoids[1]);
        assert_ne!(medoids[0], medoids[2]);
        assert_ne!(medoids[1], medoids[2]);
    }

    #[test]
    fn farthest_first_single() {
        let vecs = vec![HdcVector::from_seed("ff", b"only")];
        let medoids = farthest_first_init(&vecs, 1);
        assert_eq!(medoids, vec![0]);
    }

    /// Build two clearly separated groups: group A vectors are all near `base_a`,
    /// group B vectors are all near `base_b`.  With high-weight bundling the
    /// intra-group similarity is ~0.95 while inter-group similarity stays ~0.5.
    #[test]
    fn k_medoids_basic_two_clusters() {
        let base_a = HdcVector::from_seed("cluster_test", b"group_a");
        let base_b = HdcVector::from_seed("cluster_test", b"group_b");

        // Six vectors: three near base_a (ids 0-2), three near base_b (ids 3-5).
        let mut input: Vec<(usize, HdcVector)> = Vec::new();
        for i in 0..3usize {
            input.push((i, make_near_vector(base_a, format!("a{}", i).as_bytes())));
        }
        for i in 3..6usize {
            input.push((i, make_near_vector(base_b, format!("b{}", i).as_bytes())));
        }

        let clusters = k_medoids(&input, 2).expect("k_medoids should succeed");
        assert_eq!(clusters.len(), 2, "expected exactly 2 clusters");

        // Total member count across all clusters must equal input size.
        let total_members: usize = clusters.iter().map(|c| c.members.len()).sum();
        assert_eq!(
            total_members, 6,
            "all vectors must be assigned to a cluster"
        );

        // Each cluster should be non-empty.
        for c in &clusters {
            assert!(
                !c.members.is_empty(),
                "cluster should have at least one member"
            );
        }

        // Identify which cluster holds which group by looking at the medoid id.
        // The medoid of the A-group cluster must be one of ids 0-2, and
        // all members of that cluster should also be from ids 0-2 (or 3-5 for B).
        let (cluster_a, cluster_b) = if clusters[0].medoid_id < 3 {
            (&clusters[0], &clusters[1])
        } else {
            (&clusters[1], &clusters[0])
        };

        // Medoids must belong to the correct group.
        assert!(
            cluster_a.medoid_id < 3,
            "cluster A medoid must be in group A"
        );
        assert!(
            cluster_b.medoid_id >= 3,
            "cluster B medoid must be in group B"
        );

        // All members of cluster A must be from ids 0-2 and vice versa for B.
        for &id in &cluster_a.members {
            assert!(id < 3, "cluster A member {id} should be in group A (0-2)");
        }
        for &id in &cluster_b.members {
            assert!(id >= 3, "cluster B member {id} should be in group B (3-5)");
        }
    }

    #[test]
    fn k_medoids_single_cluster() {
        let input: Vec<(usize, HdcVector)> = (0..5)
            .map(|i| (i, HdcVector::from_seed("sc", format!("{i}").as_bytes())))
            .collect();

        let clusters = k_medoids(&input, 1).expect("k_medoids should succeed");
        assert_eq!(clusters.len(), 1, "k=1 must yield exactly one cluster");

        // All 5 items must be in the single cluster.
        assert_eq!(
            clusters[0].members.len(),
            5,
            "single cluster must contain all vectors"
        );
    }

    #[test]
    fn k_medoids_k_exceeds_n() {
        // 3 vectors, k=10 → should clamp to 3 clusters without panic.
        let input: Vec<(usize, HdcVector)> = (0..3)
            .map(|i| (i, HdcVector::from_seed("clamp", format!("{i}").as_bytes())))
            .collect();

        let clusters = k_medoids(&input, 10).expect("k_medoids should clamp gracefully");
        assert_eq!(
            clusters.len(),
            3,
            "k > n must clamp to n clusters (one per vector)"
        );

        let total_members: usize = clusters.iter().map(|c| c.members.len()).sum();
        assert_eq!(total_members, 3, "all 3 vectors must be assigned");
    }

    #[test]
    fn k_medoids_empty_input_errors() {
        let input: Vec<(usize, HdcVector)> = vec![];
        let result = k_medoids(&input, 2);
        assert!(
            matches!(result, Err(HdcError::EmptyInput { .. })),
            "empty input should return EmptyInput error, got {result:?}"
        );
    }

    #[test]
    fn k_medoids_k_zero_errors() {
        let input: Vec<(usize, HdcVector)> = vec![(0, HdcVector::from_seed("zero_k", b"only"))];
        let result = k_medoids(&input, 0);
        assert!(
            matches!(result, Err(HdcError::InvalidDedupConfig { .. })),
            "k=0 should return InvalidDedupConfig error, got {result:?}"
        );
    }

    #[test]
    fn all_identical_vectors() {
        let v = HdcVector::from_seed("identical", b"same");
        let input: Vec<(usize, HdcVector)> = (0..5).map(|i| (i, v)).collect();
        let clusters = k_medoids(&input, 3).expect("should not panic on identical vectors");
        // All vectors are identical, so they all end up in one cluster
        // (distance to every medoid is 0, ties break to cluster 0).
        let total: usize = clusters.iter().map(|c| c.members.len()).sum();
        assert_eq!(total, 5, "all vectors must be assigned");
        for c in &clusters {
            assert_eq!(c.inertia, 0.0, "identical vectors have zero inertia");
        }
    }

    #[test]
    fn deterministic_output() {
        let input: Vec<(usize, HdcVector)> = (0..10)
            .map(|i| {
                (
                    i,
                    HdcVector::from_seed("det", format!("item_{i}").as_bytes()),
                )
            })
            .collect();
        let a = k_medoids(&input, 3).unwrap();
        let b = k_medoids(&input, 3).unwrap();
        assert_eq!(a.len(), b.len(), "cluster count must be deterministic");
        for (ca, cb) in a.iter().zip(b.iter()) {
            assert_eq!(ca.medoid_id, cb.medoid_id, "medoid ids must match");
            assert_eq!(ca.members, cb.members, "member lists must match");
            assert_eq!(ca.inertia, cb.inertia, "inertia must match");
        }
    }

    #[test]
    fn medoid_in_members() {
        let input: Vec<(usize, HdcVector)> = (0..10)
            .map(|i| (i, HdcVector::from_seed("mid", format!("v{i}").as_bytes())))
            .collect();
        let clusters = k_medoids(&input, 3).unwrap();
        for c in &clusters {
            assert!(
                c.members.contains(&c.medoid_id),
                "medoid_id {} must appear in members {:?}",
                c.medoid_id,
                c.members
            );
        }
    }

    #[test]
    fn three_separated_groups() {
        let base_a = HdcVector::from_seed("group_a_base", b"anchor");
        let base_b = HdcVector::from_seed("group_b_base", b"anchor");
        let base_c = HdcVector::from_seed("group_c_base", b"anchor");

        let mut input: Vec<(usize, HdcVector)> = Vec::with_capacity(30);
        // Group 0: ids 0..10 near base_a
        for i in 0..10usize {
            input.push((i, make_near_vector(base_a, format!("a{i}").as_bytes())));
        }
        // Group 1: ids 10..20 near base_b
        for i in 10..20usize {
            input.push((i, make_near_vector(base_b, format!("b{i}").as_bytes())));
        }
        // Group 2: ids 20..30 near base_c
        for i in 20..30usize {
            input.push((i, make_near_vector(base_c, format!("c{i}").as_bytes())));
        }

        let clusters = k_medoids(&input, 3).expect("k_medoids should succeed");
        assert_eq!(clusters.len(), 3, "expected 3 clusters");

        let total: usize = clusters.iter().map(|c| c.members.len()).sum();
        assert_eq!(total, 30, "all 30 vectors must be assigned");

        // Check purity: for each cluster, the majority of members should
        // come from the same original group.
        for c in &clusters {
            let group_a_count = c.members.iter().filter(|&&id| id < 10).count();
            let group_b_count = c
                .members
                .iter()
                .filter(|&&id| (10..20).contains(&id))
                .count();
            let group_c_count = c.members.iter().filter(|&&id| id >= 20).count();
            let majority = group_a_count.max(group_b_count).max(group_c_count);
            let purity = majority as f64 / c.members.len() as f64;
            assert!(
                purity >= 0.8,
                "cluster with medoid {} has purity {purity:.2} (a={group_a_count}, b={group_b_count}, c={group_c_count}), expected >= 0.8",
                c.medoid_id
            );
        }
    }
}
