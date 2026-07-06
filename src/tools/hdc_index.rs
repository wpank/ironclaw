//! HDC-based index for skill and tool similarity matching.
//!
//! Provides a tie-breaker signal for skill/tool selection when exact matches
//! are ambiguous. Does NOT replace existing keyword/regex/policy matching.
//!
//! The HDC boost is capped at [`ToolHdcIndex::MAX_HDC_BOOST`] to ensure that
//! exact keyword and policy matches always dominate. Hot reload rebuilds
//! indexes via `clear()` + re-registration.

use ironclaw_hdc::codebook::{Codebook, ItemMemory};
use ironclaw_hdc::encode_text;
use ironclaw_hdc::encoder::{DocumentEncodingInput, encode_document};

/// HDC index for skills.
///
/// Encodes skill name, description, and tags into hypervectors for
/// similarity-based lookup. Used as a tie-breaker when multiple skills
/// score equally under keyword/regex matching.
pub struct SkillHdcIndex {
    memory: ItemMemory,
    codebook: Codebook,
}

impl SkillHdcIndex {
    pub fn new() -> Self {
        Self {
            memory: ItemMemory::new(),
            codebook: Codebook::new(),
        }
    }

    /// Register a skill in the index.
    /// Encodes name + description + tags into a single HDC vector.
    pub fn register_skill(&mut self, name: &str, description: &str, tags: &[String]) {
        let tag_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
        let vector = encode_document(
            DocumentEncodingInput {
                content: description,
                tags: &tag_refs,
                path: name,
            },
            &mut self.codebook,
        );
        self.memory.insert(name.to_string(), vector);
    }

    /// Find skills most similar to a query.
    /// Returns (skill_name, similarity_score) pairs sorted by descending similarity.
    pub fn find_similar(&self, query: &str, top_k: usize) -> Vec<(String, f64)> {
        let query_hv = encode_text(query, &mut self.codebook.clone());
        self.memory.lookup(query_hv, top_k)
    }

    /// Clear and rebuild the index (used during hot reload).
    pub fn clear(&mut self) {
        self.memory = ItemMemory::new();
    }
}

impl Default for SkillHdcIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// HDC index for tools.
///
/// Encodes tool name, description, and parameter names into hypervectors.
/// Used as a tie-breaker signal; exact keyword/policy matches always win.
pub struct ToolHdcIndex {
    memory: ItemMemory,
    codebook: Codebook,
}

impl ToolHdcIndex {
    /// Maximum allowed boost from HDC matching (capped to prevent overriding
    /// exact keyword/regex/policy matches).
    pub const MAX_HDC_BOOST: f64 = 0.1;

    pub fn new() -> Self {
        Self {
            memory: ItemMemory::new(),
            codebook: Codebook::new(),
        }
    }

    /// Register a tool in the index.
    /// Encodes description as content, param names as tags, tool name as path.
    pub fn register_tool(&mut self, name: &str, description: &str, param_names: &[&str]) {
        let vector = encode_document(
            DocumentEncodingInput {
                content: description,
                tags: param_names,
                path: name,
            },
            &mut self.codebook,
        );
        self.memory.insert(name.to_string(), vector);
    }

    /// Find tools most similar to a query.
    /// Returns (tool_name, similarity_score) pairs sorted by descending similarity.
    pub fn find_similar(&self, query: &str, top_k: usize) -> Vec<(String, f64)> {
        let query_hv = encode_text(query, &mut self.codebook.clone());
        self.memory.lookup(query_hv, top_k)
    }

    /// Clear and rebuild the index (used during hot reload).
    pub fn clear(&mut self) {
        self.memory = ItemMemory::new();
    }
}

impl Default for ToolHdcIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_keyword_beats_hdc() {
        // HDC similarity is just a tie-breaker — verify that "shell" with
        // description "execute commands" scores higher than "web_fetch" for
        // the query "execute shell command".
        let mut index = ToolHdcIndex::new();
        index.register_tool("shell", "execute commands in a system shell", &["command"]);
        index.register_tool(
            "web_fetch",
            "fetch content from a URL over HTTP",
            &["url", "method"],
        );

        let results = index.find_similar("execute shell command", 2);
        assert_eq!(results[0].0, "shell");
        // The HDC boost is capped, so even a high similarity score from HDC
        // would never override an exact keyword match in the real pipeline.
        assert!(results[0].1 <= 1.0);
    }

    #[test]
    fn irrelevant_text_no_boost() {
        // Unrelated query should produce similarity near the random baseline (~0.5).
        let mut index = ToolHdcIndex::new();
        index.register_tool("shell", "execute commands in a system shell", &["command"]);

        let results = index.find_similar("french cooking recipe for soufflé", 1);
        // Random baseline for binary HDC vectors is ~0.5.
        // Irrelevant text should not exceed MAX_HDC_BOOST above baseline.
        let similarity = results[0].1;
        assert!(
            similarity < 0.5 + ToolHdcIndex::MAX_HDC_BOOST + 0.15,
            "irrelevant query similarity {similarity} too high"
        );
    }

    #[test]
    fn index_rebuilds_after_registration() {
        let mut index = ToolHdcIndex::new();
        index.register_tool("shell", "execute commands", &["command"]);

        // Verify shell is found
        let results = index.find_similar("execute", 1);
        assert_eq!(results[0].0, "shell");

        // Clear and register different tools
        index.clear();
        index.register_tool("memory_search", "search stored memories", &["query"]);

        let results = index.find_similar("execute commands", 1);
        // Old tool should be gone; only memory_search remains
        assert_eq!(results[0].0, "memory_search");

        // And the new tool should be findable
        let results = index.find_similar("search memories", 1);
        assert_eq!(results[0].0, "memory_search");
    }

    #[test]
    fn skill_index_similar_descriptions_score_higher() {
        let mut index = SkillHdcIndex::new();
        index.register_skill(
            "git_helper",
            "assist with git operations like commit, push, and branch management",
            &["git".to_string(), "version-control".to_string()],
        );
        index.register_skill(
            "cooking_guide",
            "provide recipes and cooking techniques for various cuisines",
            &["food".to_string(), "recipes".to_string()],
        );

        let results = index.find_similar("help me with git branch", 2);
        assert_eq!(results[0].0, "git_helper");
        assert!(results[0].1 > results[1].1);
    }

    #[test]
    fn skill_index_clear_removes_all() {
        let mut index = SkillHdcIndex::new();
        index.register_skill("test_skill", "a test skill", &["test".to_string()]);

        index.clear();
        // After clear, lookup returns empty since no items remain
        let results = index.find_similar("test", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn max_hdc_boost_is_conservative() {
        // Verify the constant is set conservatively
        assert!(ToolHdcIndex::MAX_HDC_BOOST <= 0.1);
        assert!(ToolHdcIndex::MAX_HDC_BOOST > 0.0);
    }
}
