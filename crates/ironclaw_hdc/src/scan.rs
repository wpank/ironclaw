use crate::vector::HdcVector;

/// Result of a similarity scan.
#[derive(Debug, Clone)]
pub struct ScanResult<Id> {
    pub id: Id,
    pub similarity: f64,
}

/// Scan candidates and return top_k most similar, sorted descending.
pub fn top_k_scan<Id: Clone>(
    query: HdcVector,
    candidates: &[(Id, HdcVector)],
    top_k: usize,
) -> Vec<ScanResult<Id>> {
    let mut results: Vec<ScanResult<Id>> = candidates
        .iter()
        .map(|(id, fp)| ScanResult {
            id: id.clone(),
            similarity: query.similarity(*fp),
        })
        .collect();
    results.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(top_k);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_k_empty_candidates() {
        let query = HdcVector::from_seed("test", b"q");
        let results: Vec<ScanResult<u64>> = top_k_scan(query, &[], 5);
        assert!(results.is_empty());
    }

    #[test]
    fn top_k_returns_sorted_descending() {
        let query = HdcVector::from_seed("test", b"query");
        let candidates: Vec<(u64, HdcVector)> = (0..5)
            .map(|i| (i, HdcVector::from_seed("test", format!("c{i}").as_bytes())))
            .collect();
        let results = top_k_scan(query, &candidates, 3);
        assert_eq!(results.len(), 3);
        for w in results.windows(2) {
            assert!(w[0].similarity >= w[1].similarity);
        }
    }

    #[test]
    fn top_k_zero() {
        let query = HdcVector::from_seed("test", b"q");
        let candidates: Vec<(u64, HdcVector)> = (0..5)
            .map(|i| (i, HdcVector::from_seed("test", format!("c{i}").as_bytes())))
            .collect();
        let results = top_k_scan(query, &candidates, 0);
        assert!(results.is_empty());
    }

    #[test]
    fn top_k_one() {
        let query = HdcVector::from_seed("test", b"q");
        let candidates: Vec<(u64, HdcVector)> = (0..5)
            .map(|i| (i, HdcVector::from_seed("test", format!("c{i}").as_bytes())))
            .collect();
        let results = top_k_scan(query, &candidates, 1);
        assert_eq!(results.len(), 1);
        // The single result should be the most similar candidate
        let all_results = top_k_scan(query, &candidates, 5);
        assert_eq!(results[0].id, all_results[0].id);
    }

    #[test]
    fn top_k_greater_than_len() {
        let query = HdcVector::from_seed("test", b"q");
        let candidates: Vec<(u64, HdcVector)> = (0..3)
            .map(|i| (i, HdcVector::from_seed("test", format!("c{i}").as_bytes())))
            .collect();
        let results = top_k_scan(query, &candidates, 10);
        assert_eq!(results.len(), 3);
        // Sorted descending by similarity
        for w in results.windows(2) {
            assert!(w[0].similarity >= w[1].similarity);
        }
    }

    #[test]
    fn empty_candidates() {
        let query = HdcVector::from_seed("test", b"q");
        let results: Vec<ScanResult<u64>> = top_k_scan(query, &[], 10);
        assert!(results.is_empty());
    }

    #[test]
    fn deterministic_ordering() {
        let query = HdcVector::from_seed("test", b"det");
        let candidates: Vec<(u64, HdcVector)> = (0..10)
            .map(|i| (i, HdcVector::from_seed("test", format!("d{i}").as_bytes())))
            .collect();
        let run1 = top_k_scan(query, &candidates, 5);
        let run2 = top_k_scan(query, &candidates, 5);
        assert_eq!(run1.len(), run2.len());
        for (a, b) in run1.iter().zip(run2.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.similarity, b.similarity);
        }
    }

    #[test]
    fn top_k_self_match_is_first() {
        let query = HdcVector::from_seed("test", b"self");
        let candidates = vec![
            (1u64, HdcVector::from_seed("test", b"other")),
            (2u64, HdcVector::from_seed("test", b"self")),
            (3u64, HdcVector::from_seed("test", b"another")),
        ];
        let results = top_k_scan(query, &candidates, 3);
        assert_eq!(results[0].id, 2);
        assert_eq!(results[0].similarity, 1.0);
    }
}
