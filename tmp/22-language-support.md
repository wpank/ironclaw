# Roko Multi-Language Code Analysis System

## The Multi-Language Analysis Problem

An AI coding agent that treats source code as raw text is fundamentally limited. It cannot answer "what calls this function?", "which files depend on this module?", or "what are the public exports of this package?" without scanning every file in the project -- a process that exhausts context windows and wastes tokens on irrelevant content. Research on retrieval-augmented code generation confirms that naive full-file inclusion wastes 80-99% of the context budget on code irrelevant to the current task [1].

The problem is compounded in polyglot codebases. A Rust backend with a TypeScript frontend, Solidity smart contracts with TypeScript test suites, or a Go service with Python scripts -- each language has its own import syntax, visibility rules, build toolchain, and naming conventions. An agent that understands Rust's `use` statements but not TypeScript's `import ... from` or Go's capitalization-based visibility is only partially useful. Empirical studies of 414,486 GitHub repositories show that multi-language development is the norm, not the exception -- developers regularly combine 2-4 languages within a single project [2].

Roko solves this with a **structural code analysis system** that parses source code into typed symbols and dependency edges, then uses graph algorithms (PageRank), hyperdimensional fingerprints (HDC), and multi-strategy search to assemble precisely-targeted context for LLM consumption. The system is designed around two core abstractions -- `BuildSystem` and `LanguageProvider` -- that isolate all language-specific knowledge into small, self-contained crates, while the analysis engine (`roko-index`) operates entirely on language-neutral data structures.

This document covers every layer of that system: the core trait contracts, the three language provider implementations (Rust, TypeScript, Go), the dual-mode Rust parser (heuristic vs. tree-sitter), the polyglot project detection pipeline, and a concrete implementation plan for how IronClaw could consume this analysis to enhance its code-aware tool execution.

---

## Architecture Overview

The system is organized into four layers, with a strict dependency direction: each layer depends only on layers below it.

```
Layer 1: Core Abstractions (roko-core)
  - BuildSystem trait        (crates/roko-core/src/build.rs)
  - LanguageProvider trait   (crates/roko-core/src/language.rs)
  - Language / DetectedBuildSystem enums (crates/roko-core/src/project.rs)
  - PolyglotProject detection (crates/roko-core/src/polyglot.rs)

Layer 2: Language Providers (roko-lang-*)
  - roko-lang-rust           (crates/roko-lang-rust/src/lib.rs)
    - CargoBuildSystem
    - RustLanguageProvider (heuristic)
    - TreeSitterRustProvider (tree-sitter, feature-gated)
  - roko-lang-typescript     (crates/roko-lang-typescript/src/lib.rs)
    - NpmBuildSystem, PnpmBuildSystem, YarnBuildSystem
    - TypeScriptLanguageProvider
  - roko-lang-go             (crates/roko-lang-go/src/lib.rs)
    - GoBuildSystem
    - GoLanguageProvider

Layer 3: Analysis Engine (roko-index)
  - parser.rs   -- SourceFile, parse_source()
  - symbol.rs   -- SymbolId, SymbolRef, find_symbol()
  - graph.rs    -- SymbolGraph, build_graph(), pagerank()
  - hdc.rs      -- HdcFingerprint, fingerprint_symbol(), similarity()

Layer 4: Consumption
  - roko-compose (prompt assembly / context budgeting)
  - MCP context server (agent-facing tools)
  - roko-gate verify cells (CompileGate, ClippyGate, TestGate)
```

The critical design property is that **language knowledge never leaks upward**. The graph builder, PageRank scorer, fingerprint generator, and search layer all operate on `Symbol`, `Import`, and `SourceFile` -- they never see Rust-specific syntax, TypeScript module resolution, or Go package conventions. Adding a new language means implementing two traits (`LanguageProvider` and `BuildSystem`); every downstream component works unchanged.

This design follows the principle of ad-hoc polymorphism through trait-based dispatch -- the same pattern as Haskell's type classes, where each language implementation provides its own "instance" of a shared interface [3]. Rust's trait system enforces this at compile time: the `Send + Sync` bounds on both traits guarantee that language providers can be shared across threads for parallel file parsing.

---

## Core Data Types

Before examining the traits, it is important to understand the data types that flow through the system. All are defined in `roko_core::language`.

**Source**: `crates/roko-core/src/language.rs`

### ImportKind

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ImportKind {
    /// Rust `use` / TypeScript `import` / Python `import` / Go `import`.
    Use,
    /// Rust `mod` declaration.
    Mod,
    /// Rust `extern crate`.
    ExternCrate,
}
```

`ImportKind` classifies import statements. Most languages use only `ImportKind::Use`. Rust is the exception because `mod` declarations and `extern crate` statements have different semantics than `use` imports -- `mod` creates a new scope, `extern crate` brings an external dependency into scope at the crate level. The `#[non_exhaustive]` attribute allows adding new variants (e.g., `ImportKind::Reexport` or `ImportKind::Dynamic` for JavaScript's `import()` expressions) without breaking downstream consumers.

### Import

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Import {
    /// The import path (e.g. `"std::collections::HashMap"`).
    pub path: String,
    /// Optional alias (`as` rename).
    pub alias: Option<String>,
    /// What kind of import this is.
    pub kind: ImportKind,
}
```

An `Import` captures one dependency edge from the current file to another module or symbol. The `path` field is the raw path string as written in source code -- `"react"` for `import React from 'react'`, `"std::collections::HashMap"` for `use std::collections::HashMap`. The `alias` captures renames: `Some("React")` for a default import, `Some("IoResult")` for `use std::io::Result as IoResult`.

### SymbolKind

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
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
```

`SymbolKind` normalizes language-specific constructs into a universal taxonomy. The cross-language mapping is intentionally lossy -- it captures structural role rather than language-specific semantics:

| Rust | TypeScript/JavaScript | Go | SymbolKind |
|------|----------------------|-----|------------|
| `fn`, `async fn`, `unsafe fn`, `const fn`, `extern "C" fn` | `function`, `async function`, `function*` | `func` (including methods) | `Function` |
| `struct` | `class`, `abstract class` | `type X struct` | `Struct` |
| `enum` | `enum`, `const enum` | -- | `Enum` |
| `trait` | `interface` | `type X interface` | `Trait` |
| `const` | `const` | `const`, `var` | `Const` |
| `type` alias | `type` alias | `type` (non-struct/interface) | `Type` |
| `mod` | `export default` (bare identifier) | -- | `Module` |
| `impl` | -- | -- | `Impl` |

This mapping means a Rust `trait` and a Go `interface` both produce `SymbolKind::Trait` nodes in the dependency graph, enabling structural comparison across languages via HDC fingerprints. A TypeScript `class` maps to `SymbolKind::Struct` (not `Class`) because it fills the same structural role as a Rust `struct` -- a named type with fields and methods.

### Visibility

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    /// Publicly visible (`pub`).
    Public,
    /// Private / crate-private.
    Private,
}
```

Each language maps its own visibility rules:
- **Rust**: `pub` / `pub(crate)` / `pub(super)` / `pub(in path)` are all `Public`; otherwise `Private`
- **TypeScript**: `export` prefix is `Public`; otherwise `Private`
- **Go**: Capitalized first letter is `Public`; lowercase is `Private`

The simplification to a binary Public/Private is deliberate. Finer-grained visibility (e.g., Rust's `pub(crate)` vs. `pub(super)`) is important for the language compiler but rarely matters for the structural analysis use cases: context assembly, symbol search, and dependency ranking.

### Symbol

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    /// Symbol name (e.g. `"HashMap"`, `"main"`).
    pub name: String,
    /// What kind of symbol this is.
    pub kind: SymbolKind,
    /// Whether the symbol is public or private.
    pub visibility: Visibility,
    /// 1-based line number where the symbol is defined.
    pub line: usize,
}
```

A `Symbol` is a named definition extracted from source code. Its identity in the analysis engine is the triple `(file_path, name, kind)` -- captured in `SymbolId` within `roko-index`. The `line` field enables the analysis engine to select precise line ranges for context assembly rather than including entire files.

---

## The BuildSystem Trait

**Source**: `crates/roko-core/src/build.rs`

The `BuildSystem` trait abstracts the four fundamental operations every software project needs: compile, test, lint, and format. It produces `BuildCommand` descriptors -- pure data structures that carry program name, arguments, environment variables, and working directory -- but **never execute anything**. This keeps `roko-core` free of `std::process` and `std::fs`, making it portable, testable, and embeddable.

### BuildCommand

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildCommand {
    /// The program binary to invoke (e.g. `"cargo"`, `"npm"`).
    pub program: String,
    /// Positional arguments.
    pub args: Vec<String>,
    /// Additional environment variables to set.
    pub env: HashMap<String, String>,
    /// Working directory override. `None` means inherit from the caller.
    pub working_dir: Option<PathBuf>,
}
```

`BuildCommand` uses a builder pattern for construction:

```rust
let cmd = BuildCommand::new("cargo")
    .arg("check")
    .args(["--workspace", "--all-targets"])
    .env("CARGO_TARGET_DIR", "/tmp/target")
    .working_dir("/repo");
```

The execution boundary lives in `roko-gate` or `roko-orchestrator`, which convert `BuildCommand` into `tokio::process::Command` at the moment of execution. This separation means unit tests can verify command construction without spawning processes -- a critical property for fast iteration and CI.

### The Trait

```rust
pub trait BuildSystem: Send + Sync {
    /// Human-readable name (e.g. `"cargo"`, `"npm"`).
    fn name(&self) -> &str;

    /// Command to compile / type-check the project.
    fn compile_cmd(&self, target_dir: &Path) -> BuildCommand;

    /// Command to run tests, optionally filtered.
    fn test_cmd(&self, target_dir: &Path, filter: Option<&str>) -> BuildCommand;

    /// Command to run the linter.
    fn lint_cmd(&self, target_dir: &Path) -> BuildCommand;

    /// Command to run the formatter.
    fn format_cmd(&self, target_dir: &Path, check_only: bool) -> BuildCommand;

    /// Check whether `file_names` (names in the project root) indicate this
    /// build system is present.
    fn detect_from_files(&self, file_names: &[&str]) -> bool;
}
```

**Method semantics**:

- `compile_cmd(target_dir)`: Produces a type-checking or compilation command. For Rust this is `cargo check --workspace --all-targets`; for Go it is `go build ./...`; for npm it is `npm run build`. The `target_dir` parameter is handled differently per build system: Cargo sets it via the `CARGO_TARGET_DIR` environment variable; Go and npm use `working_dir`.

- `test_cmd(target_dir, filter)`: Produces a test-runner command. When `filter` is `Some("my_test")`, the command includes a test name filter (`-- my_test` for Cargo, `-- my_test` for npm, `-run my_test` for Go).

- `lint_cmd(target_dir)`: Produces a linter command. Cargo uses `clippy --workspace --all-targets -- -D warnings`; Go uses `go vet ./...`; npm uses `npx eslint .`.

- `format_cmd(target_dir, check_only)`: Produces a formatter command. When `check_only` is true, the command checks without modifying files (`cargo fmt --all --check`, `gofmt -l .`, `npx prettier --check .`). When false, it writes formatted output.

- `detect_from_files(file_names)`: Returns `true` if the file list contains marker files for this build system. This takes a `&[&str]` rather than touching the filesystem so that `roko-core` stays I/O-free. Detection is based on simple file presence:
  - Cargo: `Cargo.toml`
  - npm: `package.json` (without `pnpm-lock.yaml` or `yarn.lock`)
  - pnpm: `package.json` + `pnpm-lock.yaml`
  - yarn: `package.json` + `yarn.lock`
  - Go: `go.mod`

### All Implementations

There are five concrete `BuildSystem` implementations across the three language crates:

| Struct | Crate | Marker Files | Compile | Test | Lint | Format |
|--------|-------|-------------|---------|------|------|--------|
| `CargoBuildSystem` | `roko-lang-rust` | `Cargo.toml` | `cargo check --workspace --all-targets` | `cargo test --workspace [-- filter]` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all [--check]` |
| `NpmBuildSystem` | `roko-lang-typescript` | `package.json` (no pnpm/yarn lock) | `npm run build` | `npm test [-- filter]` | `npx eslint .` | `npx prettier [--check\|--write] .` |
| `PnpmBuildSystem` | `roko-lang-typescript` | `package.json` + `pnpm-lock.yaml` | `pnpm run build` | `pnpm test [-- filter]` | `pnpm exec eslint .` | `pnpm exec prettier [--check\|--write] .` |
| `YarnBuildSystem` | `roko-lang-typescript` | `package.json` + `yarn.lock` | `yarn run build` | `yarn test [-- filter]` | `yarn run eslint .` | `yarn run prettier [--check\|--write] .` |
| `GoBuildSystem` | `roko-lang-go` | `go.mod` | `go build ./...` | `go test ./... [-run filter]` | `go vet ./...` | `gofmt [-l\|-w] .` |

**Detection priority in the TypeScript ecosystem**: The npm build system only matches when `package.json` is present but neither `pnpm-lock.yaml` nor `yarn.lock` exist. This prevents false positives -- pnpm and yarn projects always have `package.json`, but the lock file disambiguates the actual package manager. The exclusion logic in the source:

```rust
fn detect_from_files(&self, file_names: &[&str]) -> bool {
    file_names.contains(&"package.json")
        && !file_names.contains(&"pnpm-lock.yaml")
        && !file_names.contains(&"yarn.lock")
}
```

---

## The LanguageProvider Trait

**Source**: `crates/roko-core/src/language.rs`

The `LanguageProvider` trait is the heart of code analysis. It defines how to extract structured information from raw source text.

```rust
pub trait LanguageProvider: Send + Sync {
    /// Human-readable language name (e.g. `"rust"`, `"typescript"`).
    fn language_name(&self) -> &str;

    /// File extensions this provider handles (e.g. `["rs"]`).
    fn file_extensions(&self) -> &[&str];

    /// Parse import statements from source text.
    fn parse_imports(&self, source: &str) -> Vec<Import>;

    /// Extract top-level symbol definitions from source text.
    fn extract_symbols(&self, source: &str) -> Vec<Symbol>;
}
```

**Design constraints**:

1. **Pure functions of input text**: Implementations must not touch the filesystem, network, or any external state. They receive `&str` source text and return vectors of typed results. This makes them trivially testable, cacheable, and parallelizable.

2. **Send + Sync**: Providers can be shared across threads. This is critical for parallel file parsing in large projects -- a 10,000-file Rust codebase can be parsed across all CPU cores with a single `Arc<dyn LanguageProvider>`.

3. **No parsing state**: Each call to `parse_imports` or `extract_symbols` is independent. There is no incremental parsing state between calls (that lives in the tree-sitter layer, which wraps the trait).

The consuming code in `roko-index` is completely language-agnostic:

**Source**: `crates/roko-index/src/parser.rs`

```rust
pub fn parse_source(
    path: &str,
    content: &str,
    provider: &dyn LanguageProvider,
) -> SourceFile {
    let symbols = provider.extract_symbols(content);
    let imports = provider.parse_imports(content);

    SourceFile {
        path: path.to_string(),
        language: provider.language_name().to_string(),
        content: content.to_string(),
        symbols,
        imports,
    }
}
```

`parse_source` is a 10-line function. It calls two trait methods and packages the results. It never mentions Rust, TypeScript, or Go. This is the extensibility payoff: adding Python support means implementing `PythonLanguageProvider`; the graph builder, PageRank scorer, HDC fingerprinter, and search layer all work unchanged.

---

## Rust Language Provider: Dual-Mode Parsing

**Source**: `crates/roko-lang-rust/src/lib.rs`
**Source**: `crates/roko-lang-rust/src/tree_sitter_parser.rs`
**Cargo.toml**: `crates/roko-lang-rust/Cargo.toml`

The Rust crate is unique among the language providers because it offers **two** `LanguageProvider` implementations: a heuristic parser that is always available, and a tree-sitter parser that is feature-gated behind `tree-sitter`. The crate's `Cargo.toml` shows the gating:

```toml
[features]
default = []
tree-sitter = ["dep:tree-sitter", "dep:tree-sitter-rust"]

[dependencies]
roko-core = { path = "../roko-core" }
tree-sitter = { version = "0.24", optional = true }
tree-sitter-rust = { version = "0.23", optional = true }
```

And the conditional compilation in `lib.rs`:

```rust
#[cfg(feature = "tree-sitter")]
pub mod tree_sitter_parser;

#[cfg(feature = "tree-sitter")]
pub use tree_sitter_parser::TreeSitterRustProvider;
```

This dual-mode design reflects a practical tradeoff. Tree-sitter provides a full incremental parser based on the GLR parsing algorithm described by Wagner and Graham [4], which handles error recovery, nested definitions, and complex generics correctly. But it depends on the `tree-sitter` C library and language-specific grammar binaries, which adds compilation time and limits portability (e.g., to WASM environments). The heuristic parser handles ~90% of real-world Rust files with zero dependencies, making it suitable for quick scanning, CI pipelines, and resource-constrained environments.

### When to Use Which

| Scenario | Parser | Why |
|----------|--------|-----|
| Quick file scanning, no native deps | `RustLanguageProvider` | Zero dependencies, compiles instantly, handles 90%+ of real-world Rust files correctly |
| WASM environments | `RustLanguageProvider` | tree-sitter requires C compilation, which may not be available |
| CI/CD pipelines needing fast startup | `RustLanguageProvider` | No grammar loading overhead |
| Full AST accuracy needed | `TreeSitterRustProvider` | Handles nested functions, complex generics, macro-generated items |
| Incremental re-parsing | `TreeSitterRustProvider` | tree-sitter supports editing a parse tree without re-parsing the entire file |
| Call graph extraction | `TreeSitterRustProvider` | Heuristic parser cannot extract function call sites |
| Scope nesting analysis | `TreeSitterRustProvider` | AST provides parent-child relationships between symbols |

### Mode 1: Heuristic Parser (RustLanguageProvider)

The heuristic parser works line-by-line, scanning for keyword patterns. It never builds an AST -- it processes each line independently.

**Import Parsing**

The import parser handles three forms of Rust imports:

1. **`use` statements** with brace expansion:
```rust
fn parse_use_line(trimmed: &str) -> Option<Vec<Import>> {
    let rest = strip_visibility_prefix(trimmed);  // strip pub/pub(crate)
    let rest = rest.strip_prefix("use ")?;
    let rest = rest.strip_suffix(';')?.trim();

    if let Some((prefix, items)) = split_brace_use(rest) {
        let imports = items.split(',').filter_map(|item| {
            let item = item.trim();
            if item.is_empty() { return None; }
            let (item_path, alias) = split_use_alias(item);
            let path = if item_path == "self" {
                prefix.to_string()
            } else {
                format!("{prefix}::{item_path}")
            };
            Some(Import { path, alias, kind: ImportKind::Use })
        }).collect();
        return Some(imports);
    }

    let (path, alias) = split_use_alias(rest);
    Some(vec![Import { path, alias, kind: ImportKind::Use }])
}
```

This handles `use std::io::Result as IoResult;`, `use std::collections::{HashMap, HashSet};`, `pub use crate::error::Error;`, and `pub(crate) use crate::inner::Foo;`. The `self` keyword in brace groups (e.g., `use std::io::{self, Read}`) resolves to the prefix itself.

2. **`mod` declarations**: `mod utils;` produces `Import { path: "utils", kind: ImportKind::Mod }`. Block modules (`mod tests { ... }`) are not treated as imports -- they are symbol definitions.

3. **`extern crate` statements**: `extern crate serde_json as json;` produces `Import { path: "serde_json", alias: Some("json"), kind: ImportKind::ExternCrate }`.

**Symbol Extraction**

The symbol extractor processes each line through a chain of pattern matchers:

```rust
fn extract_symbol_from_line(line: &str, line_num: usize) -> Option<Symbol> {
    let trimmed = line.trim();

    // Skip comments and attributes
    if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.is_empty() {
        return None;
    }

    let (vis, rest) = parse_visibility(trimmed);

    // Try each symbol kind in order
    if let Some(name) = try_extract_fn(rest) { ... }
    if let Some(name) = try_extract_keyword(rest, "struct") { ... }
    if let Some(name) = try_extract_keyword(rest, "enum") { ... }
    if let Some(name) = try_extract_keyword(rest, "trait") { ... }
    if let Some(name) = try_extract_impl(rest) { ... }
    if let Some(name) = try_extract_const(rest) { ... }
    if let Some(name) = try_extract_type_alias(rest) { ... }
    if let Some(name) = try_extract_mod_decl(rest) { ... }
    if let Some(name) = try_extract_mod_block(rest) { ... }

    None
}
```

Each `try_extract_*` function strips a keyword prefix, extracts the identifier, and validates that what follows makes sense for that symbol kind:

- **Functions**: Strips `async`, `unsafe`, `const`, and `extern "C"` qualifiers before looking for `fn`. Example: `pub async unsafe fn process<T>(x: T)` correctly extracts `process`. The `strip_fn_qualifiers` function loops to handle combined qualifiers (`async unsafe fn`).

- **Impls**: Handles both `impl Foo { ... }` and `impl Display for Foo { ... }`. For trait impls, the name is `"Display for Foo"`. Generic parameters in angle brackets are skipped with `skip_angle_brackets()`, which tracks bracket depth to handle nested generics like `impl<T: Clone + Debug> Foo<T>`.

- **Constants vs. const fn**: Distinguishes `const MAX_SIZE: usize = 1024;` (a constant) from `const fn compute()` (a function qualifier) by checking if the character after the identifier is `:` (constant) or something else. This disambiguation is done in `strip_fn_qualifiers`, which only strips `const ` if followed by `fn`.

- **Type aliases vs. type definitions**: Ensures `type Foo = ...` is a type alias (has `=` or `<` after the name) rather than a Go-style type keyword.

- **Module blocks vs. declarations**: `mod tests { ... }` (has `{` after name) is a block module symbol. `mod utils;` (has `;` after name) is both a module symbol and a module import.

**Visibility parsing**: The `parse_visibility` function handles `pub`, `pub(crate)`, `pub(super)`, and `pub(in path)` by stripping the prefix and returning the remaining text:

```rust
fn parse_visibility(s: &str) -> (Visibility, &str) {
    if let Some(rest) = s.strip_prefix("pub") {
        let rest = rest.trim_start();
        if let Some(after_paren) = rest.strip_prefix('(') {
            if let Some(close) = after_paren.find(')') {
                return (Visibility::Public, after_paren[close + 1..].trim_start());
            }
        }
        (Visibility::Public, rest)
    } else {
        (Visibility::Private, s)
    }
}
```

**Known heuristic limitations**:
- Cannot parse nested function definitions (closures, inner functions inside function bodies)
- Multi-line function signatures where `fn` and the name are on different lines
- Items generated by procedural macros (`#[derive]`, `#[tokio::main]`, etc.)
- `#[cfg]`-gated items are always included regardless of active feature flags
- No call-site extraction -- the heuristic parser cannot produce `Calls` edges in the dependency graph
- No scope nesting -- produces a flat list of symbols

### Mode 2: Tree-Sitter Parser (TreeSitterRustProvider)

**Source**: `crates/roko-lang-rust/src/tree_sitter_parser.rs`

The tree-sitter parser builds a full abstract syntax tree using the `tree-sitter-rust` grammar, then walks the AST to extract symbols and imports. It implements the same `LanguageProvider` trait, so callers can swap transparently.

Tree-sitter itself is a parser generator that produces incremental, error-tolerant GLR parsers [5]. It was originally developed at GitHub for the Atom editor and is now used by Neovim, Helix, Zed, and dozens of other tools. The key properties that make it suitable for code analysis are:

1. **Error tolerance**: Tree-sitter produces partial ASTs even for malformed source code. An incomplete function signature still yields a usable parse tree for the rest of the file.
2. **Incremental re-parsing**: After an edit, only the affected portion of the parse tree is rebuilt. This makes it suitable for interactive use cases.
3. **Language-agnostic runtime**: The same tree-sitter library parses any language with a grammar definition. Roko could add Python or Java support by adding new grammar crates.

```rust
pub struct TreeSitterRustProvider;

impl LanguageProvider for TreeSitterRustProvider {
    fn language_name(&self) -> &str { "rust" }
    fn file_extensions(&self) -> &[&str] { &["rs"] }

    fn parse_imports(&self, source: &str) -> Vec<Import> {
        let Some(tree) = parse_source(source) else {
            return Vec::new();
        };
        let mut imports = Vec::new();
        let root = tree.root_node();
        collect_imports(root, source, &mut imports);
        imports
    }

    fn extract_symbols(&self, source: &str) -> Vec<Symbol> {
        let Some(tree) = parse_source(source) else {
            return Vec::new();
        };
        let mut symbols = Vec::new();
        let root = tree.root_node();
        collect_symbols(root, source, &mut symbols);
        symbols
    }
}
```

**Parsing infrastructure**:

```rust
fn parse_source(source: &str) -> Option<tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser
        .set_language(&language.into())
        .expect("tree-sitter-rust grammar should load");
    parser.parse(source, None)
}
```

**Import collection**: The `collect_imports` function walks the AST root's children and dispatches on node kind:

```rust
fn collect_imports(node: tree_sitter::Node<'_>, source: &str, imports: &mut Vec<Import>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "use_declaration" => {
                if let Some(argument) = child.child_by_field_name("argument") {
                    extract_use_paths(argument, source, String::new(), imports);
                }
            }
            "mod_item" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = node_text(name_node, source);
                    imports.push(Import { path: name, alias: None, kind: ImportKind::Mod });
                }
            }
            "extern_crate_declaration" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = node_text(name_node, source);
                    let alias = child.child_by_field_name("alias")
                        .and_then(|a| a.child_by_field_name("alias"))
                        .map(|a| node_text(a, source));
                    imports.push(Import { path: name, alias, kind: ImportKind::ExternCrate });
                }
            }
            _ => {}
        }
    }
}
```

The `extract_use_paths` function recursively walks `use` tree nodes, handling `scoped_identifier` (`path::name`), `scoped_use_list` (`path::{a, b}`), `use_as_clause` (`path as alias`), `use_wildcard` (`path::*`), and `use_list` (bare `{a, b}` without a prefix path). It correctly handles nested brace groups that the heuristic parser cannot process -- for example, `use std::collections::{hash_map::{Entry, HashMap}, BTreeMap}` requires recursive descent through nested scoped use lists.

**Symbol collection**: The `collect_symbols` function maps tree-sitter node kinds to `SymbolKind`:

```rust
fn collect_symbols(node: tree_sitter::Node<'_>, source: &str, symbols: &mut Vec<Symbol>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let (kind, name_field) = match child.kind() {
            "function_item" => (SymbolKind::Function, "name"),
            "struct_item"   => (SymbolKind::Struct, "name"),
            "enum_item"     => (SymbolKind::Enum, "name"),
            "trait_item"    => (SymbolKind::Trait, "name"),
            "const_item"    => (SymbolKind::Const, "name"),
            "type_item"     => (SymbolKind::Type, "name"),
            "mod_item"      => (SymbolKind::Module, "name"),
            "impl_item"     => { /* special handling for trait/type extraction */ }
            _ => { continue; }
        };
        // Extract name from field, determine visibility, push Symbol
    }
}
```

For `impl` blocks, the tree-sitter parser extracts both the type and optional trait:

```rust
"impl_item" => {
    let vis = node_visibility(&child);
    let type_name = child.child_by_field_name("type")
        .map(|t| node_text(t, source))
        .unwrap_or_else(|| "unknown".to_string());
    let trait_name = child.child_by_field_name("trait")
        .map(|t| node_text(t, source));
    let name = if let Some(tr) = trait_name {
        format!("{tr} for {type_name}")
    } else {
        type_name
    };
    symbols.push(Symbol {
        name,
        kind: SymbolKind::Impl,
        visibility: vis,
        line: child.start_position().row + 1,
    });

    // Recurse into impl body for methods
    if let Some(body) = child.child_by_field_name("body") {
        collect_impl_methods(body, source, symbols);
    }
}
```

This recursion into `impl` bodies means the tree-sitter parser finds methods that the heuristic parser might miss (e.g., methods defined on separate lines after multi-line where clauses). The `collect_impl_methods` function walks the body looking for `function_item` children and adds them as `SymbolKind::Function` symbols.

**Visibility detection** checks for a `visibility_modifier` child node:

```rust
fn node_visibility(node: &tree_sitter::Node<'_>) -> Visibility {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "visibility_modifier" {
            return Visibility::Public;
        }
    }
    Visibility::Private
}
```

**Parity verification**: The crate includes a test that verifies the tree-sitter provider extracts at least as many symbols as the heuristic parser for a standard set of Rust constructs:

```rust
#[test]
fn heuristic_vs_tree_sitter_parity() {
    let heuristic = crate::RustLanguageProvider;
    let ts = TreeSitterRustProvider;
    let source = r#"
pub fn public_fn() {}
fn private_fn() {}
pub struct MyStruct { x: i32 }
enum MyEnum { A, B }
pub trait MyTrait { fn required(&self); }
const MY_CONST: i32 = 42;
type MyType = Vec<i32>;
"#;
    let heuristic_symbols = heuristic.extract_symbols(source);
    let ts_symbols = ts.extract_symbols(source);
    assert!(ts_symbols.len() >= heuristic_symbols.len());
}
```

### Heuristic vs. Tree-Sitter Tradeoffs Summary

| Property | Heuristic (`RustLanguageProvider`) | Tree-Sitter (`TreeSitterRustProvider`) |
|----------|------------------------------------|----------------------------------------|
| Dependencies | Zero (pure Rust string processing) | `tree-sitter` + `tree-sitter-rust` (C compilation) |
| Parse accuracy | ~90% of real-world files | ~99%+ (full AST) |
| Nested definitions | Cannot extract | Fully supported |
| Macro-generated items | Invisible | Visible in expanded form (if provided) |
| Multi-line signatures | Fails if keyword and name split across lines | Correct |
| Error recovery | Skips the line | Produces partial AST |
| Call site extraction | Impossible | Possible via query |
| Scope nesting | Flat list only | Full parent-child tree |
| Performance | ~1 microsecond per line | ~10 microseconds per file (grammar loading amortized) |
| Binary size impact | None | ~2-5 MB for grammar |
| WASM compatibility | Full | Requires C compilation toolchain |

---

## TypeScript/JavaScript Language Provider

**Source**: `crates/roko-lang-typescript/src/lib.rs`

The TypeScript crate handles four file extensions (`ts`, `tsx`, `js`, `jsx`) and provides three `BuildSystem` implementations plus one `LanguageProvider`. All parsing is heuristic (no tree-sitter integration yet).

### Build Systems: npm, pnpm, yarn

The TypeScript ecosystem has three major package managers, each with slightly different CLI interfaces. All three produce equivalent build commands with different binary names:

**NpmBuildSystem** (`package.json` present, no pnpm/yarn lock files):
```
compile:  npm run build
test:     npm test [-- filter]
lint:     npx eslint .
format:   npx prettier [--check|--write] .
```

**PnpmBuildSystem** (`package.json` + `pnpm-lock.yaml`):
```
compile:  pnpm run build
test:     pnpm test [-- filter]
lint:     pnpm exec eslint .
format:   pnpm exec prettier [--check|--write] .
```

**YarnBuildSystem** (`package.json` + `yarn.lock`):
```
compile:  yarn run build
test:     yarn test [-- filter]
lint:     yarn run eslint .
format:   yarn run prettier [--check|--write] .
```

### TypeScriptLanguageProvider

**Import Parsing**

The import parser handles two module systems:

1. **ES Module imports** via `parse_es_import()`:

```rust
fn parse_es_import(trimmed: &str) -> Option<Import> {
    let rest = trimmed.strip_prefix("import ")?;

    // Side-effect import: `import './styles.css';`
    let rest_no_semi = rest.trim_end_matches(';').trim();
    if rest_no_semi.starts_with('\'') || rest_no_semi.starts_with('"')
       || rest_no_semi.starts_with('`') {
        let path = extract_quoted_string(rest_no_semi)?;
        return Some(Import { path: path.to_string(), alias: None, kind: ImportKind::Use });
    }

    // Find `from` keyword and extract module path
    let from_idx = find_from_keyword(rest)?;
    let after_from = rest[from_idx + 4..].trim().trim_end_matches(';').trim();
    let path = extract_quoted_string(after_from)?;
    let before_from = rest[..from_idx].trim();
    let alias = extract_import_alias(before_from);

    Some(Import { path: path.to_string(), alias, kind: ImportKind::Use })
}
```

This handles all standard ES import forms:
- `import React from 'react'` -- default import, alias = `Some("React")`
- `import { useState, useEffect } from 'react'` -- named imports, alias = `None`
- `import * as path from 'path'` -- namespace import, alias = `Some("path")`
- `import './styles.css'` -- side-effect import, alias = `None`
- `import type { Config } from './config'` -- type-only import

The `find_from_keyword` function is careful about the `from` keyword -- it must be preceded by whitespace or `}` and followed by whitespace, to avoid false positives on identifiers containing "from" (e.g., `import { fromEvent } from 'rxjs'`).

2. **CommonJS `require()`** via `parse_require()`:

```rust
fn parse_require(trimmed: &str) -> Option<Import> {
    let req_idx = trimmed.find("require(")?;
    let after_req = &trimmed[req_idx + 8..];
    let close_paren = after_req.find(')')?;
    let inside = after_req[..close_paren].trim();
    let path = extract_quoted_string(inside)?;
    let before_req = trimmed[..req_idx].trim();
    let alias = extract_require_alias(before_req);
    Some(Import { path: path.to_string(), alias, kind: ImportKind::Use })
}
```

This handles:
- `const express = require('express')` -- alias = `Some("express")`
- `const { readFile } = require('fs')` -- destructured, alias = `None`
- `require('module')` -- bare require, alias = `None`

**Symbol Extraction**

Visibility parsing handles `export`, `export declare`, `declare`, and `export default` prefixes:

```rust
fn parse_ts_visibility(s: &str) -> (Visibility, &str) {
    let rest = s.trim_start();
    if let Some(after_export) = rest.strip_prefix("export ") {
        let after_export = after_export.trim_start();
        if let Some(after_declare) = after_export.strip_prefix("declare ") {
            return (Visibility::Public, after_declare.trim_start());
        }
        if let Some(after_default) = after_export.strip_prefix("default ") {
            return (Visibility::Public, after_default.trim_start());
        }
        (Visibility::Public, after_export)
    } else if let Some(after_declare) = rest.strip_prefix("declare ") {
        (Visibility::Private, after_declare.trim_start())
    } else {
        (Visibility::Private, rest)
    }
}
```

Symbol extractors for each kind:

| Pattern | SymbolKind | Example |
|---------|-----------|---------|
| `function name(` / `async function name(` / `function* name(` | `Function` | `export async function fetchData() {}` |
| `class Name` / `abstract class Name` | `Struct` | `export class UserService {}` |
| `interface Name` | `Trait` | `export interface Config {}` |
| `type Name = ...` / `type Name<...` | `Type` | `export type Result<T> = Success<T> \| Error` |
| `const NAME = ...` / `const NAME: Type` | `Const` | `export const MAX_RETRIES = 3` |
| `enum Name` / `const enum Name` | `Enum` | `export enum Direction { Up, Down }` |
| `export default Name;` (bare identifier) | `Module` | `export default App` |

The `export default` handling has a subtle split: `export default class App {}` and `export default function init() {}` are handled by the class/function extractors (after the visibility parser strips the `export default` prefix). Bare `export default App;` (just an identifier) is handled by `try_extract_export_default()` as a `Module` symbol. Expressions like `export default { ... }` or `export default () => { ... }` are intentionally skipped -- they have no meaningful name.

**Identifier extraction** includes the `$` character because JavaScript identifiers can contain dollar signs (e.g., `$scope`, `jQuery`):

```rust
fn extract_ts_identifier(s: &str) -> String {
    s.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
        .collect()
}
```

---

## Go Language Provider

**Source**: `crates/roko-lang-go/src/lib.rs`

### GoBuildSystem

The Go build system has one implementation that maps cleanly to Go toolchain commands:

```rust
impl BuildSystem for GoBuildSystem {
    fn name(&self) -> &str { "go" }

    fn compile_cmd(&self, target_dir: &Path) -> BuildCommand {
        BuildCommand::new("go").args(["build", "./..."]).working_dir(target_dir)
    }

    fn test_cmd(&self, target_dir: &Path, filter: Option<&str>) -> BuildCommand {
        let mut cmd = BuildCommand::new("go")
            .args(["test", "./..."]).working_dir(target_dir);
        if let Some(f) = filter { cmd = cmd.arg("-run").arg(f); }
        cmd
    }

    fn lint_cmd(&self, target_dir: &Path) -> BuildCommand {
        BuildCommand::new("go").args(["vet", "./..."]).working_dir(target_dir)
    }

    fn format_cmd(&self, target_dir: &Path, check_only: bool) -> BuildCommand {
        if check_only {
            BuildCommand::new("gofmt").args(["-l", "."]).working_dir(target_dir)
        } else {
            BuildCommand::new("gofmt").args(["-w", "."]).working_dir(target_dir)
        }
    }

    fn detect_from_files(&self, file_names: &[&str]) -> bool {
        file_names.contains(&"go.mod")
    }
}
```

The `./...` pattern in compile, test, and lint commands is Go's recursive package wildcard -- it includes all packages in the current directory and its subdirectories. Format uses `gofmt` rather than `go fmt` because `gofmt` supports `-l` (list files that differ) for check-only mode, while `go fmt` always writes.

### GoLanguageProvider

**Import Parsing**

Go imports have two forms: single-line and grouped. The parser maintains a boolean state machine to track whether it is inside an `import ( ... )` block:

```rust
fn parse_imports(&self, source: &str) -> Vec<Import> {
    let mut imports = Vec::new();
    let mut in_import_block = false;

    for line in source.lines() {
        let trimmed = line.trim();

        if in_import_block {
            if trimmed == ")" {
                in_import_block = false;
                continue;
            }
            if let Some(imp) = parse_go_import_line(trimmed) {
                imports.push(imp);
            }
            continue;
        }

        if trimmed == "import (" {
            in_import_block = true;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("import ") {
            if let Some(imp) = parse_go_import_line(rest.trim()) {
                imports.push(imp);
            }
        }
    }
    imports
}
```

The `parse_go_import_line` function handles all Go import variants:

```rust
fn parse_go_import_line(s: &str) -> Option<Import> {
    let s = s.trim();
    if s.is_empty() || s.starts_with("//") { return None; }

    let quote_start = s.find('"')?;
    let after_quote = &s[quote_start + 1..];
    let quote_end = after_quote.find('"')?;
    let path = &after_quote[..quote_end];

    let before = s[..quote_start].trim();
    let alias = if before.is_empty() || before == "_" || before == "." {
        if before == "." || before == "_" { Some(before.to_string()) } else { None }
    } else {
        Some(before.to_string())
    };

    Some(Import { path: path.to_string(), alias, kind: ImportKind::Use })
}
```

This handles:
- `"fmt"` -- plain import, alias = `None`
- `log "github.com/sirupsen/logrus"` -- aliased import, alias = `Some("log")`
- `. "testing"` -- dot import (injects all names into current scope), alias = `Some(".")`
- `_ "net/http/pprof"` -- side-effect import, alias = `Some("_")`

**Symbol Extraction**

Go symbol extraction has two unique challenges: grouped `const`/`var` blocks and capitalization-based visibility.

The extractor maintains a state machine for grouped declarations:

```rust
fn extract_symbols(&self, source: &str) -> Vec<Symbol> {
    let mut symbols = Vec::new();
    let mut decl_group = None;  // tracks whether we're inside const(...) or var(...)

    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;

        if let Some(keyword) = decl_group {
            let trimmed = line.trim();
            if trimmed == ")" {
                decl_group = None;
                continue;
            }
            if let Some(sym) = extract_go_group_member(trimmed, line_num, keyword) {
                symbols.push(sym);
            }
            continue;
        }

        let trimmed = line.trim();
        if is_go_decl_group_start(trimmed, "const") {
            decl_group = Some("const");
            continue;
        }
        if is_go_decl_group_start(trimmed, "var") {
            decl_group = Some("var");
            continue;
        }

        if let Some(sym) = extract_go_symbol(line, line_num) {
            symbols.push(sym);
        }
    }
    symbols
}
```

**Top-level filtering**: Go's symbol extractor only processes lines that start at column 0 (no leading whitespace). This filters out method bodies, struct field definitions, and other indented code:

```rust
// Only process lines that start at column 0 (top-level declarations)
if !line.is_empty() && (line.starts_with(' ') || line.starts_with('\t')) {
    return None;
}
```

**Go visibility**: Unlike Rust and TypeScript, Go determines visibility by the first letter of the name:

```rust
fn go_visibility(name: &str) -> Visibility {
    name.chars().next().map_or(Visibility::Private, |c| {
        if c.is_uppercase() { Visibility::Public } else { Visibility::Private }
    })
}
```

**Function/method extraction** handles both package-level functions and methods with receivers:

```rust
fn try_extract_go_func(trimmed: &str, line_num: usize) -> Option<Symbol> {
    let rest = trimmed.strip_prefix("func ")?;

    // Method with receiver: `(r *Receiver) Name(`
    let rest = if rest.starts_with('(') {
        let close_paren = rest.find(')')?;
        rest[close_paren + 1..].trim_start()
    } else {
        rest
    };

    let name = extract_go_identifier(rest);
    if name.is_empty() { return None; }

    Some(Symbol {
        visibility: go_visibility(&name),
        name,
        kind: SymbolKind::Function,
        line: line_num,
    })
}
```

`func (s *Server) Start() error {}` correctly extracts `Start` with `Visibility::Public`. The receiver `(s *Server)` is skipped.

**Type extraction** distinguishes struct, interface, and alias forms:

```rust
fn try_extract_go_type(trimmed: &str, line_num: usize) -> Option<Symbol> {
    let rest = trimmed.strip_prefix("type ")?;
    let name = extract_go_identifier(rest);
    let after_name = rest[name.len()..].trim_start();

    let kind = if after_name.starts_with("struct") {
        SymbolKind::Struct
    } else if after_name.starts_with("interface") {
        SymbolKind::Trait
    } else {
        SymbolKind::Type
    };

    Some(Symbol { visibility: go_visibility(&name), name, kind, line: line_num })
}
```

`type Reader interface { ... }` produces `SymbolKind::Trait` with `Visibility::Public`. `type handler interface { ... }` produces `SymbolKind::Trait` with `Visibility::Private`. This cross-language mapping allows the dependency graph to treat Go interfaces and Rust traits equivalently.

---

## Polyglot Project Detection

**Source**: `crates/roko-core/src/polyglot.rs`
**Source**: `crates/roko-core/src/project.rs`

Many real-world projects span multiple languages. A Rust backend with a TypeScript frontend, Solidity smart contracts with TypeScript test suites, or a Go service with Python scripts. Roko handles this with a two-level detection system: `detect_from_files` for single-language detection and `detect_polyglot` for multi-language detection.

### Language and DetectedBuildSystem Enums

```rust
// From project.rs
pub enum Language {
    Rust,         // Cargo.toml
    TypeScript,   // package.json
    Go,           // go.mod
    Python,       // pyproject.toml or setup.py
    Solidity,     // foundry.toml
    Unknown,
}

pub enum DetectedBuildSystem {
    Cargo,    // Rust
    Npm,      // Node (npm/yarn/pnpm)
    Go,       // Go toolchain
    Python,   // pip/poetry/uv
    Forge,    // Foundry (Solidity)
    Unknown,
}
```

Note that `DetectedBuildSystem` is a simple enum (a detection tag), distinct from the `BuildSystem` trait which provides runnable commands. The enum tells callers *which* build system was detected; the trait provides the concrete commands.

### Single-Language Detection

The `detect_from_files` function uses an ordered priority list of marker-file rules:

```rust
const RULES: &[Rule] = &[
    Rule { marker: "Cargo.toml",    language: Language::Rust,       build_system: DetectedBuildSystem::Cargo },
    Rule { marker: "go.mod",        language: Language::Go,         build_system: DetectedBuildSystem::Go },
    Rule { marker: "foundry.toml",  language: Language::Solidity,   build_system: DetectedBuildSystem::Forge },
    Rule { marker: "pyproject.toml",language: Language::Python,     build_system: DetectedBuildSystem::Python },
    Rule { marker: "setup.py",      language: Language::Python,     build_system: DetectedBuildSystem::Python },
    Rule { marker: "package.json",  language: Language::TypeScript, build_system: DetectedBuildSystem::Npm },
];
```

**Priority order matters**: Rust wins over Go wins over Solidity wins over Python wins over TypeScript. If a project has both `Cargo.toml` and `package.json` (common for WASM projects), Rust is the primary language. This ordering reflects the assumption that "lower-level" languages are more likely to be the primary project identity.

**Workspace detection** looks for additional marker files:
- TypeScript: `pnpm-workspace.yaml` or `lerna.json`
- Go: `go.work`
- Rust: Requires inspecting `Cargo.toml` contents for a `[workspace]` section (a separate `detect_from_files_with_cargo_toml` function handles this by accepting `Option<&str>` for the file contents)

### Multi-Language Detection

The `detect_polyglot` function extends single detection to identify all languages present:

```rust
pub fn detect_polyglot(file_names: &[&str]) -> PolyglotProject {
    let mut primary = Language::Unknown;
    let mut languages = Vec::new();
    let mut build_systems = Vec::new();

    for rule in POLY_RULES {
        if file_names.contains(&rule.marker) {
            if !languages.contains(&rule.language) {
                languages.push(rule.language);
            }
            if !build_systems.contains(&rule.build_system) {
                build_systems.push(rule.build_system);
            }
            if primary == Language::Unknown {
                primary = rule.language;
            }
        }
    }

    let secondary: Vec<Language> = languages.into_iter()
        .filter(|l| *l != primary).collect();

    PolyglotProject { primary, secondary, build_systems }
}
```

The `PolyglotProject` struct provides convenience methods:

```rust
pub struct PolyglotProject {
    pub primary: Language,
    pub secondary: Vec<Language>,
    pub build_systems: Vec<DetectedBuildSystem>,
}

impl PolyglotProject {
    pub fn is_polyglot(&self) -> bool { !self.secondary.is_empty() }
    pub fn all_languages(&self) -> Vec<Language> {
        let mut all = vec![self.primary];
        all.extend_from_slice(&self.secondary);
        all
    }
}
```

Deduplication ensures that `pyproject.toml` + `setup.py` (both Python markers) produce a single `Language::Python` entry, not a polyglot project. Rule priority determines the primary language regardless of slice ordering -- even if `package.json` appears before `Cargo.toml` in the input slice, Rust wins because it comes first in `POLY_RULES`.

---

## The Symbol Extraction Pipeline

### End-to-End Flow

```
Source text (&str)
    |
    v
[parse_imports(source)]     -- Extract dependency edges
    |                          Returns Vec<Import>
    v
[extract_symbols(source)]   -- Extract symbol definitions
    |                          Returns Vec<Symbol>
    v
[parse_source(path, content, provider)]  -- Package into SourceFile
    |
    v
 SourceFile { path, language, content, symbols, imports }
    |
    v
[build_graph(source_files)]  -- Build SymbolGraph from cross-file edges
    |                           Produces EdgeKind::{Calls, Imports, Implements,
    |                                               Contains, TypeRef}
    v
[pagerank(graph)]            -- Score symbols by structural importance
    |                           Adapts the PageRank algorithm [6] to code
    |                           dependency graphs
    v
[fingerprint_symbol(sym)]    -- Generate 10,240-bit HDC fingerprint
    |                           Uses Kanerva's Binary Spatter Codes [7]
    |                           for similarity search via Hamming distance
    v
Context assembly             -- Budget-constrained selection for LLM prompt
```

### Per-Language Symbol Extraction Detail

**Rust** extracts: `fn` (including `async fn`, `unsafe fn`, `const fn`, `extern "C" fn`), `struct`, `enum`, `trait`, `impl` (both `impl Type` and `impl Trait for Type`), `const`, `type` aliases, `mod` (both declarations and blocks). Visibility: `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`. Import types: `use` with brace expansion, `mod` declarations, `extern crate`.

**TypeScript** extracts: `function` (including `async function`, generator `function*`), `class` (including `abstract class`), `interface`, `type` aliases, `const`, `enum` (including `const enum`), `export default` (bare identifier). Visibility: `export`, `export declare`, `declare`, `export default`. Import types: ES module imports (all forms), CommonJS `require()`.

**Go** extracts: `func` (including methods with receivers), `type X struct`, `type X interface`, `type X` (aliases), `const`, `var`, grouped `const(...)` and `var(...)` blocks. Visibility: capitalization convention. Import types: single imports, grouped imports with aliases/dot/blank.

### What Each Language Does NOT Extract

Understanding the limits is as important as understanding the capabilities:

| Limitation | Rust (heuristic) | Rust (tree-sitter) | TypeScript | Go |
|-----------|-------------------|---------------------|------------|-----|
| Nested functions / closures | No | Yes | No | No |
| Arrow functions / lambdas | N/A | N/A | No | N/A |
| Class methods | N/A | N/A | No | N/A |
| Struct fields | No | No | No | No |
| Enum variants | No | No | No | No |
| Function parameters | No | No | No | No |
| Return types | No | No | No | No |
| Call sites | No | Possible | No | No |
| Decorator/attribute items | No | Partial | No | No |
| Multi-line signatures (split keyword/name) | No | Yes | No | No |

---

## How Language-Aware Analysis Improves Agent Coding Performance

Before discussing the IronClaw integration plan, it is worth examining concrete scenarios where structural code analysis produces measurably better agent output compared to naive text-based approaches.

### Scenario 1: "Fix the authentication bug"

**Without language analysis**: The agent searches for "auth" across all files, includes entire files in context, exhausts 50-80% of the token budget on test helpers, database migrations, and unrelated modules. The agent may miss the relevant `verify_token` function buried in a large file because it does not contain the word "auth".

**With language analysis**: The symbol index finds `AuthService`, `verify_token`, and `TokenValidator` as `SymbolKind::Struct/Function/Trait`. The import graph reveals that `verify_token` is called from `login_handler` and `middleware`, and that it imports `TokenValidator`. PageRank scores `AuthService` highest because it has the most incoming edges. The agent receives exactly the 8-12 function definitions it needs, fitting comfortably in 2,000 tokens rather than 50,000.

### Scenario 2: "Add a new API endpoint"

**Without language analysis**: The agent may generate an endpoint following outdated patterns from a different part of the codebase, or miss a required middleware registration step.

**With language analysis**: The build system detection identifies this as a Rust/Cargo project. The symbol graph shows all existing handler functions, their import patterns, and the middleware registration flow. The agent can follow the established pattern: define handler, register in router, add tests. The `BuildSystem` trait produces the correct `cargo test` command to verify the new endpoint.

### Scenario 3: "Why is this test failing?"

**Without language analysis**: The agent reads the test file and the implementation file, but misses a transitive dependency that was recently changed.

**With language analysis**: Starting from the failing test function, the import graph traces the dependency chain: test -> handler -> service -> repository -> database adapter. The agent finds the recent change two levels deep that altered the return type of a repository method, causing a type mismatch in the service layer.

### Scenario 4: "Refactor this TypeScript service to use the new API"

**Without language analysis**: The agent finds the service file but misses the 7 other files that import from it. After the refactor, those files have broken imports.

**With language analysis**: The import graph shows all 8 files that import from the service module. The agent includes all call sites in its context and updates each one, producing a complete refactoring. The `BuildSystem` produces `npx eslint .` to verify no type errors remain.

---

## IronClaw Integration Plan

IronClaw's architecture provides natural integration points for the roko language analysis system. The key insight is that IronClaw's tool system (`src/tools/`) and workspace memory (`src/workspace/`) already provide the infrastructure for code-aware agent behavior -- the roko crates add the structural analysis layer that makes it precise.

### Integration Point 1: Project Detection Tool

**Maps to**: `src/tools/builtin/` (new tool) + `src/agent/` (session context)

When a user first mentions a project directory, a new `project_detect` built-in tool runs polyglot detection and stores the result in the session context:

```rust
use roko_core::polyglot::detect_polyglot;
use roko_core::project::detect_from_files;

// Tool implementation: project_detect
async fn execute(&self, params: Value) -> Result<ToolOutput, ToolError> {
    let project_dir = params["path"].as_str().ok_or(ToolError::MissingParam("path"))?;

    // List files in the project root (I/O happens here, not in roko-core)
    let entries = tokio::fs::read_dir(project_dir).await?;
    let file_names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect()
        .await;
    let refs: Vec<&str> = file_names.iter().map(|s| s.as_str()).collect();

    let polyglot = detect_polyglot(&refs);

    Ok(ToolOutput::text(format!(
        "Primary: {}, Secondary: {:?}, Build systems: {:?}",
        polyglot.primary, polyglot.secondary, polyglot.build_systems
    )))
}
```

This follows IronClaw's "Everything Goes Through Tools" principle (see `CLAUDE.md`) -- the detection runs through `ToolDispatcher::dispatch()` and gets the full audit trail, safety pipeline, and channel-agnostic surface.

### Integration Point 2: Build Commands via Tool System

**Maps to**: `src/tools/builtin/shell.rs` (command execution) + `BuildSystem` trait

IronClaw's shell tool currently requires the agent to know the correct build commands for each language. With `BuildSystem` integration, the agent can generate correct commands automatically:

```rust
use roko_lang_rust::CargoBuildSystem;
use roko_lang_typescript::NpmBuildSystem;
use roko_core::build::BuildSystem;

fn get_build_system(language: Language) -> Box<dyn BuildSystem> {
    match language {
        Language::Rust => Box::new(CargoBuildSystem),
        Language::TypeScript => Box::new(NpmBuildSystem),
        Language::Go => Box::new(GoBuildSystem),
        _ => return Err(ToolError::unsupported("Unknown language")),
    }
}

// Agent wants to run tests for the user's project
let bs = get_build_system(detected_language);
let cmd = bs.test_cmd(Path::new("/user/project"), Some("my_failing_test"));
// -> BuildCommand { program: "cargo", args: ["test", "--workspace", "--", "my_failing_test"] }
```

The `BuildCommand` descriptor is converted to a `tokio::process::Command` at the execution boundary. This separation means the agent never hardcodes `cargo test` vs. `npm test` vs. `go test -run` -- it asks the build system for the correct command.

For IronClaw's sandbox environment (`SANDBOX_ENABLED=true`), the `BuildCommand` maps naturally to the existing `sandbox_daemon` NDJSON protocol. The `program`, `args`, and `env` fields translate directly to what the daemon expects.

### Integration Point 3: Symbol Index for Context Assembly

**Maps to**: `src/workspace/` (memory system) + `crates/ironclaw_engine/` (context management)

The most powerful integration uses `LanguageProvider` + `roko-index` to build a structural index of the user's codebase, then uses PageRank and HDC fingerprints to select the most relevant code for each conversation:

```rust
use roko_index::{parse_source, build_graph, pagerank};
use roko_lang_rust::RustLanguageProvider;

// Parse all Rust files in the project
let provider = RustLanguageProvider;
let mut source_files = Vec::new();
for file_path in walk_dir("/user/project", &["rs"]) {
    let content = tokio::fs::read_to_string(&file_path).await?;
    let sf = parse_source(&file_path, &content, &provider);
    source_files.push(sf);
}

// Build dependency graph and score symbols
let graph = build_graph(&source_files);
let scores = pagerank(&graph, 0.85, 100);

// Now the agent knows:
// - What symbols exist in every file
// - Which symbols are structurally important (high PageRank)
// - Which files import which other files
// - The visibility (public API vs. internal) of every symbol
```

This integrates with IronClaw's progressive context management system (`crates/ironclaw_engine/` context management). Instead of including entire files in the system prompt, the engine can include only the symbols relevant to the current conversation, ranked by structural importance.

### Integration Point 4: Memory-Backed Code Understanding

**Maps to**: `src/workspace/` (memory_write, memory_search tools)

IronClaw's workspace memory system uses hybrid search (FTS + vector via RRF). Symbol-level knowledge can be stored as memory entries, making code structure searchable alongside the user's natural-language knowledge:

```rust
// When indexing a project, write structural knowledge to workspace memory
for file in &source_files {
    for sym in &file.symbols {
        memory_write(
            format!("code:{}:{}:{}", file.path, sym.kind, sym.name),
            format!(
                "Symbol '{}' ({:?}, {:?}) at {}:{}",
                sym.name, sym.kind, sym.visibility, file.path, sym.line
            ),
        );
    }
}

// When the agent needs to find relevant code:
let results = memory_search("UserService authentication handler");
// Returns ranked results including code symbols alongside knowledge entries
```

The memory system already supports the `memory_search` tool with hybrid ranking. Code symbols stored this way benefit from the same FTS + vector scoring, meaning the agent can find relevant code structures using natural language queries.

### Integration Point 5: Smart File Selection for Context Windows

**Maps to**: `crates/ironclaw_engine/` (tool disclosure and context budgeting)

Instead of including entire files in the context window, IronClaw could use the symbol graph to include only the relevant definitions:

```rust
// User asks: "How does the authentication flow work?"
// Agent identifies relevant entry points via memory_search

// Use the code index to find all symbols related to authentication
let auth_symbols = index.search("authenticate", SearchStrategy::Keyword)?;

// Get the transitive closure of dependencies
let deps = graph.transitive_deps(&auth_symbols);

// Assemble only the relevant code into the context
let context = assemble_context(&deps, token_budget: 4000);
// Returns exactly the functions, structs, and traits needed
// to understand authentication, skipping unrelated code
```

This is the core value proposition: instead of blindly including entire files (wasting 80-99% of context on irrelevant code), the agent includes exactly the symbols it needs, ranked by structural importance. Research on retrieval-augmented code generation confirms that precise context selection significantly improves generation quality [1].

### Implementation Phases

**Phase 1: Detection only** (low effort, high value)
- Add `roko-core` as a dependency to IronClaw
- Implement `project_detect` tool using `detect_polyglot`
- Store detected language/build system in session context
- Use `BuildSystem` trait to generate correct compile/test/lint/format commands
- No changes to the agent loop or memory system

**Phase 2: Symbol index** (medium effort, high value)
- Add `roko-lang-rust`, `roko-lang-typescript`, `roko-lang-go` as dependencies
- Add `roko-index` for graph building and PageRank
- Build symbol index on project first load, store in workspace memory
- Use symbol search in `memory_search` queries

**Phase 3: Context-aware assembly** (high effort, transformative value)
- Integrate PageRank scores into the engine's context budgeting
- Use HDC fingerprints for similarity-based code retrieval
- Build incremental update pipeline (re-index only changed files)
- Add MCP server endpoint for external tool access to the symbol graph

---

## References

[1] Y. Li et al., "Retrieval-Augmented Code Generation: A Survey with Focus on Repository-Level Approaches," arXiv:2510.04905, 2025. Available: https://arxiv.org/abs/2510.04905

[2] M. Mayer, S. Islam, and A. Goel, "Multi-Lingual Development & Programming Languages Interoperability: An Empirical Study," arXiv:2411.08388, 2024. Available: https://arxiv.org/abs/2411.08388

[3] S. Peyton Jones et al., "Type Classes in Haskell," in Proc. European Symposium on Programming (ESOP), 1994. Rust's trait system is a direct descendant of Haskell's type class mechanism, providing bounded polymorphism through trait bounds on generic parameters.

[4] T. A. Wagner and S. L. Graham, "Efficient and Flexible Incremental Parsing," ACM Transactions on Programming Languages and Systems, vol. 20, no. 5, pp. 980-1013, September 1998. Available: https://dl.acm.org/doi/10.1145/293677.293678. Tree-sitter's incremental parsing algorithm builds on this foundational work.

[5] M. Brunsfeld, "Tree-sitter -- A New Parsing System for Programming Tools," Strange Loop Conference, 2018. Available: https://www.thestrangeloop.com/2018/tree-sitter---a-new-parsing-system-for-programming-tools.html. Tree-sitter was originally developed at GitHub for the Atom editor.

[6] S. Brin and L. Page, "The Anatomy of a Large-Scale Hypertextual Web Search Engine," Computer Networks and ISDN Systems, vol. 30, pp. 107-117, 1998. Roko adapts PageRank for code dependency graphs, where incoming edges represent imports/calls rather than hyperlinks.

[7] P. Kanerva, "Hyperdimensional Computing: An Introduction to Computing in Distributed Representation with High-Dimensional Random Vectors," Cognitive Computation, vol. 1, no. 2, pp. 139-159, 2009. Roko's HDC fingerprints use Binary Spatter Codes (BSC) to encode symbol structure into 10,240-bit vectors for fast similarity search via Hamming distance.

[8] D. Kolovos et al., "Polyglot and Distributed Software Repository Mining with Crossflow," in Proc. Mining Software Repositories (MSR), 2020. Available: https://dl.acm.org/doi/10.1145/3379597.3387481. Demonstrates the challenges and approaches for analyzing polyglot software repositories.

[9] M. Atzeni et al., "Polyglot AST: Towards Enabling Polyglot Code Analysis," 2023. Available: https://www.researchgate.net/publication/375851504_Polyglot_AST_Towards_Enabling_Polyglot_Code_Analysis. Proposes unified AST representations for cross-language code analysis, a similar goal to roko-core's universal `SymbolKind` taxonomy.

[10] S. Kang et al., "Guiding Language Models of Code with Global Context using Monitors," arXiv:2306.10763, 2023. Demonstrates that providing language-server-derived context (types, imports, definitions) to LLMs during code generation significantly reduces errors.

[11] "Language Server Protocol Specification," Microsoft, 2016-present. Available: https://microsoft.github.io/language-server-protocol/. The LSP provides a complementary approach to roko's trait-based design: while LSP defines a runtime protocol between editors and language servers, roko's `LanguageProvider` trait defines a compile-time abstraction for embedding language analysis directly into the agent.

[12] Deprank (codemix), "Use PageRank to find the most important files in your codebase," 2022. Available: https://github.com/codemix/deprank. A JavaScript implementation of PageRank over file dependency graphs that validates the approach roko-index takes at the symbol level.

[13] D. Kempf et al., "A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures, Part I: Models and Data Transformations," ACM Computing Surveys, vol. 55, no. 6, 2023. Available: https://dl.acm.org/doi/10.1145/3538531. Comprehensive survey of VSA models including the Binary Spatter Codes that roko-index uses.

---

## File Reference Index

All source files referenced in this document, with their paths relative to the roko repository root (`/Users/will/dev/nunchi/roko/roko/`):

| File | Purpose |
|------|---------|
| `crates/roko-core/src/build.rs` | `BuildSystem` trait, `BuildCommand` struct |
| `crates/roko-core/src/language.rs` | `LanguageProvider` trait, `Symbol`, `Import`, `SymbolKind`, `Visibility` |
| `crates/roko-core/src/project.rs` | `Language` enum, `DetectedBuildSystem` enum, `ProjectInfo`, `detect_from_files()` |
| `crates/roko-core/src/polyglot.rs` | `PolyglotProject`, `detect_polyglot()` |
| `crates/roko-lang-rust/src/lib.rs` | `CargoBuildSystem`, `RustLanguageProvider` (heuristic) |
| `crates/roko-lang-rust/src/tree_sitter_parser.rs` | `TreeSitterRustProvider` (AST-based parser) |
| `crates/roko-lang-rust/Cargo.toml` | Feature-gating for tree-sitter dependency |
| `crates/roko-lang-typescript/src/lib.rs` | `NpmBuildSystem`, `PnpmBuildSystem`, `YarnBuildSystem`, `TypeScriptLanguageProvider` |
| `crates/roko-lang-go/src/lib.rs` | `GoBuildSystem`, `GoLanguageProvider` |
| `crates/roko-index/src/lib.rs` | `roko-index` crate root, re-exports |
| `crates/roko-index/src/parser.rs` | `SourceFile`, `parse_source()` |
| `crates/roko-index/src/symbol.rs` | `SymbolId`, `SymbolRef`, `find_symbol()` |
| `crates/roko-index/src/graph.rs` | `SymbolGraph`, `build_graph()`, `pagerank()`, `EdgeKind` |
| `crates/roko-index/src/hdc.rs` | `HdcFingerprint`, `fingerprint_symbol()`, `similarity()` |
