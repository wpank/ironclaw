# Code Intelligence: Multi-Modal Source Indexing, Graph Ranking, and Hybrid Search

**Source crate**: `roko-index` (with language-specific crates `roko-lang-rust`, `roko-lang-typescript`, `roko-lang-go`)
**Priority**: MEDIUM -- multi-modal code indexing, PageRank, HDC fingerprints, hybrid search
**Roko docs**: `docs/v2-depth/22-code-intelligence/` (5 depth docs absorbing 11 source docs from `docs/v1/15-code-intelligence/`)

---

## 1. Introduction: What Code Intelligence Is and Why It Matters

Code intelligence gives an AI agent structural understanding of a codebase -- not just the ability to search for text patterns, but genuine knowledge of what symbols exist, how they relate to each other through typed dependency edges, which symbols are structurally important (measured by PageRank), and which symbols are structurally similar (measured by hyperdimensional fingerprints). The term encompasses a family of techniques from compiler front-ends, information retrieval, and graph theory, adapted specifically for the problem of assembling minimal, high-relevance context windows for large language models.

The fundamental problem is **context assembly**. Given a natural-language task description ("add error handling to `process_input`"), an AI coding agent must decide which source code fragments to include in its prompt. Without code intelligence, the agent falls back to text search (grep), which produces noisy results: 20-50 candidate files, roughly 50,000 tokens of raw source text, with no structural understanding of how the matched symbols relate to each other. The LLM must then spend its own capacity figuring out which function is the right one, what it calls, what calls it, and what types it depends on.

With code intelligence, the same agent gets a ranked, graph-expanded, budget-constrained context of roughly 5,000 tokens containing exactly the target function, its callers, its type dependencies, and nothing else. The savings compound: fewer tokens means faster inference, lower cost, and higher accuracy because the model's attention is not diluted by irrelevant code.

The roko code intelligence engine achieves this through **multi-modal indexing** -- four parallel strategies that each capture a different dimension of code structure, then merge their results using Reciprocal Rank Fusion (RRF) [1]. No single indexing strategy is sufficient:

- **Symbol indexing** finds exact names but misses semantic similarity.
- **Graph indexing** finds structural relationships but misses content similarity.
- **HDC fingerprinting** finds structural similarity but misses exact names.
- **Full-text search** finds fuzzy text matches but misses structural relationships.

By running all four in parallel and merging with RRF, the system produces results that any single strategy would miss. A symbol that appears in three of four result lists rises to the top even if it was ranked low in each individual list.

---

## 2. Architecture Overview

The code intelligence system spans five crates:

```
crates/
  roko-core/src/language.rs      # Core trait definitions: LanguageProvider, BuildSystem,
                                 # Symbol, SymbolKind, Import, ImportKind, Visibility
  roko-core/src/build.rs         # BuildSystem trait and BuildCommand type

  roko-lang-rust/src/
    lib.rs                       # RustLanguageProvider (heuristic), CargoBuildSystem
    tree_sitter_parser.rs        # TreeSitterRustProvider (AST-based, feature-gated)

  roko-lang-typescript/src/
    lib.rs                       # TypeScriptLanguageProvider, NpmBuildSystem,
                                 # PnpmBuildSystem, YarnBuildSystem

  roko-lang-go/src/
    lib.rs                       # GoLanguageProvider, GoBuildSystem

  roko-index/src/
    lib.rs                       # Public API, convenience re-exports
    parser.rs                    # Language-agnostic SourceFile + parse_source()
    symbol.rs                    # SymbolId, SymbolRef, find_symbol()
    graph.rs                     # SymbolGraph, EdgeKind, build_graph(),
                                 # pagerank(), weighted_pagerank(),
                                 # personalized_pagerank()
    hdc.rs                       # HdcFingerprint, fingerprint_symbol(),
                                 # fingerprint_file(), similarity()
    sqlite.rs                    # SqliteIndex -- persistent storage (feature-gated)
    workspace.rs                 # WorkspaceIndex, CodeIndex trait, SearchStrategy,
                                 # RRF merge, context assembly, overlays, privacy
```

The separation is deliberate: `roko-index` contains zero language-specific logic. All language knowledge lives in `roko-lang-*` crates that implement the `LanguageProvider` trait from `roko-core`. Adding Python support means implementing `PythonLanguageProvider`; every downstream module (graph, HDC, search, context assembly) works unchanged.

> **Ref**: `crates/roko-index/src/lib.rs` (lines 1-47), `crates/roko-core/src/language.rs` (lines 14-106)

---

## 3. Core Data Types

Before diving into the four indexing modes, these are the foundational types that everything builds on.

### Symbol (defined in roko-core)

A symbol is a named entity extracted from source code:

```rust
// crates/roko-core/src/language.rs

pub enum SymbolKind {
    Function,   // fn, function, func
    Struct,     // struct, class, type X struct
    Enum,       // enum
    Trait,      // trait, interface, type X interface
    Const,      // const, var (Go)
    Type,       // type alias
    Module,     // mod, export default
    Impl,       // impl block
}

pub enum Visibility {
    Public,     // pub, export, capitalized (Go)
    Private,    // default, unexported
}

pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub visibility: Visibility,
    pub line: usize,         // 1-based line number
}
```

The 8-variant `SymbolKind` normalizes constructs across languages:

| Rust | TypeScript | Go | SymbolKind |
|---|---|---|---|
| `fn` | `function` | `func` | `Function` |
| `struct` | `class` | `type X struct` | `Struct` |
| `enum` | `enum` | -- | `Enum` |
| `trait` | `interface` | `type X interface` | `Trait` |
| `const` | `const` | `const` / `var` | `Const` |
| `type` | `type` | `type` (non-struct) | `Type` |
| `mod` | `export default` | -- | `Module` |
| `impl` | -- | -- | `Impl` |

This uniform mapping means the graph, PageRank, HDC fingerprints, and search all treat symbols identically regardless of source language. A Go `interface` and a Rust `trait` both produce `SymbolKind::Trait` nodes.

> **Ref**: `crates/roko-core/src/language.rs` (lines 43-58, 60-92)

### SymbolId (defined in roko-index)

A unique identifier for a symbol within an index, composed of three fields:

```rust
// crates/roko-index/src/symbol.rs

pub struct SymbolId {
    pub file_path: String,
    pub symbol_name: String,
    pub kind: SymbolKind,
}
```

Two symbols with identical `(file_path, symbol_name, kind)` are considered the same definition. Two symbols with the same name but different kinds are distinct (a `struct Config` and an `fn Config` constructor). Two symbols with the same name and kind in different files are distinct.

Display format: `"lib.rs::main(Function)"`.

> **Ref**: `crates/roko-index/src/symbol.rs` (lines 20-66)

### SymbolRef

A reference to a symbol at a specific location in source code:

```rust
// crates/roko-index/src/symbol.rs

pub struct SymbolRef {
    pub file: String,
    pub line: usize,    // 1-based
    pub column: usize,  // 0-based
}
```

### SourceFile

The parsed representation of a single source file, produced by the parser module:

```rust
// crates/roko-index/src/parser.rs

pub struct SourceFile {
    pub path: String,
    pub language: String,
    pub content: String,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<Import>,
}
```

### Import

An import statement extracted from source:

```rust
// crates/roko-core/src/language.rs

pub enum ImportKind {
    Use,          // Rust use, TS import, Go import
    Mod,          // Rust mod declaration
    ExternCrate,  // Rust extern crate
}

pub struct Import {
    pub path: String,
    pub alias: Option<String>,
    pub kind: ImportKind,
}
```

---

## 4. Multi-Language Support

### The LanguageProvider Trait

All parsing flows through a single trait defined in `roko-core`:

```rust
// crates/roko-core/src/language.rs

pub trait LanguageProvider: Send + Sync {
    fn language_name(&self) -> &str;
    fn file_extensions(&self) -> &[&str];
    fn parse_imports(&self, source: &str) -> Vec<Import>;
    fn extract_symbols(&self, source: &str) -> Vec<Symbol>;
}
```

The `parse_source` function in `roko-index` delegates to the appropriate provider:

```rust
// crates/roko-index/src/parser.rs

pub fn parse_source(path: &str, content: &str, provider: &dyn LanguageProvider) -> SourceFile {
    let symbols = provider.extract_symbols(content);
    let imports = provider.parse_imports(content);
    SourceFile { path: path.to_string(), language: provider.language_name().to_string(),
                 content: content.to_string(), symbols, imports }
}
```

### The BuildSystem Trait

Companion trait for language-specific build commands:

```rust
// crates/roko-core/src/build.rs

pub trait BuildSystem: Send + Sync {
    fn name(&self) -> &str;
    fn compile_cmd(&self, target_dir: &Path) -> BuildCommand;
    fn test_cmd(&self, target_dir: &Path, filter: Option<&str>) -> BuildCommand;
    fn lint_cmd(&self, target_dir: &Path) -> BuildCommand;
    fn format_cmd(&self, target_dir: &Path, check_only: bool) -> BuildCommand;
    fn detect_from_files(&self, file_names: &[&str]) -> bool;
}
```

The `detect_from_files` method enables automatic build system detection: `CargoBuildSystem` looks for `Cargo.toml`, `NpmBuildSystem` looks for `package.json` (without `pnpm-lock.yaml` or `yarn.lock`), `GoBuildSystem` looks for `go.mod`.

### Rust Provider (Dual-Mode: Heuristic + Tree-Sitter)

The Rust language crate provides two implementations of `LanguageProvider`, selectable at compile time:

**1. Heuristic parser** (`RustLanguageProvider`): Line-by-line parsing. Always available. Handles `fn`, `struct`, `enum`, `trait`, `impl`, `const`, `type`, `mod`. Parses `use` imports with brace expansion (`use std::collections::{HashMap, HashSet}`), `mod` declarations, and `extern crate`. Handles `pub`, `pub(crate)`, `pub(super)` visibility. Strips `async`, `unsafe`, `const`, `extern "C"` function qualifiers. Skips balanced angle brackets in generic signatures.

```rust
// crates/roko-lang-rust/src/lib.rs

pub struct RustLanguageProvider;

impl LanguageProvider for RustLanguageProvider {
    fn language_name(&self) -> &str { "rust" }
    fn file_extensions(&self) -> &[&str] { &["rs"] }
    fn parse_imports(&self, source: &str) -> Vec<Import> { /* line-by-line */ }
    fn extract_symbols(&self, source: &str) -> Vec<Symbol> { /* line-by-line */ }
}
```

**2. Tree-sitter parser** (`TreeSitterRustProvider`): Feature-gated behind `tree-sitter`. Uses the `tree-sitter-rust` grammar for accurate, incremental, error-tolerant parsing [5]. Falls back gracefully on parse errors by extracting whatever symbols the partial AST contains. Recurses into impl bodies to extract methods via a dedicated `collect_impl_methods` function. Handles nested modules.

```rust
// crates/roko-lang-rust/src/tree_sitter_parser.rs

pub struct TreeSitterRustProvider;

impl LanguageProvider for TreeSitterRustProvider {
    fn language_name(&self) -> &str { "rust" }
    fn file_extensions(&self) -> &[&str] { &["rs"] }

    fn parse_imports(&self, source: &str) -> Vec<Import> {
        let Some(tree) = parse_source(source) else { return Vec::new(); };
        let mut imports = Vec::new();
        collect_imports(tree.root_node(), source, &mut imports);
        imports
    }

    fn extract_symbols(&self, source: &str) -> Vec<Symbol> {
        let Some(tree) = parse_source(source) else { return Vec::new(); };
        let mut symbols = Vec::new();
        collect_symbols(tree.root_node(), source, &mut symbols);
        symbols
    }
}
```

The tree-sitter implementation handles AST node types directly:
- `function_item` -> `SymbolKind::Function`
- `struct_item` -> `SymbolKind::Struct`
- `enum_item` -> `SymbolKind::Enum`
- `trait_item` -> `SymbolKind::Trait`
- `impl_item` -> `SymbolKind::Impl` (with `trait for type` naming)
- `const_item` -> `SymbolKind::Const`
- `type_item` -> `SymbolKind::Type`
- `mod_item` -> `SymbolKind::Module`

The test `heuristic_vs_tree_sitter_parity` verifies that the tree-sitter provider extracts at least as many symbols as the heuristic parser for any given input.

**Key advantages of tree-sitter over heuristic parsing:**

| Capability | Heuristic | Tree-sitter |
|---|---|---|
| Nested definitions | Missed | Captured at correct scope |
| Multi-line signatures | Fragile | Robust |
| Impl method extraction | Not recursive | Recurses via `collect_impl_methods` |
| Scope-aware lookup | Impossible | Natural via AST depth |
| Error recovery (broken code) | Crashes/misparses | Partial tree with ERROR nodes |
| Incremental re-parse (1-line edit) | Full re-parse | Reuse unchanged subtrees |

> **Ref**: `crates/roko-lang-rust/src/lib.rs` (entire file), `crates/roko-lang-rust/src/tree_sitter_parser.rs` (entire file)

### TypeScript/JavaScript Provider

Handles ES module imports (`import ... from`, `import '...'`), CommonJS `require()` calls, and type-only imports (`import type`). Extracts `function`, `class`, `interface`, `type`, `const`, `enum`, and `export default` symbols, including named `export default class` and `export default function` declarations. Maps `class` to `SymbolKind::Struct`, `interface` to `SymbolKind::Trait`.

Three build system implementations: `NpmBuildSystem`, `PnpmBuildSystem`, `YarnBuildSystem`. Detection: npm wins if `package.json` present without `pnpm-lock.yaml` or `yarn.lock`; pnpm wins with `pnpm-lock.yaml`; yarn wins with `yarn.lock`.

> **Ref**: `crates/roko-lang-typescript/src/lib.rs` (entire file)

### Go Provider

Parses single and grouped `import` statements (including aliased, dot, and blank imports). Extracts `func` (including methods with receivers), `type ... struct`, `type ... interface`, `const`, `var`, and grouped `const`/`var` blocks. Uses Go's capitalization convention for visibility: names starting with uppercase are `Public`, lowercase are `Private`.

> **Ref**: `crates/roko-lang-go/src/lib.rs` (entire file)

---

## 5. Indexing Mode 1: Symbol Index

The symbol index is a traditional symbol table. For every source file, it stores the symbol name, kind, visibility, file path, line number, and language.

The `WorkspaceIndex` maintains multiple hash maps for fast lookup:

```rust
// crates/roko-index/src/workspace.rs

pub struct WorkspaceIndex {
    root: PathBuf,
    files_by_path: HashMap<String, SourceFile>,
    file_paths: HashSet<String>,
    imports_by_file: HashMap<String, Vec<Import>>,
    symbols_by_name: HashMap<String, Vec<SymbolInfo>>,
    functions_by_name: HashMap<String, Vec<SymbolInfo>>,
    symbols_by_id: HashMap<SymbolId, SymbolInfo>,
    file_fingerprints: HashMap<String, HdcFingerprint>,
    symbol_fingerprints: HashMap<SymbolId, HdcFingerprint>,
    pagerank_scores: HashMap<SymbolId, f64>,
    graph: SymbolGraph,
}
```

Symbol search supports:

- **Exact name lookup**: `symbols_by_name.get("HashMap")` -- O(1)
- **Keyword search** with case sensitivity, whole-word, and scope (symbols only, files only, or both):

```rust
// crates/roko-index/src/workspace.rs

pub struct KeywordQuery {
    pub text: String,
    pub scope: SearchScope,      // Symbols, Files, Both
    pub case_sensitive: bool,
    pub whole_word: bool,
}
```

Keyword search scores results by match quality (exact match = 1.0, prefix match = 0.95, file match = 0.9, substring = 0.8) plus a PageRank bonus capped at 0.2.

- **Structural search** with filters for kind, visibility, file pattern, caller existence, and minimum PageRank:

```rust
// crates/roko-index/src/workspace.rs

pub struct StructuralQuery {
    pub kind: Option<SymbolKind>,
    pub visibility: Option<Visibility>,
    pub file_pattern: Option<String>,
    pub has_callers: Option<bool>,
    pub min_pagerank: Option<f64>,
}
```

> **Ref**: `crates/roko-index/src/workspace.rs` (lines 26-41, 92-134, 428-555)

---

## 6. Indexing Mode 2: Graph Index (Dependency Graph)

### Data Structure

The dependency graph represents relationships between symbols using dual adjacency lists:

```rust
// crates/roko-index/src/graph.rs

pub struct SymbolGraph {
    nodes: HashSet<SymbolId>,
    forward: HashMap<SymbolId, Vec<(SymbolId, EdgeKind)>>,  // X depends on Y
    reverse: HashMap<SymbolId, Vec<(SymbolId, EdgeKind)>>,  // Y is depended on by X
}
```

Dual adjacency lists enable O(1) lookup in either direction. For ~10K symbols and ~30K edges, memory cost is ~2MB.

### The 5 Edge Types

```rust
// crates/roko-index/src/graph.rs

#[non_exhaustive]
pub enum EdgeKind {
    Calls,       // A calls B (function/method invocation)
    Imports,     // A imports B (use/require/import)
    Implements,  // A implements B (trait/interface)
    Contains,    // A contains B (method in impl block)
    TypeRef,     // A references type B in its signature or body
}
```

Each edge type captures a distinct relationship:

**1. Imports** -- Module A imports from Module B. This is the most reliably extracted edge type because import statements have well-defined syntax in every language.

**2. Calls** -- Function A calls Function B. Extracted by matching `identifier(` patterns against known function names. The `CALL_RE` regex (`\b([A-Za-z_][A-Za-z0-9_]*)\s*\(`) captures call sites.

**3. Implements** -- Struct A implements Trait B. Extracted from `impl Trait for Type` patterns.

**4. Contains** -- Module A contains Function B. Represents hierarchical containment: a method inside an impl block, a function inside a module.

**5. TypeRef** -- Function A references Type B in its signature or body. Extracted by matching `PascalCase` identifiers (`TYPE_REF_RE`: `\b([A-Z][A-Za-z0-9_]*)\b`) against known type names.

> **Ref**: `crates/roko-index/src/graph.rs` (lines 24-42)

### Graph Construction Algorithm

The `build_graph()` function constructs the graph in four phases:

```
Phase 1: Register all symbols as graph nodes.
Phase 2: Build name-to-SymbolId lookup tables for import, call, and type resolution.
         Three separate tables: name_to_ids, function_name_to_ids, type_name_to_ids.
Phase 3: Create import edges -- match last segment of each import path
         against known symbol names. For example, `use std::collections::HashMap`
         extracts "HashMap" and links to any symbol named HashMap.
Phase 4: Infer call and type-reference edges from function bodies.
         For each function symbol, scan the source lines from its definition
         to the next symbol's definition. Match CALL_RE captures against
         function_name_to_ids; match TYPE_REF_RE captures against type_name_to_ids.
```

Edge deduplication uses a `HashSet<(SymbolId, SymbolId, EdgeKind)>` to prevent duplicate edges.

**Complexity**: O(S + F * L), where S = total symbols, F = files, and L = average source lines per file. For a typical 300-file, 5K-symbol codebase, construction completes in under 5ms.

> **Ref**: `crates/roko-index/src/graph.rs` (lines 279-465)

### Graph Operations

The `SymbolGraph` provides these core operations:

```rust
// crates/roko-index/src/graph.rs

impl SymbolGraph {
    pub fn node_count(&self) -> usize;
    pub fn edge_count(&self) -> usize;
    pub fn edge_count_by_kind(&self, kind: EdgeKind) -> usize;
    pub fn neighbors(&self, id: &SymbolId) -> Vec<&SymbolId>;           // forward (dependencies)
    pub fn reverse_neighbors(&self, id: &SymbolId) -> Vec<&SymbolId>;   // reverse (dependents)
    pub fn neighbors_by_kind(&self, id: &SymbolId, kind: EdgeKind) -> Vec<&SymbolId>;
    pub fn reverse_neighbors_by_kind(&self, id: &SymbolId, kind: EdgeKind) -> Vec<&SymbolId>;
    pub fn transitive(&self, start: &SymbolId, max_depth: usize) -> Vec<(SymbolId, usize)>;
}
```

The graph also supports **rkyv serialization** (feature-gated) for zero-copy snapshot persistence:

```rust
impl SymbolGraph {
    pub fn snapshot(&self) -> SymbolGraphSnapshot;
    pub fn from_snapshot(snapshot: &SymbolGraphSnapshot) -> Self;
    pub fn save_rkyv(&self, path: &Path) -> Result<()>;
    pub fn load_rkyv(path: &Path) -> Result<Self>;
}
```

### PageRank Algorithm

PageRank [2] computes structural importance for every symbol in the graph. The core formula:

```
PR(v) = (1 - d) / N + d * SUM( PR(u) / out_degree(u) )
                         for each u that links to v
```

Where `d` = 0.85 (damping factor) and `N` = total nodes. The damping factor models the "random surfer" metaphor: with probability `d` (85%) the surfer follows an edge; with probability `1-d` (15%) the surfer teleports to a uniformly random node.

Implementation (verbatim from source):

```rust
// crates/roko-index/src/graph.rs

pub fn pagerank(
    graph: &SymbolGraph,
    iterations: u32,    // typically 30
    damping: f64,       // typically 0.85
) -> HashMap<SymbolId, f64> {
    let n_f = n as f64;
    // Initialize: every node gets 1/N
    let mut rank: HashMap<SymbolId, f64> = all_nodes.iter()
        .map(|id| ((*id).clone(), 1.0 / n_f)).collect();

    for _ in 0..iterations {
        let base = (1.0 - damping) / n_f;
        for &node in &all_nodes {
            let mut incoming_sum = 0.0;
            if let Some(inbound) = graph.reverse.get(node) {
                for (src, _) in inbound {
                    let src_rank = rank.get(src).copied().unwrap_or(0.0);
                    let out_degree = graph.forward.get(src).map_or(1, Vec::len).max(1) as f64;
                    incoming_sum += src_rank / out_degree;
                }
            }
            new_rank.insert(node.clone(), damping.mul_add(incoming_sum, base));
        }
        rank = new_rank;
    }
    rank
}
```

**Convergence**: The power iteration converges geometrically with rate `d = 0.85`. After 30 iterations the error is bounded by `d^30 < 0.008`. For 5K nodes, computation takes roughly 1ms.

**What PageRank captures**:

| Pattern | Typical rank | Why |
|---|---|---|
| Core types (Config, Error, Signal) | Top 1% | Imported everywhere |
| Trait definitions | Top 5% | Implemented by many types |
| Entry points (main, run) | Top 15% | High out-degree but also referenced |
| Module-internal helpers | Bottom 50% | Few external imports |
| Dead code | Bottom 5% | Zero in-links |

> **Ref**: `crates/roko-index/src/graph.rs` (lines 589-622)

### Weighted PageRank

Weighted PageRank assigns different weights to different edge types, reflecting the intuition that an import relationship is more semantically significant than a type reference:

```rust
// crates/roko-index/src/graph.rs

fn edge_weight(kind: &EdgeKind) -> f64 {
    match kind {
        EdgeKind::Imports    => 1.0,
        EdgeKind::Calls      => 0.8,
        EdgeKind::Implements => 0.9,
        EdgeKind::Contains   => 0.6,
        EdgeKind::TypeRef    => 0.5,
    }
}
```

The weighted variant replaces uniform out-degree with the sum of edge weights:

```
WPR(v) = (1 - d) / N + d * SUM( WPR(u) * w(u,v) / weighted_out_degree(u) )
```

Where `weighted_out_degree(u) = SUM(w(u, target))` for all outgoing edges from u.

```rust
pub fn weighted_pagerank(
    graph: &SymbolGraph,
    damping: f64,
    iterations: u32,
) -> HashMap<SymbolId, f64>;
```

> **Ref**: `crates/roko-index/src/graph.rs` (lines 624-693)

### Personalized PageRank (PPR)

PPR [3] replaces uniform teleportation with a biased distribution. Instead of jumping to any random node with probability `(1-d)/N`, the random surfer teleports only to task-relevant seed nodes:

```
PPR(v) = (1 - d) * teleport(v) + d * SUM( PPR(u) * w(u,v) / weighted_out_degree(u) )

where teleport(v) = 1/|seeds|  if v is a seed node
                   = 0          otherwise
```

This biases the entire ranking toward symbols structurally close to the current task context.

```rust
// crates/roko-index/src/graph.rs

pub fn personalized_pagerank(
    graph: &SymbolGraph,
    seed_nodes: &[SymbolId],   // task-relevant symbols
    damping: f64,
    iterations: u32,
) -> HashMap<SymbolId, f64>;
```

Verified by tests:
- Seed node gets higher rank than non-seed nodes in star topology
- Hub with many inbound edges still ranks highly even when not a seed
- Multiple seeds both rank higher than non-seeds
- Empty seeds still runs without panic

> **Ref**: `crates/roko-index/src/graph.rs` (lines 709-772)

---

## 7. Indexing Mode 3: HDC Index (Hyperdimensional Computing Fingerprints)

Each function and symbol gets a 10,240-bit binary fingerprint that encodes its kind, name, and context. Similar code produces similar fingerprints, enabling fast similarity search via Hamming distance. Comparison is pure bitwise XOR + popcount and completes well under 1 microsecond.

### The HDC Algebra

Hyperdimensional computing (HDC), also known as Vector Symbolic Architectures (VSA), uses high-dimensional binary or bipolar vectors as distributed representations [4]. The roko implementation uses three operations:

| Operation | Symbol | Implementation | What it does |
|---|---|---|---|
| **Bind** | XOR | `a[i] ^ b[i]` | Associates two concepts (role-filler binding) |
| **Bundle** | Majority vote | Bit is 1 if > 50% of inputs have it set | Creates set superposition |
| **Permute** | Bit rotation | `rotate_left(n)` | Creates ordered sequences (not yet used for code) |

```rust
// crates/roko-index/src/hdc.rs

const WORDS: usize = 160;       // 10,240 / 64 = 160 u64 words
const TOTAL_BITS: usize = WORDS * 64;  // 10,240 bits

pub struct HdcFingerprint {
    bits: [u64; WORDS],         // 1,280 bytes per fingerprint
}
```

### Deterministic Vector Generation

Random-like base vectors are generated deterministically from byte seeds using FNV-1a hashing followed by splitmix64 PRNG expansion:

```rust
// crates/roko-index/src/hdc.rs

fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;   // FNV offset basis
    for &byte in data {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3); // FNV prime
    }
    if hash == 0 { hash = 0xA5A5_A5A5_5A5A_5A5A; } // avoid zero seed
    hash
}

fn vector_from_seed(seed: &[u8]) -> [u64; WORDS] {
    let mut state = fnv1a(seed);
    let mut bits = [0u64; WORDS];
    for word in &mut bits {
        *word = splitmix64(&mut state);
    }
    bits
}
```

Deterministic generation means the same seed always produces the same 10,240-bit vector. No randomness, no model dependency.

### Fingerprint Composition

Each symbol's fingerprint encodes three properties:

**1. Role vector** -- deterministic base vector for each `SymbolKind`:

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

Role vectors are near-orthogonal by the high-dimensional quasi-orthogonality property -- random 10,240-bit vectors have expected Hamming distance of exactly 5,120 (50%).

**2. Name vector** -- encoded via overlapping character trigrams:

```rust
fn encode_name(name: &str) -> [u64; WORDS] {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() < 3 { return vector_from_seed(name.as_bytes()); }
    let trigrams: Vec<[u64; WORDS]> = chars.windows(3)
        .map(|w| {
            let trigram: String = w.iter().collect();
            vector_from_seed(trigram.as_bytes())
        })
        .collect();
    bundle(&trigrams)
}
```

For `process_input`: trigrams are `pro`, `roc`, `oce`, `ces`, `ess`, `ss_`, `s_i`, `_in`, `inp`, `npu`, `put`. Each becomes a 10,240-bit vector; all are bundled via majority vote.

Properties:
- Similar names produce similar vectors: `process_input` and `process_output` share 7 of 11 trigrams
- Order sensitivity: `abc` and `bca` produce different trigram sets
- Short names (< 3 chars): fall back to direct seed encoding

**3. Context vector** -- seeded from surrounding source text (currently entire file content):

```rust
let ctx_vec = vector_from_seed(context);  // context = file content bytes
```

**Final composition** (verbatim from source):

```rust
pub fn fingerprint_symbol(symbol: &Symbol, context: &[u8]) -> HdcFingerprint {
    let role_vec = role_vector(&symbol.kind);
    let name_vec = encode_name(&symbol.name);
    let ctx_vec = vector_from_seed(context);
    let combined = bundle(&[name_vec, ctx_vec]);
    HdcFingerprint { bits: bind(&role_vec, &combined) }
}
```

The bundle preserves both name and context (superposition). The bind tags the result with the symbol kind (role-filler association).

### File-Level Fingerprints

Entire files are fingerprinted by bundling all symbol fingerprints:

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

### Similarity Measurement

Similarity is computed via normalized Hamming distance:

```rust
impl HdcFingerprint {
    pub fn similarity(&self, other: &Self) -> f64 {
        let dist = hamming_distance(&self.bits, &other.bits);
        1.0 - (f64::from(dist) / TOTAL_BITS as f64)
    }
}

fn hamming_distance(a: &[u64; WORDS], b: &[u64; WORDS]) -> u32 {
    let mut diff = 0u32;
    for (left, right) in a.iter().zip(b.iter()) {
        diff += (left ^ right).count_ones();  // POPCNT instruction
    }
    diff
}
```

Range [0.0, 1.0]: 1.0 = identical, ~0.5 = unrelated (random), 0.0 = maximally different.

### Performance Characteristics

| Operation | Time | Notes |
|---|---|---|
| `vector_from_seed()` | ~200ns | 160 splitmix64 iterations |
| `encode_name()` (15-char) | ~3us | 13 trigrams, 13 vectors, 1 bundle |
| `fingerprint_symbol()` | ~5us | role + name + context + bind + bundle |
| `similarity()` | ~50ns | 160 XOR + POPCNT operations |
| Brute-force scan 5K symbols | ~0.25ms | 5000 x 50ns |
| Brute-force scan 50K symbols | ~2.5ms | May need HNSW for larger |

Verified by tests:
- Identical symbols produce identical fingerprints (similarity = 1.0 exactly)
- Similar names have high similarity: `process_input` vs `process_output` > 0.5
- Different kinds lower similarity: `Config(Function)` vs `Config(Struct)` < 0.9
- Completely different symbols have low similarity: < 0.7
- Self-similarity is exactly 1.0
- 10,000 comparisons complete in under 1ms

### HDC vs Dense Embeddings

| Property | HDC (10,240-bit) | Dense embedding (384-dim float) |
|---|---|---|
| Vector size | 1,280 bytes | 1,536 bytes |
| Computation | ~5us (CPU only) | ~10ms (GPU) or ~100ms (CPU) |
| Similarity op | ~50ns (XOR+POPCNT) | ~500ns (dot product) |
| Structural similarity | Good | Excellent |
| Semantic similarity | Limited | Excellent |
| Model dependency | None | Requires embedding model |
| Incremental update | ~5us per symbol | ~10ms per symbol |

HDC is 200x-20,000x faster than neural embeddings and requires no GPU. Neural embeddings capture semantic meaning that HDC misses. The design uses both: HDC for fast structural matching (always on), embeddings for semantic refinement (feature-gated).

> **Ref**: `crates/roko-index/src/hdc.rs` (entire file, 355 lines)

---

## 8. Indexing Mode 4: FTS5 Index (Full-Text Search)

The SQLite-backed persistent index provides FTS5 full-text search over symbol names and file paths. FTS5 is SQLite's built-in full-text search extension that maintains an inverted index over text columns and ranks results using BM25 (Best Matching 25) [6], a probabilistic ranking function from the Okapi information retrieval system.

### Schema

```sql
-- crates/roko-index/src/sqlite.rs

CREATE TABLE files (
    path     TEXT PRIMARY KEY,
    mtime_ns INTEGER NOT NULL,
    hash     TEXT NOT NULL
);

CREATE TABLE symbols (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path  TEXT NOT NULL,
    name       TEXT NOT NULL,
    kind       TEXT NOT NULL,
    line       INTEGER NOT NULL,
    col        INTEGER NOT NULL DEFAULT 0,
    visibility TEXT NOT NULL DEFAULT 'Private',
    UNIQUE(file_path, name, kind)
);

CREATE TABLE edges (
    from_file TEXT NOT NULL,
    from_name TEXT NOT NULL,
    from_kind TEXT NOT NULL,
    to_file   TEXT NOT NULL,
    to_name   TEXT NOT NULL,
    to_kind   TEXT NOT NULL,
    edge_kind TEXT NOT NULL,
    UNIQUE(from_file, from_name, from_kind, to_file, to_name, to_kind, edge_kind)
);

CREATE VIRTUAL TABLE symbols_fts USING fts5(name, file_path, kind, sym_id);
```

### FTS Search

```rust
// crates/roko-index/src/sqlite.rs

pub fn fts_search(&self, query: &str) -> Result<Vec<SymbolInfo>> {
    let safe_query = query.replace('"', "\"\"");
    let fts_query = format!("\"{safe_query}\"*");   // prefix matching
    // SELECT s.file_path, s.name, s.kind, s.line, s.visibility
    // FROM symbols_fts fts
    // JOIN symbols s ON s.id = CAST(fts.sym_id AS INTEGER)
    // WHERE symbols_fts MATCH ?1
    // ORDER BY rank LIMIT 100
}
```

FTS5 uses BM25 ranking by default. The `rank` column is a hidden FTS5 column that contains the BM25 relevance score (note: FTS5 BM25 scores are negative, with more negative = more relevant; the `ORDER BY rank` clause naturally sorts most-relevant first). The query is wrapped with prefix matching (`"query"*`) for fuzzy-match behavior.

### Incremental Updates

The `incremental_update` method checks file modification times and only re-indexes changed files:

```rust
pub fn incremental_update<F>(
    &self,
    changed_files: &[PathBuf],
    mut index_file: F,
) -> Result<UpdateStats>
where
    F: FnMut(&Path) -> Result<(Vec<SymbolInfo>, Vec<(SymbolId, SymbolId, EdgeKind)>)>,
```

For each file: check `mtime_ns` against stored value. If unchanged, skip. If changed, delete stale symbols and edges, re-index, update file record with blake3 hash. All within a single transaction (`unchecked_transaction`) for atomicity.

The database uses WAL mode for concurrent reads and `synchronous=NORMAL` for performance.

> **Ref**: `crates/roko-index/src/sqlite.rs` (entire file, 500 lines)

---

## 9. Hybrid Search with Reciprocal Rank Fusion (RRF)

When a query comes in, multiple search strategies run in parallel. The `SearchStrategy` enum defines the options:

```rust
// crates/roko-index/src/workspace.rs

pub enum SearchStrategy {
    Keyword(KeywordQuery),
    Structural(StructuralQuery),
    Hdc(HdcQuery),
    Hybrid {
        keyword: Option<KeywordQuery>,
        structural: Option<StructuralQuery>,
        hdc: Option<HdcQuery>,
    },
}
```

For `Hybrid`, results from each sub-strategy are merged using RRF.

### The RRF Algorithm

RRF [1] combines ranked lists by assigning each result a score based on its rank position in each list:

```
RRF_score(symbol) = SUM_over_all_lists( 1 / (k + rank_i(symbol)) )
```

Where `k` = 60 (the standard constant from the original paper). The key insight: RRF is rank-based, not score-based. It does not require normalizing scores across different search strategies. A symbol at rank 1 in one list and rank 3 in another gets `1/(60+1) + 1/(60+3) = 1/61 + 1/63 = 0.0164 + 0.0159 = 0.0323`. A symbol appearing in only one list at rank 1 gets `1/61 = 0.0164`. The multi-list symbol wins.

Note that the implementation uses 1-based ranks (`rank + 1`), matching the convention in the original paper.

Implementation (verbatim from source):

```rust
// crates/roko-index/src/workspace.rs

fn rrf_merge(lists: &[Vec<SearchResult>], k: f64, limit: usize) -> Vec<SearchResult> {
    let mut scores: HashMap<SymbolId, (f64, SearchResult)> = HashMap::new();
    for list in lists {
        for (rank, result) in list.iter().enumerate() {
            let rrf_score = 1.0 / (k + (rank + 1) as f64);
            match scores.entry(result.symbol.id.clone()) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().0 += rrf_score;   // accumulate
                }
                Entry::Vacant(entry) => {
                    entry.insert((rrf_score, result.clone()));
                }
            }
        }
    }
    let mut merged: Vec<SearchResult> = scores.into_values()
        .map(|(rrf_score, mut result)| { result.score = rrf_score; result })
        .collect();
    sort_search_results(&mut merged);
    merged.truncate(limit);
    merged
}
```

The unified search method handles oversampling -- it requests 3x the limit from each sub-strategy (minimum 30) before merging:

```rust
pub fn search(&self, strategy: SearchStrategy, limit: usize) -> Vec<SearchResult> {
    match strategy {
        SearchStrategy::Hybrid { keyword, structural, hdc } => {
            let oversample = limit.saturating_mul(3).max(30);
            let mut lists: Vec<Vec<SearchResult>> = Vec::new();
            if let Some(q) = keyword { lists.push(self.keyword_search(&q, oversample)); }
            if let Some(q) = structural { lists.push(self.structural_search(&q, oversample)); }
            if let Some(q) = hdc {
                let mut q = q;
                q.max_results = oversample;
                lists.push(self.hdc_search(&q));
            }
            rrf_merge(&lists, 60.0, limit)
        }
        // ... single-strategy cases
    }
}
```

> **Ref**: `crates/roko-index/src/workspace.rs` (lines 600-634, 1430-1464)

### Worked Example: Query "process_input error handling"

Suppose the agent receives the task "add error handling to `process_input`" and a workspace with the following symbols:

```
A = fn process_input (lib.rs:45)
B = fn validate_input (lib.rs:78)
C = struct InputError (error.rs:12)
D = fn handle_error (error.rs:30)
E = trait ErrorHandler (error.rs:5)
```

**Step 1: Keyword search** for "process_input error" returns:
```
rank 1: A (process_input -- exact match on first word)
rank 2: C (InputError -- contains "Error")
rank 3: D (handle_error -- contains "error")
```

**Step 2: Structural search** filtering for functions with callers, sorted by PageRank:
```
rank 1: A (process_input -- high PageRank, many callers)
rank 2: D (handle_error -- moderate PageRank)
rank 3: B (validate_input -- called by process_input)
```

**Step 3: HDC similarity search** against a query fingerprint built from "process_input error handling":
```
rank 1: D (handle_error -- name trigrams overlap with "handling")
rank 2: A (process_input -- name trigrams overlap with "process_input")
rank 3: E (ErrorHandler -- name trigrams overlap with "error" and "handling")
```

**Step 4: RRF merge** with k=60:

```
A: 1/(60+1) + 1/(60+1) + 1/(60+2)  = 0.01639 + 0.01639 + 0.01613  = 0.04891  <-- HIGHEST
D: 1/(60+3) + 1/(60+2) + 1/(60+1)  = 0.01587 + 0.01613 + 0.01639  = 0.04839
C: 1/(60+2)                         = 0.01613
B: 1/(60+3)                         = 0.01587
E: 1/(60+3)                         = 0.01587

Result order: A > D > C > B = E
```

`A` (process_input) wins because it appears in all three result lists. `D` (handle_error) ranks second because it also appears in all three. `C`, `B`, and `E` appear in only one list each.

**Step 5: Graph expansion** (depth=1) from top results:
- From A: callees include B (validate_input). Type refs include C (InputError).
- From D: type refs include C (InputError). Implements E (ErrorHandler).

**Step 6: Slice and budget**: Extract code slices for A, D, B, C, E. Estimate tokens. Fit within 5,000-token budget. If budget exceeded, drop lowest-ranked symbols first.

**Final context**: 5 focused code slices totaling ~3,000 tokens, containing exactly the target function, its validation helper, the error types, and the error handler. No irrelevant code.

---

## 10. Privacy and Context Overlays

The index supports two overlay mechanisms that customize what symbols are included in assembled context.

### Context Overlay (Per-Agent View)

```rust
// crates/roko-index/src/workspace.rs

pub struct ContextOverlay {
    pub pinned_files: Vec<String>,          // Files that should always be preferred
    pub excluded_patterns: Vec<String>,     // File patterns to exclude
    pub importance_overrides: HashMap<SymbolId, f64>,  // Boost/demote specific symbols
    pub max_expansion_depth: usize,         // Maximum graph expansion depth
}
```

Different agents may need different views of the same codebase. A testing agent might pin test files and exclude implementation internals. A security agent might focus on auth-related symbols. Context overlays enable this without re-indexing.

### Privacy Config (Redaction)

```rust
// crates/roko-index/src/workspace.rs

pub struct PrivacyConfig {
    pub redact_patterns: Vec<String>,    // String patterns to redact (e.g., API keys)
    pub ignore_files: Vec<String>,       // Files to exclude entirely (e.g., ".env")
    pub blocked_symbols: Vec<String>,    // Symbol names to exclude
}
```

Privacy redaction happens after search/ranking but before context assembly. Sensitive data is never sent to the LLM, even if it appears in the index. The `apply_privacy()` function replaces matching patterns with `[REDACTED]` in code slices and the `is_excluded_symbol` / `is_excluded_file` predicates filter results.

> **Ref**: `crates/roko-index/src/workspace.rs` (lines 200-222)

---

## 11. Context Assembly Pipeline

The `WorkspaceIndex` implements the `CodeIndex` trait, which provides the full suite of code intelligence queries:

```rust
// crates/roko-index/src/workspace.rs

pub trait CodeIndex {
    fn lookup_symbol(&self, name: &str) -> Vec<SymbolInfo>;
    fn search_by_keyword(&self, query: &KeywordQuery, limit: usize) -> Vec<SearchResult>;
    fn search_by_structure(&self, query: &StructuralQuery, limit: usize) -> Vec<SearchResult>;
    fn search_by_fingerprint(&self, query: &HdcQuery) -> Vec<SearchResult>;
    fn search_by_embedding(&self, query: &EmbeddingQuery) -> Vec<SearchResult>;
    fn list_imports_for_file(&self, file: &str) -> Result<Vec<Import>>;
    fn build_symbol_context(&self, name: &str, file: Option<&str>, depth: usize)
        -> Result<Vec<SymbolContext>>;
    fn find_call_graph(&self, function: &str, depth: u32) -> CallGraph;
    fn file_ast(&self, file: &str) -> Result<FileAst>;
    fn index_stats(&self) -> IndexStats;
    fn find_references(&self, name: &str, file: Option<&str>, include_defs: bool)
        -> Result<Vec<ReferenceMatch>>;
    fn find_implementations(&self, trait_name: &str) -> Vec<ImplementationMatch>;
    fn workspace_map(&self, focus: Option<&str>) -> WorkspaceMap;
    fn assemble_context(&self, query: &str, max: usize, budget: usize,
                        overlay: Option<&ContextOverlay>, privacy: Option<&PrivacyConfig>)
        -> AssembledContext;
}
```

### The 6-Step Assembly Pipeline

The `context_for_query` method implements a 6-step pipeline:

```
Step 1: PARSE QUERY       Extract search terms, intent, focal symbols
Step 2: MULTI-STRATEGY    Run keyword + HDC semantic search in parallel
Step 3: RANK (RRF)        Merge, deduplicate, sort by combined score
Step 4: EXPAND GRAPH      Add graph neighbors of top results (configurable depth)
Step 5: SLICE             Extract relevant code fragments (not whole files)
Step 6: BUDGET            Fit into token budget, prioritizing by rank;
                          set truncated=true if budget exceeded
```

The output is an `AssembledContext`:

```rust
pub struct AssembledContext {
    pub query: String,
    pub slices: Vec<CodeSlice>,
    pub token_estimate: usize,
    pub truncated: bool,        // true if budget was exceeded
}

pub struct CodeSlice {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub symbols_included: Vec<SymbolId>,
    pub token_estimate: usize,  // ~4 chars per token heuristic
}
```

### Semantic Search (HDC-Powered)

HDC-powered semantic search creates a fingerprint from the query text and compares against all indexed symbols:

```rust
pub fn semantic_search(&self, query: &str, limit: u32) -> Vec<SearchResult> {
    let query_file = SourceFile {
        path: "<query>".to_string(),
        content: query.to_string(),
        language: "query".to_string(),
        symbols: Vec::new(),
        imports: Vec::new(),
    };
    let query_fp = fingerprint_file(&query_file);

    // Score = 0.7 * similarity(query, symbol_fp) + 0.3 * similarity(query, file_fp)
    // This blend of symbol-level and file-level similarity captures both
    // direct matches and contextual relevance
}
```

The 0.7/0.3 weighting favors direct symbol-level matches while still accounting for file-level contextual similarity.

### Call Graph Queries

The `call_graph()` method traverses the dependency graph in both directions from a function name:

```rust
pub struct CallGraph {
    pub function: String,
    pub depth: usize,
    pub roots: Vec<SymbolInfo>,      // Matching functions
    pub callers: Vec<SymbolInfo>,    // Who calls this function (BFS up to depth)
    pub callees: Vec<SymbolInfo>,    // What this function calls (BFS up to depth)
    pub edges: Vec<CallGraphEdge>,   // Traversed edges with direction and depth
}
```

### Symbol Context

The `symbol_context()` method bundles rich context for a symbol:

```rust
pub struct SymbolContext {
    pub symbol: SymbolInfo,
    pub pagerank: f64,
    pub imports: Vec<Import>,
    pub dependencies: Vec<SymbolInfo>,    // Forward graph neighbors
    pub callers: Vec<SymbolInfo>,         // Reverse graph neighbors
    pub definition: Option<CodeSlice>,    // Source code slice
}
```

> **Ref**: `crates/roko-index/src/workspace.rs` (lines 350-410, 600-760, 928-999)

---

## 12. Workspace Index Construction

The `WorkspaceIndex::load()` method builds a complete index from a directory:

```rust
pub fn load(root: impl AsRef<Path>) -> Result<Self> {
    let root = std::fs::canonicalize(root)?;
    let files = collect_source_files(&root)?;
    Ok(Self::from_source_files_with_root(root, files))
}
```

The `from_source_files_with_root` constructor:

1. Builds the dependency graph: `let graph = build_graph(&files);`
2. Runs PageRank: `let pagerank_scores = pagerank(&graph, 30, 0.85);`
3. Populates all lookup maps: `files_by_path`, `symbols_by_name`, `functions_by_name`, `symbols_by_id`
4. Computes all fingerprints: `file_fingerprints`, `symbol_fingerprints` (one per file, one per symbol)

File collection uses the language provider registry: files with `.rs` extensions get `RustLanguageProvider`, `.ts/.tsx/.js/.jsx` get `TypeScriptLanguageProvider`, `.go` gets `GoLanguageProvider`.

Static provider instances avoid allocation:

```rust
static RUST_PROVIDER: RustLanguageProvider = RustLanguageProvider;
static TS_PROVIDER: TypeScriptLanguageProvider = TypeScriptLanguageProvider;
static GO_PROVIDER: GoLanguageProvider = GoLanguageProvider;
```

> **Ref**: `crates/roko-index/src/workspace.rs` (lines 22-24, 435-445)

---

## 13. Empirical Token Savings

### Without intelligence
Agent tasked with "add error handling to `process_input`":
1. grep-like search -> 20-50 candidate files
2. Include full files -> ~50K tokens
3. LLM must identify the right function and dependencies
4. Risk missing callers, trait implementations, type definitions

### With intelligence
Same agent:
1. Keyword search -> 1 result (5ms)
2. Graph expansion -> 3 deps, 7 callers (1ms)
3. Code slicing -> 11 focused slices, ~5K tokens
4. Context includes exactly the function, dependencies, and callers

**Result: 10x fewer tokens, no missed dependencies.**

### Impact analysis scenario
"What breaks if I change the `Verdict` type?"

Without: `grep -rn "Verdict"` -> 47 files, ~150K tokens. No structural understanding.

With: `reverse_neighbors(Verdict)` -> 12 direct dependents. `transitive(depth=2)` -> 23 total. Code slices -> ~2K tokens. Structured impact report by relationship type.

**Result: 75x fewer tokens, structured understanding.**

---

## 14. IronClaw Integration Plan

### Existing Infrastructure

IronClaw already has several of the building blocks needed for code intelligence:

1. **Hybrid search with RRF** in `src/workspace/search.rs`: The workspace already implements both Reciprocal Rank Fusion and weighted score fusion for combining FTS and vector search results. The `SearchConfig` struct mirrors roko-index's approach with `rrf_k: u32`, `use_fts`, `use_vector`, and `fusion_strategy` fields. The `reciprocal_rank_fusion()` function uses the identical `1/(k + rank)` formula.

2. **Dual-backend persistence** in `src/db/`: Both PostgreSQL (with pgvector) and libSQL (with libsql_vector) provide full-text search and vector similarity search. New code intelligence tables would follow the same dual-backend pattern.

3. **Embedding infrastructure** in `crates/ironclaw_embeddings/`: Multi-provider embedding support (OpenAI, fastembed, Anthropic Voyager) for computing dense vector representations of text. This could be extended to embed code symbols.

4. **Tool system** in `src/tools/`: The `memory_search`, `memory_write`, `memory_read`, and `memory_tree` tools provide the agent-facing API pattern. Code intelligence tools would follow the same `Tool` trait implementation.

### Integration Architecture

```
crates/
  ironclaw_code_index/           # New extracted crate (mirrors roko-index)
    src/
      lib.rs                     # Public API, re-exports
      parser.rs                  # LanguageProvider trait, SourceFile
      symbol.rs                  # SymbolId, SymbolRef
      graph.rs                   # SymbolGraph, PageRank
      hdc.rs                     # HDC fingerprints
      search.rs                  # Hybrid search with RRF

src/
  tools/builtin/
    code_search.rs               # New: code_search tool (keyword + HDC + graph)
    code_graph.rs                # New: code_graph tool (call graph, references)
    code_symbols.rs              # New: code_symbols tool (workspace map, file AST)
```

### Mapping to Existing Modules

| roko-index Component | IronClaw Mapping | Notes |
|---|---|---|
| `WorkspaceIndex::load()` | Project indexing on file open/change | Use `notify` watcher for incremental updates |
| `CodeIndex` trait | `src/tools/builtin/code_*.rs` | Each CodeIndex method becomes a tool parameter mode |
| `rrf_merge()` | `src/workspace/search.rs::reciprocal_rank_fusion()` | Already implemented for memory search |
| `SqliteIndex` | `src/db/` (libSQL backend) | Extend existing migration system with code tables |
| `SearchStrategy::Hybrid` | `SearchConfig { use_fts, use_vector }` | Add HDC as third search dimension |
| `ContextOverlay` | Engine v2 per-project sandbox context | Maps to project-level symbol pinning |
| `PrivacyConfig` | `src/workspace/privacy.rs` | Already has redaction infrastructure |
| `AssembledContext` | Engine v2 context assembly | Budget-fit code slices for LLM prompts |

### New Tool Specifications

**`code_search`** -- Hybrid code search across the project's source files:
```json
{
  "name": "code_search",
  "parameters": {
    "query": "process_input error handling",
    "strategy": "hybrid",          // "keyword", "structural", "hdc", "hybrid"
    "scope": "both",               // "symbols", "files", "both"
    "limit": 20,
    "kind_filter": "Function",     // optional SymbolKind filter
    "file_pattern": "src/**/*.rs"  // optional glob
  }
}
```

**`code_graph`** -- Call graph and reference queries:
```json
{
  "name": "code_graph",
  "parameters": {
    "function": "process_input",
    "mode": "call_graph",          // "call_graph", "references", "implementations", "impact"
    "depth": 2
  }
}
```

**`code_symbols`** -- Workspace structure queries:
```json
{
  "name": "code_symbols",
  "parameters": {
    "mode": "file_ast",            // "file_ast", "workspace_map", "stats", "symbol_context"
    "file": "src/main.rs"
  }
}
```

### Implementation Phases

**Phase 1: Core Index** (aligns with `crates/ironclaw_code_index/`)
- Port the `LanguageProvider` trait and `RustLanguageProvider`
- Port `SymbolGraph`, `build_graph()`, `pagerank()`
- Port HDC fingerprinting
- Wire into project sandbox file watcher for incremental indexing

**Phase 2: Search & Tools** (aligns with `src/tools/builtin/`)
- Implement `code_search`, `code_graph`, `code_symbols` tools
- Connect to existing `ToolRegistry` and `ToolDispatcher`
- Add FTS5 persistence to libSQL backend using existing migration system

**Phase 3: Context Assembly** (aligns with engine v2)
- Implement `assemble_context()` for LLM prompt construction
- Integrate with engine v2 per-project sandbox context
- Connect `PrivacyConfig` to existing `src/workspace/privacy.rs`
- Add `ContextOverlay` support per agent/session

---

## 15. References

[1] G. V. Cormack, C. L. A. Clarke, and S. Buettcher. "Reciprocal Rank Fusion Outperforms Condorcet and Individual Rank Learning Methods." In *Proceedings of the 32nd International ACM SIGIR Conference on Research and Development in Information Retrieval* (SIGIR '09), pp. 758-759. ACM, 2009. https://doi.org/10.1145/1571941.1572114

[2] S. Brin and L. Page. "The Anatomy of a Large-Scale Hypertextual Web Search Engine." In *Proceedings of the 7th International World Wide Web Conference*, pp. 107-117. Brisbane, Australia, 1998. http://infolab.stanford.edu/pub/papers/google.pdf

[3] R. Andersen, F. Chung, and K. Lang. "Local Graph Partitioning Using PageRank Vectors." In *Proceedings of the 47th Annual IEEE Symposium on Foundations of Computer Science* (FOCS '06), pp. 475-486. IEEE, 2006. (Describes the push-based approximate Personalized PageRank algorithm.)

[4] D. Kleyko, D. Rachkovskij, E. Osipov, and A. Rahimi. "A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures, Part I: Models and Data Transformations." *ACM Computing Surveys*, 55(6), Article 130. 2023. https://doi.org/10.1145/3538531

[5] M. Brunsfeld et al. "Tree-sitter: An Incremental Parsing System for Programming Tools." https://tree-sitter.github.io/tree-sitter/. Based on incremental LR parsing research by T. A. Wagner and S. L. Graham, "Efficient and Flexible Incremental Parsing," *ACM Transactions on Programming Languages and Systems*, 20(5), 980-1013, 1998.

[6] S. Robertson and H. Zaragoza. "The Probabilistic Relevance Framework: BM25 and Beyond." *Foundations and Trends in Information Retrieval*, 3(4), 333-389, 2009. (The theoretical basis for FTS5's default ranking function.)

[7] Y. Xie, J. Lin, H. Dong, L. Zhang, and Z. Wu. "A Survey of Source Code Search: A 3-Dimensional Perspective." *arXiv preprint arXiv:2311.07107*, 2023. (Comprehensive survey of code search techniques including graph-based, embedding-based, and hybrid approaches.)
