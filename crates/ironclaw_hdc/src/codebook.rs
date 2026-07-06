use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::vector::HdcVector;

/// A codebook maps symbolic names to deterministic hypervectors.
///
/// Vectors are created via `HdcVector::from_seed`, making them reproducible
/// across runs without requiring external randomness. Any symbol requested
/// for the first time is assigned a deterministic vector derived from its name.
#[derive(Clone, Serialize, Deserialize)]
pub struct Codebook {
    entries: HashMap<String, HdcVector>,
}

impl Codebook {
    /// Create an empty codebook.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Pre-populate with given symbols.
    ///
    /// Each symbol gets a deterministic vector assigned immediately.
    pub fn with_defaults(symbols: &[&str]) -> Self {
        let mut cb = Self::new();
        for &s in symbols {
            cb.get_or_create(s);
        }
        cb
    }

    /// Get or create a vector for the symbol.
    ///
    /// Deterministic: uses `HdcVector::from_seed("codebook", symbol.as_bytes())`.
    /// The same symbol always produces the same vector regardless of insertion order.
    pub fn get_or_create(&mut self, symbol: &str) -> HdcVector {
        if let Some(&v) = self.entries.get(symbol) {
            return v;
        }
        let v = HdcVector::from_seed("codebook", symbol.as_bytes());
        self.entries.insert(symbol.to_string(), v);
        v
    }

    /// Lookup without inserting. Returns `None` if absent.
    pub fn get(&self, symbol: &str) -> Option<HdcVector> {
        self.entries.get(symbol).copied()
    }

    /// Number of entries in the codebook.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the codebook is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for Codebook {
    fn default() -> Self {
        Self::new()
    }
}

/// An item memory stores named vectors for nearest-neighbor lookup.
///
/// Items are identified by name (no duplicates) and can be queried by
/// similarity to a given vector, returning the top-k matches.
#[derive(Clone)]
pub struct ItemMemory {
    items: Vec<(String, HdcVector)>,
}

impl ItemMemory {
    /// Create an empty item memory.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Insert or replace by name (no duplicate names).
    pub fn insert(&mut self, name: String, vector: HdcVector) {
        if let Some(pos) = self.items.iter().position(|(n, _)| n == &name) {
            self.items[pos].1 = vector;
        } else {
            self.items.push((name, vector));
        }
    }

    /// Lookup top_k most similar entries.
    ///
    /// Returns `(name, similarity)` sorted by descending similarity,
    /// then by name for deterministic tie-breaking.
    pub fn lookup(&self, query: HdcVector, top_k: usize) -> Vec<(String, f64)> {
        let mut scored: Vec<(String, f64)> = self
            .items
            .iter()
            .map(|(name, vec)| (name.clone(), query.similarity(*vec)))
            .collect();
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        scored.truncate(top_k);
        scored
    }

    /// Remove an item by name.
    pub fn remove(&mut self, name: &str) {
        self.items.retain(|(n, _)| n != name);
    }

    /// Number of items stored.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the item memory is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for ItemMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codebook_get_or_create_is_deterministic() {
        let mut cb1 = Codebook::new();
        let mut cb2 = Codebook::new();
        let v1 = cb1.get_or_create("hello");
        let v2 = cb2.get_or_create("hello");
        assert_eq!(v1, v2);
    }

    #[test]
    fn codebook_get_or_create_returns_same_on_second_call() {
        let mut cb = Codebook::new();
        let v1 = cb.get_or_create("test");
        let v2 = cb.get_or_create("test");
        assert_eq!(v1, v2);
        assert_eq!(cb.len(), 1);
    }

    #[test]
    fn codebook_get_returns_none_for_missing() {
        let cb = Codebook::new();
        assert!(cb.get("missing").is_none());
    }

    #[test]
    fn codebook_get_returns_some_after_insert() {
        let mut cb = Codebook::new();
        let v = cb.get_or_create("present");
        assert_eq!(cb.get("present"), Some(v));
    }

    #[test]
    fn codebook_with_defaults_populates() {
        let cb = Codebook::with_defaults(&["a", "b", "c"]);
        assert_eq!(cb.len(), 3);
        assert!(cb.get("a").is_some());
        assert!(cb.get("b").is_some());
        assert!(cb.get("c").is_some());
    }

    #[test]
    fn codebook_different_symbols_differ() {
        let mut cb = Codebook::new();
        let a = cb.get_or_create("alpha");
        let b = cb.get_or_create("beta");
        assert_ne!(a, b);
    }

    #[test]
    fn item_memory_insert_and_lookup() {
        let mut mem = ItemMemory::new();
        let v = HdcVector::from_seed("test", b"item1");
        mem.insert("item1".to_string(), v);
        let results = mem.lookup(v, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "item1");
        assert_eq!(results[0].1, 1.0);
    }

    #[test]
    fn item_memory_replace_by_name() {
        let mut mem = ItemMemory::new();
        let v1 = HdcVector::from_seed("test", b"first");
        let v2 = HdcVector::from_seed("test", b"second");
        mem.insert("x".to_string(), v1);
        mem.insert("x".to_string(), v2);
        assert_eq!(mem.len(), 1);
        let results = mem.lookup(v2, 1);
        assert_eq!(results[0].1, 1.0);
    }

    #[test]
    fn item_memory_remove() {
        let mut mem = ItemMemory::new();
        mem.insert("a".to_string(), HdcVector::from_seed("t", b"a"));
        mem.insert("b".to_string(), HdcVector::from_seed("t", b"b"));
        assert_eq!(mem.len(), 2);
        mem.remove("a");
        assert_eq!(mem.len(), 1);
    }

    #[test]
    fn item_memory_lookup_top_k_ordering() {
        let mut mem = ItemMemory::new();
        let query = HdcVector::from_seed("q", b"query");
        // Insert the query itself (similarity 1.0) and a random vector (~0.5)
        mem.insert("exact".to_string(), query);
        mem.insert("random".to_string(), HdcVector::from_seed("r", b"other"));
        let results = mem.lookup(query, 2);
        assert_eq!(results[0].0, "exact");
        assert_eq!(results[0].1, 1.0);
        assert!(results[1].1 < 0.6);
    }

    #[test]
    fn item_memory_lookup_truncates_to_top_k() {
        let mut mem = ItemMemory::new();
        for i in 0..10 {
            mem.insert(
                format!("item{i}"),
                HdcVector::from_seed("bulk", format!("{i}").as_bytes()),
            );
        }
        let query = HdcVector::from_seed("bulk", b"0");
        let results = mem.lookup(query, 3);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn codebook_orthogonal_symbols() {
        let mut cb = Codebook::new();
        let symbols: Vec<HdcVector> = (0..50)
            .map(|i| cb.get_or_create(&format!("symbol_{i}")))
            .collect();
        for i in 0..50 {
            for j in (i + 1)..50 {
                let sim = symbols[i].similarity(symbols[j]);
                assert!(
                    (0.47..0.53).contains(&sim),
                    "symbols {i} and {j} have similarity {sim}, expected 0.47..0.53"
                );
            }
        }
    }

    #[test]
    fn codebook_serde_roundtrip() {
        let mut cb = Codebook::new();
        cb.get_or_create("alpha");
        cb.get_or_create("beta");
        cb.get_or_create("gamma");

        let json = serde_json::to_string(&cb).unwrap();
        let restored: Codebook = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.len(), 3);
        assert_eq!(restored.get("alpha"), cb.get("alpha"));
        assert_eq!(restored.get("beta"), cb.get("beta"));
        assert_eq!(restored.get("gamma"), cb.get("gamma"));
    }

    #[test]
    fn item_memory_top_k_clamp() {
        let mut mem = ItemMemory::new();
        for i in 0..5 {
            mem.insert(
                format!("item{i}"),
                HdcVector::from_seed("clamp", format!("{i}").as_bytes()),
            );
        }
        let query = HdcVector::from_seed("clamp", b"0");
        let results = mem.lookup(query, 1000);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn item_memory_deterministic_ties() {
        let mut mem = ItemMemory::new();
        // Insert items with same similarity to query (use vectors equidistant from query)
        let query = HdcVector::from_seed("tie", b"query");
        // Two distinct vectors that should have similar (but not identical) similarity to query
        mem.insert("beta".to_string(), HdcVector::from_seed("tie", b"x1"));
        mem.insert("alpha".to_string(), HdcVector::from_seed("tie", b"x1"));
        let results = mem.lookup(query, 2);
        // When similarities are equal, should sort by name
        if results[0].1 == results[1].1 {
            assert_eq!(results[0].0, "alpha");
            assert_eq!(results[1].0, "beta");
        }
    }
}
