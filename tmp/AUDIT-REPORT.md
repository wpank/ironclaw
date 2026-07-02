# Cross-Document Consistency Audit Report

**Auditor**: Automated cross-document consistency analysis
**Date**: 2026-07-02
**Scope**: All 29 documents in `/Users/will/dev/near/ironclaw/tmp/` (00-INDEX.md through 28-plans-catalog.md)

---

## Executive Summary

The document collection is a substantial and well-structured technology transfer analysis comprising 29 documents totaling an estimated 400,000+ words. The core concept documents (01-17) are individually thorough and technically deep. However, the collection suffers from several systemic consistency issues:

1. **The INDEX (00) is stale** -- it references only 19 documents but 29 exist. Documents 20-28 are completely absent from the index table.
2. **The ROI formula in the priority matrix (19) does not produce the claimed scores** -- every single value in the table is incorrect relative to the stated formula.
3. **Priority ratings in individual document headers conflict with the priority matrix rankings** -- what a document calls "HIGH" may be ranked mid-table or split across multiple entries.
4. **Terminology is inconsistent** -- "Engram" vs "Signal" is the most pervasive, but "Substrate" vs "Store" and crate count discrepancies also appear.
5. **The roadmap (18) and priority matrix (19) are not fully aligned** on phasing and effort estimates.

Despite these issues, the individual documents are well-written, technically rigorous, and internally consistent. The problems are primarily in the meta-layer (index, cross-references, scoring) rather than in the technical content itself.

---

## 1. Consistency Issues

### 1.1 INDEX Document (00) Does Not Cover All Documents

**Severity**: HIGH

The INDEX document (00-INDEX.md) states "Analysis documents produced: 19 (docs 01-19)" and its document table lists only entries 01 through 19. However, 9 additional documents exist:

| # | Document | Title |
|---|----------|-------|
| 20 | `20-persistence-storage.md` | Roko Persistence and Storage Layer |
| 21 | `21-mcp-editor-integration.md` | Roko's ACP and MCP Editor Integration Protocols |
| 22 | `22-language-support.md` | Roko Multi-Language Code Analysis System |
| 23 | `23-control-plane.md` | Control Plane & API Server Architecture |
| 24 | `24-smart-contracts.md` | On-Chain Smart Contract Architecture |
| 25 | `25-research-citations.md` | Roko Research Citations: Comprehensive Bibliography |
| 26 | `26-roko-architecture-overview.md` | Roko Architecture Overview |
| 27 | `27-v2-depth-research.md` | Comprehensive Catalog: Roko v2-Depth Research Documents |
| 28 | `28-plans-catalog.md` | Roko Implementation Plans -- Comprehensive Catalog |

None of these appear in the INDEX table, the reading guides, the "Novel Concepts" section, or the "Recommended Next Steps." The INDEX presents itself as the "definitive entry point" but is incomplete.

**Recommendation**: Update the INDEX table to include documents 20-28 with summaries, priority, effort, and IronClaw fit columns. Update the Key Statistics section to say "29" instead of "19." Add the new documents to the appropriate reading guides.

### 1.2 ROI Formula Does Not Produce Claimed Values

**Severity**: HIGH

The priority matrix (19-priority-matrix.md) states the ROI formula as:

```
ROI = (UserImpact * 0.30 + SystemImpact * 0.20) / (Effort * 0.25 + Risk * 0.15 + DepBurden * 0.10)
```

This formula was applied to all 25 entries. **None of the 25 claimed ROI values match what the formula produces.** Examples:

| Concept | UI | SI | Eff | Risk | Dep | Claimed ROI | Computed ROI | Delta |
|---------|----|----|-----|------|-----|-------------|--------------|-------|
| Metacognitive Monitor | 5 | 4 | 2 | 1 | 1 | 4.71 | 3.07 | +1.64 |
| Ebbinghaus Decay | 4 | 4 | 1 | 1 | 1 | 4.57 | 4.00 | +0.57 |
| Cascade Router | 5 | 4 | 3 | 2 | 1 | 3.31 | 2.00 | +1.31 |
| Full Affect Engine | 2 | 2 | 4 | 4 | 2 | 0.95 | 0.56 | +0.39 |

The discrepancies are not rounding errors -- they range from +0.35 to +1.64. The claimed values appear to come from a different formula (possibly an inverted additive weighting where effort/risk/dep are subtracted from 6 before weighting), but no such formula is documented.

**Impact**: The relative ranking is broadly preserved (high-impact, low-effort items still score highest), so the practical prioritization advice remains directionally correct. However, the mathematical foundation as stated is wrong.

**Recommendation**: Either correct the formula to match the actual computation, or recompute all ROI values using the stated formula and re-verify rankings.

### 1.3 Priority Ratings in Document Headers vs. Priority Matrix

**Severity**: MEDIUM

Each concept document (01-17) has a header line like "Priority: HIGH" or "Priority: MEDIUM." The priority matrix (19) provides a more granular ranking. These do not always agree:

| Doc | Header Priority | Matrix Rank | Matrix Quadrant | Conflict? |
|-----|-----------------|-------------|-----------------|-----------|
| 01 (HDC) | HIGH | #6 | Big Bets | Mild -- HIGH but ranked 6th, not top-4 |
| 02 (Dreams) | HIGH | #8 (Enhanced) / #18 (Full) | Big Bets / Long-term | Split -- "HIGH" header but full version ranked 18th |
| 04 (DAG) | HIGH | #13 | Big Bets | Conflict -- #13 is well below what "HIGH" implies |
| 05 (Gate) | HIGH | #7 | Big Bets | Mild -- reasonably aligned |
| 06 (Conductor) | MEDIUM | #12 | Big Bets | Consistent |
| 07 (Cascade Router) | HIGH | #5 | Big Bets | Consistent |
| 08 (Chain Rep) | MEDIUM | #21 | Long-term | Conflict -- #21 is lower than MEDIUM implies |
| 10 (Engram) | HIGH | #2 (Decay) / #3 (Dedup) | Stars | Consistent (split into sub-features) |
| 13 (Cognitive Arch) | HIGH | #11 / #22 | Nice-to-have / Long-term | Conflict -- split into two lower-ranked items |
| 14 (Runtime Infra) | MEDIUM | #9 / #15 / #25 | Mixed | Split into three items at different ranks |
| 16 (Plugin) | LOW | #16 | Nice-to-have | Consistent |
| 17 (Agent Patterns) | MEDIUM | #1 / #10 / #14 | Stars / Nice-to-have | Conflict -- Metacognitive Monitor is #1 but doc is "MEDIUM" |

The most notable conflicts:
- Doc 17 is labeled "MEDIUM" but its top sub-feature (Metacognitive Monitor) is ranked #1 overall.
- Doc 04 (DAG) is labeled "HIGH" but ranked #13 in the matrix.
- Doc 13 (Cognitive Architecture) is labeled "HIGH" but its sub-features rank #11 and #22.

**Recommendation**: Align document header priorities with the matrix ranking. Consider using sub-feature ratings in headers when a document covers multiple concepts (e.g., "Priority: HIGH (metacognitive monitor), MEDIUM (composable scorers), MEDIUM (resumable checkpoints)").

### 1.4 Crate Count Discrepancy

**Severity**: LOW

The documents cite different numbers for how many crates roko contains:

| Document | Claim |
|----------|-------|
| 00-INDEX.md | "18+ crates" (Key Statistics table) |
| 26-roko-architecture-overview.md | "18 crates, ~200K lines of Rust, 1,600+ tests" |
| 27-v2-depth-research.md | "32+ crates" (in the header description of roko) |
| 18-integration-roadmap.md | "32+ crates and 155+ depth research documents" |

The 18 vs 32 discrepancy likely reflects different counting criteria (core crates vs all crates including dev/build/lang-specific), but the inconsistency creates confusion.

**Recommendation**: Standardize on one count with a clear definition (e.g., "18 core crates, 32 total including language providers, MCP servers, and tooling").

### 1.5 Effort Estimate Inconsistencies Between Roadmap and Matrix

**Severity**: MEDIUM

The roadmap (18) and priority matrix (19) sometimes give different effort estimates for the same feature:

| Feature | Roadmap (18) Estimate | Matrix (19) Effort Score | Matrix (19) Time |
|---------|----------------------|--------------------------|------------------|
| Metacognitive Monitor | 2-3 developer-days | Effort = 2 (Low) | 2-3 days |
| Ebbinghaus Decay | 2-3 developer-days | Effort = 1 (Trivial) | 1-2 days |
| Robust Statistics | 1-2 developer-days | Effort = 1 (Trivial) | 1 day |
| Cascade Router | 8-12 developer-days | Effort = 3 (Medium) | 2-3 weeks |
| HDC Similarity | (doc 18 says 2-3 weeks) | Effort = 3 (Medium) | 2-3 weeks |
| Enhanced Heartbeat | (doc 18 says 2-3 weeks) | Effort = 3 (Medium) | 2-3 weeks |

The Ebbinghaus Decay entry is mildly inconsistent: "2-3 days" in the roadmap but "Effort = 1 (Trivial, done in a day)" and "1-2 days" in the matrix. The matrix rubric defines Effort 1 as "Under 200 lines, touches 1-2 files, no migrations, done in a day" but the actual description acknowledges a database migration is needed for both backends, which by the rubric's own definition would push it to Effort 2.

**Recommendation**: Reconcile the effort estimates and ensure they match the rubric definitions. If a feature requires database migrations on two backends, it should not be scored as Effort 1 per the rubric's own criteria.

---

## 2. Factual Conflicts

### 2.1 Signal vs. Engram Terminology

**Severity**: MEDIUM

Document 27 (v2-depth-research.md) explicitly states:

> The unified vocabulary mandates specific terms: Signal (not Engram), Cell (not Module), Graph (not Workflow), Store (not Substrate), etc.

However, the following documents use "Engram" as the primary term:

- **10-universal-engram.md**: Title is "Universal Engram." Uses "Engram" ~200+ times. The `Engram` struct is the central type.
- **06-conductor-anomaly.md**: References "React trait and Engram Signal Model" and uses `Engram` in code examples.
- **16-plugin-extension.md**: Uses `SignalSender = Sender<Engram>` -- showing the `Engram` type is what flows through the system.
- **20-persistence-storage.md**: The `Store` trait operates on `Engram` types (`put(&self, engram: Engram)`).
- **13-cognitive-architecture.md**: Uses "Engram" in multiple code citations.

Document 27 also notes:

> Five core primitives: Signal (durable data), Cell (atomic computation), Graph (composition of Cells), Bus (ephemeral pub/sub), Store (persistent state).

This creates a factual conflict: the v2 spec mandates "Signal" but the actual roko code (and most documents in this collection) uses "Engram." The resolution is likely that the code has not yet migrated to the v2 terminology, but the documents should be consistent about which term they use and note the discrepancy.

**Recommendation**: Add a terminology note to the INDEX explaining that roko has both v1 terminology (Engram) and v2 terminology (Signal), and that the codebase primarily uses v1 terms. Individual documents should note which version of the vocabulary they follow.

### 2.2 Roko's LLM Provider List

**Severity**: LOW

Different documents list different sets of LLM providers:

| Document | Providers Listed |
|----------|-----------------|
| 00-INDEX.md | "Claude, Gemini, Perplexity, OpenRouter, Ollama, and any OpenAI-compatible API" |
| 17-agent-patterns.md | "Anthropic Claude, OpenAI, Ollama, Gemini, Perplexity, and OpenAI-compatible endpoints" |
| 26-roko-architecture-overview.md | Same as INDEX |

The INDEX and doc 26 omit "OpenAI" as a distinct provider (only mentioning "OpenAI-compatible"), while doc 17 includes it explicitly. IronClaw's CLAUDE.md lists its own providers as "NEAR AI, OpenAI, Anthropic, Ollama, AWS Bedrock, GitHub Copilot, Tinfoil, and OpenAI-compatible."

This is a minor inconsistency but could confuse readers about exact provider support.

### 2.3 IronClaw's SmartRoutingProvider Dimensionality

**Severity**: LOW

- Doc 18 (roadmap) says: "IronClaw already has SmartRoutingProvider in crates/ironclaw_llm/src/smart_routing.rs with a 13-dimension complexity scorer."
- Doc 00 (INDEX) says: "A 13-dimension SmartRoutingProvider already classifies request complexity."

Both agree on 13 dimensions, which is internally consistent.

### 2.4 Heartbeat Default Interval

**Severity**: LOW

- Doc 18 (roadmap): "IronClaw's heartbeat runs every 30 minutes" (matches CLAUDE.md).
- Doc 13 (cognitive architecture): References heartbeat as the Delta speed, mapping to "hours."
- Doc 19 (priority matrix): "current heartbeat reads HEARTBEAT.md."

These are consistent with each other. The 30-minute interval is an IronClaw fact, while "hours" is the roko-style Delta speed for deep consolidation. The documents appropriately distinguish these.

---

## 3. Gaps and Missing Cross-References

### 3.1 Documents 20-28 Have No Cross-References from 01-19

**Severity**: HIGH

Documents 20-28 were apparently written after the original 01-19 set, but no backward links were added. Specific missing cross-references:

| From Doc | Should Reference | Topic |
|----------|------------------|-------|
| 10 (Universal Engram) | 20 (Persistence Storage) | The Store trait that persists Engrams is documented in detail in doc 20, but doc 10 does not mention doc 20 |
| 12 (Code Intelligence) | 22 (Language Support) | Doc 22 provides the detailed language provider implementations that doc 12 references generically |
| 08 (Chain Reputation) | 24 (Smart Contracts) | Doc 24 provides the full Solidity contract reference for the concepts described in doc 08 |
| 15 (Orchestrator) | 23 (Control Plane) | Doc 23 describes the control plane that sits above the orchestrator described in doc 15 |
| 17 (Agent Patterns) | 21 (MCP/ACP Integration) | Doc 21 covers the ACP protocol that implements several agent patterns from doc 17 |
| 25 (Research Citations) | All (01-24) | Doc 25 is a comprehensive bibliography but is not referenced from any other document as a "see also" |
| 26 (Architecture Overview) | 00 (INDEX) | Doc 26 provides an architecture overview that the INDEX should recommend as a starting point |

**Recommendation**: Add "See Also" sections to documents 01-19 that reference the relevant documents from 20-28. Add documents 20-28 to the INDEX reading guides.

### 3.2 Missing Deep-Dive: Roko's Test Infrastructure

Multiple documents mention roko's test suite (1,600+ tests) and testing patterns, but no dedicated document covers roko's testing infrastructure, test patterns, or verification methodology. Given IronClaw's emphasis on testing discipline, a document on roko's testing approach would be valuable.

### 3.3 Missing Deep-Dive: Roko's TUI/Dashboard

Document 26 mentions roko has "an interactive ratatui TUI dashboard" and document 28 references TUI-related plans (P17-P18). No dedicated document covers the TUI architecture, which could inform IronClaw's own CLI/TUI (which also uses ratatui).

### 3.4 Missing Deep-Dive: Roko's Configuration System

Document 27 lists a `14-config` section with "Configuration as Signal" but no standalone document in the 00-28 set covers roko's configuration system in detail. IronClaw has a complex configuration system (`src/config/`) that could benefit from comparing approaches.

### 3.5 Priority Matrix Does Not Cover Documents 20-28

The priority matrix (19) ranks 25 concepts extracted from documents 01-17. Documents 20-28 introduce additional concepts that could be ranked:

| Doc | Concept Not Ranked |
|-----|--------------------|
| 20 | Append-only JSONL persistence pattern |
| 21 | ACP protocol implementation for IDE integration |
| 22 | Multi-language code analysis with dual-mode parsers |
| 23 | Hub-and-spoke control plane with per-agent sidecars |
| 24 | Solidity contract suite for AI agent economics |
| 28 | TOML-based plan specification for self-development |

These are distinct concepts that may warrant ranking in the priority matrix.

### 3.6 Roadmap (18) Does Not Reference Documents 20-28

The integration roadmap references documents 01-17 as "companion docs" but does not incorporate any integration recommendations from documents 20-28. In particular:

- Doc 21 (ACP/MCP) has direct IronClaw integration relevance (IronClaw already has MCP support).
- Doc 23 (Control Plane) describes patterns that could inform IronClaw's web gateway.
- Doc 28 (Plans Catalog) contains implementation blueprints that could accelerate roadmap items.

---

## 4. Quality Scores (1-10)

Scoring criteria: Technical depth, accuracy of claims, completeness, internal consistency, cross-reference quality, practical utility for the stated audience, and writing quality.

| Doc | Title | Score | Notes |
|-----|-------|-------|-------|
| 00 | INDEX | 6/10 | Well-structured reading guides and comparison tables, but stale -- missing 9 documents, statistics outdated |
| 01 | Hyperdimensional Computing | 9/10 | Excellent depth, clear mathematical exposition, concrete code citations, strong IronClaw integration plan |
| 02 | Dream Consolidation | 9/10 | Thorough neuroscience grounding, detailed architecture, clear implementation path |
| 03 | Affect Engine | 8/10 | Good coverage of PAD model and somatic markers, but some sections feel speculative |
| 04 | DAG Execution | 8/10 | Solid technical description of Cell/Graph model, good TOML examples, but "HIGH" priority conflicts with matrix rank |
| 05 | Gate Verification | 9/10 | Comprehensive verification pipeline, adaptive thresholds well-explained, strong IronClaw mapping |
| 06 | Conductor Anomaly | 9/10 | Excellent technical depth on Holt smoothing, Thompson Sampling, and compound event processing |
| 07 | Online Learning | 9/10 | Best cost-reduction document, clear LinUCB derivation, practical cascade design, measured outcomes cited |
| 08 | Chain Reputation | 8/10 | Thorough coverage of on-chain primitives, but IronClaw integration feels distant from current priorities |
| 09 | Budget Composition | 8/10 | Innovative VCG auction approach, good U-shaped attention research, but high complexity for uncertain payoff |
| 10 | Universal Engram | 9/10 | Central data type well-documented, 7-axis scoring and 4 decay variants thoroughly explained |
| 11 | Mathematical Primitives | 8/10 | Impressive mathematical rigor, but most content is research-grade with limited near-term applicability beyond robust statistics |
| 12 | Code Intelligence | 8/10 | Good multi-modal indexing architecture, but high effort and uncertain IronClaw fit |
| 13 | Cognitive Architecture | 9/10 | Excellent theoretical grounding (Buzsaki, Kahneman, Beer VSM, Friston), strong roko-to-IronClaw mapping |
| 14 | Runtime Infrastructure | 8/10 | Solid coverage of EventBus, cancellation, lifecycle, but split across too many sub-topics |
| 15 | Orchestrator & Swarm | 8/10 | Good event-sourcing and DAG scheduling coverage, but some sections overlap with doc 04 |
| 16 | Plugin & Extension | 7/10 | Thorough roko coverage but lower IronClaw relevance since IronClaw already has a mature extension system |
| 17 | Agent Patterns | 9/10 | High-value patterns, especially Metacognitive Monitor. Well-structured pattern catalog |
| 18 | Integration Roadmap | 8/10 | Excellent phased plan with step-by-step implementation guides, but does not cover docs 20-28 |
| 19 | Priority Matrix | 6/10 | Good methodology description and quadrant diagram, but ROI formula produces wrong values, undermining trust |
| 20 | Persistence Storage | 8/10 | Clear exposition of append-only JSONL pattern and Store trait, good IronClaw comparison |
| 21 | MCP/ACP Integration | 8/10 | Comprehensive protocol reference, good ACP vs MCP distinction, relevant to IronClaw's existing MCP support |
| 22 | Language Support | 8/10 | Detailed language provider architecture, good dual-mode parser coverage |
| 23 | Control Plane | 8/10 | Thorough API server architecture, good hub-and-spoke model description |
| 24 | Smart Contracts | 8/10 | Complete Solidity contract reference, good NEAR porting analysis |
| 25 | Research Citations | 9/10 | Comprehensive bibliography with 100+ references, well-organized by topic with crate mappings |
| 26 | Roko Architecture Overview | 9/10 | Excellent single-document architecture reference, good comparison tables with IronClaw |
| 27 | v2-Depth Research | 7/10 | Useful catalog of 155 depth documents, but hard to verify accuracy without access to all source files |
| 28 | Plans Catalog | 7/10 | Useful catalog of implementation plans, but some plans may be outdated |

**Average Score**: 8.1/10

---

## 5. Recommendations

### Immediate (fix before sharing these documents)

1. **Update the INDEX (00)** to include documents 20-28 in the table, statistics, and reading guides.
2. **Fix the ROI formula** in doc 19. Either recompute all values using the stated formula, or document the actual formula used.
3. **Align priority ratings** in document headers (01-17) with the priority matrix (19) rankings.

### Short-Term (improve quality)

4. **Add cross-references** from documents 01-19 to relevant documents 20-28 (see Section 3.1 table).
5. **Add a terminology glossary** to the INDEX or as a standalone document, covering the Engram/Signal duality and other v1-vs-v2 term differences.
6. **Reconcile effort estimates** between the roadmap (18) and priority matrix (19), ensuring they match the stated rubric definitions.
7. **Extend the priority matrix** to rank concepts from documents 20-28.

### Medium-Term (fill gaps)

8. **Write a roko testing infrastructure document** covering test patterns, verification methodology, and testcontainers usage.
9. **Write a roko TUI/dashboard document** comparing it to IronClaw's ratatui-based CLI/TUI.
10. **Write a roko configuration system document** comparing it to IronClaw's env-based configuration.

### Structural

11. **Standardize crate counts** across all documents (18 core crates, 32 total, or whatever the accurate number is).
12. **Add document version/date stamps** to each document header so readers know when it was last updated.
13. **Consider splitting the INDEX reading guides** by audience (developer, architect, researcher) since the current guides overlap significantly.

---

## 6. Summary Statistics

| Metric | Value |
|--------|-------|
| Documents audited | 29 |
| Consistency issues found | 5 major, 3 minor |
| Factual conflicts found | 4 |
| Missing cross-references | 7 document pairs |
| Content gaps identified | 5 |
| Average quality score | 8.1 / 10 |
| Documents scoring 9+ | 9 (31%) |
| Documents scoring 7 or below | 4 (14%) |
| Highest-scoring documents | 01, 02, 05, 06, 07, 10, 13, 17, 25, 26 |
| Lowest-scoring documents | 00 (stale index), 19 (broken formula) |

---

## 7. Overall Assessment

This is a high-quality document collection with deep technical content. The individual concept documents (01-17) are among the best technical writing I have seen for technology transfer analysis -- each provides sufficient context for a first-time reader, backs claims with actual code, and provides concrete integration recommendations.

The weaknesses are in the meta-layer: the index is incomplete, the priority matrix's math is wrong, and cross-references between the two batches (01-19 and 20-28) are absent. These are all fixable with modest effort and do not undermine the technical value of the individual documents.

The most impactful improvement would be updating the INDEX (00) and fixing the priority matrix formula (19), as these are the two documents most likely to be read first by someone navigating the collection.
