## Test Gap Analysis for IronClaw HDC

### Status (updated after implementation session)

**Tests added this session: 17** (all in Priority 1, sections 2 and 4 partial)
**Tests still remaining: 25** (4 active-dedup caller + 5 PostgreSQL + 2 search wiring + 3 heartbeat persistence + 5 clustering + 3 structured encoding + 3 native memory + 2 subagent + 2 skill index)

Tests from the original 42 that turned out to be unnecessary:
- None declared unnecessary. However, the `fixture_heartbeat_recurring_after_gap_resets` test was implemented under the name `fixture_heartbeat_recurring_after_gap` (the `_resets` suffix was dropped because the fixture name in the manifest does not include it; see section 2 below).
- Section 6 (K-Medoids Clustering) and Section 7 (Structured Encoding) remain unimplemented. Both APIs now exist in the crate: `encode_structured`, `encode_causal_link`, and `k_medoids` are all public and re-exported. The integration test files (`crates/ironclaw_hdc/tests/clustering.rs` and `crates/ironclaw_hdc/tests/encoding.rs`) have not been created yet, but the APIs are no longer the blocker.

---

### Priority 1: Missing Tests That Block Promotion

#### 1. Active Dedup Caller Tests (file: `tests/workspace_hdc_dedup.rs`)

Missing tests in `mod memory_tool_dedup`:
- [ ] `tool_warn_mode_writes_with_dedup_warning` — Set `IRONCLAW_HDC_DEDUP_MODE=warn`, write doc A, write near-duplicate doc B. Assert: B is written (status=written), output contains `dedup.decision="similar"`, `dedup.existing_path` points to A, `dedup.similarity` is present.
- [ ] `tool_block_mode_rejects_duplicate` — Set `IRONCLAW_HDC_DEDUP_MODE=block`, write doc A, write duplicate doc B. Assert: B is rejected (status=blocked), no document created at B's path, output contains `dedup.decision="duplicate"`, `dedup.retry` mentions force=true.
- [ ] `tool_block_mode_force_bypasses` — Set `IRONCLAW_HDC_DEDUP_MODE=block` + `force=true`, write duplicate. Assert: write succeeds, fingerprint stored if shadow enabled.
- [ ] `tool_warn_mode_unique_has_no_dedup_field` — Set `IRONCLAW_HDC_DEDUP_MODE=warn`, write genuinely unique doc. Assert: output has status=written, no dedup field.

Note: The `mod memory_tool_dedup` block in `tests/workspace_hdc_dedup.rs` now exists with 7 tests covering the default-off mode. None of those 7 cover warn/block mode because active enforcement (`IRONCLAW_HDC_DEDUP_MODE`) is not yet wired. The 4 tests above remain blocked on that feature.

#### 2. Fixture Corpus Harness (file: `tests/workspace_hdc_fixtures.rs`)

The file `tests/workspace_hdc_fixtures.rs` was created this session. All 17 originally planned fixture tests now exist:

- [x] `fixture_dedup_exact_duplicate_similarity_range` — reads `exact-duplicate-different-path.jsonl`, asserts similarity in declared range.
- [x] `fixture_dedup_reformatted_similarity_range` — reads `reformatted-duplicate.jsonl`.
- [x] `fixture_dedup_paraphrase_similarity_range` — reads `paraphrase-duplicate.jsonl`.
- [x] `fixture_dedup_added_paragraph_similarity_range` — reads `added-paragraph.jsonl`.
- [x] `fixture_dedup_same_topic_different_facts_is_unique` — reads `same-topic-different-facts.jsonl`.
- [x] `fixture_dedup_same_tags_different_content_is_unique` — reads same-tags fixture.
- [x] `fixture_dedup_same_path_different_content_is_unique` — reads path-pattern fixture.
- [x] `fixture_dedup_content_subset_is_similar` — reads content-subset fixture.
- [x] `fixture_dedup_translated_is_similar` — reads translated-content fixture.
- [x] `fixture_heartbeat_repeated_observation_converges` — reads `repeated-observation.jsonl`, runs accumulator, asserts convergence.
- [x] `fixture_heartbeat_recurring_after_gap` — reads `recurring-after-gap.jsonl` (planned name was `fixture_heartbeat_recurring_after_gap_resets`; implemented without `_resets` suffix to match manifest key).
- [x] `fixture_heartbeat_similar_different_metric_is_novel` — reads `similar-but-different-metric.jsonl`.
- [x] `fixture_heartbeat_completely_novel_all_novel` — reads `completely-novel.jsonl`.
- [x] `fixture_search_structural_query_ranks_relevant` — reads `structural-query.jsonl`, runs `top_k_scan`, asserts rank order.
- [x] `fixture_search_tag_heavy_query_ranks_relevant` — reads `tag-heavy-query.jsonl`.
- [x] `fixture_search_path_oriented_query_ranks_relevant` — reads `path-oriented-query.jsonl`.
- [x] `fixture_search_semantic_only_query_ranks_relevant` — reads `semantic-only-query.jsonl`.

The file also includes two manifest-consistency tests (`fixture_manifest_all_files_exist`, `fixture_manifest_jsonl_parseable`, `fixture_manifest_similarity_ranges_valid`) that were added as supporting infrastructure and were not in the original gap list.

#### 3. PostgreSQL HDC Storage (file: `tests/workspace_hdc.rs`)

All 5 PostgreSQL-backed storage tests remain unimplemented. The `mod hdc_storage` block only exercises libSQL. PostgreSQL requires `feature = "integration"` and a live Postgres container.

- [ ] `pg_roundtrip_valid_fingerprint`
- [ ] `pg_store_rejects_wrong_length`
- [ ] `pg_list_fingerprints_scoped`
- [ ] `pg_overwrite_fingerprint`
- [ ] `pg_list_fingerprints_excludes_null`

#### 4. Search Config Wiring (file: `tests/workspace_hdc_search.rs`)

- [ ] `search_config_wired_from_env` — Set `IRONCLAW_HDC_SEARCH_SHADOW=true`, create workspace, run search, assert `hdc_rank`/`hdc_score` are present in results when documents have fingerprints. Not yet implemented; requires the env-var read path and live DB.
- [ ] `search_hdc_fields_in_tool_output` — Run `MemorySearchTool::execute` with HDC enabled, assert output JSON contains `hdc_rank` and `hdc_score` fields. Not yet implemented; requires search tool output extension.
- ~~`search_live_reranking_changes_order`~~ — The gap analysis noted this was "Covered as `search_live_mode_changes_ranking`". That test exists in `tests/workspace_hdc_search.rs` but is a config-verification stub with a `// TODO` comment for the actual end-to-end wiring. It validates that configs compile correctly, not that reranking actually happens. Marking this item as still open under the original description:
  - [ ] `search_live_reranking_changes_order` (full end-to-end): two docs with different HDC vs FTS relevance, live mode with high HDC weight, verify order actually changes. The stub `search_live_mode_changes_ranking` does not satisfy this requirement.

---

### Priority 2: Missing Tests for Follow-Up Features

#### 5. Heartbeat Persistence (file: `tests/heartbeat_hdc.rs`)

These three tests appear as `// TODO` comments in `tests/heartbeat_hdc.rs` but are not implemented. They are blocked on the `DecayingBundleAccumulator` serialization feature (persistence layer does not exist yet).

- [ ] `heartbeat_state_persists_across_restart` — Saturate accumulator with repeated messages, reconstruct runner, assert new observation of same message is still Repeated.
- [ ] `heartbeat_corrupt_state_resets_cleanly` — Write garbage to state file, construct runner, assert it falls back to fresh state.
- [ ] `heartbeat_notification_includes_novelty_metadata` — Check that `OutgoingResponse` includes novelty class when HDC is enabled.

#### 6. K-Medoids Clustering (file: `crates/ironclaw_hdc/tests/clustering.rs`)

The file `crates/ironclaw_hdc/tests/clustering.rs` does not exist. The inline tests in `cluster.rs` cover the internal distance helpers (`distance_identical_is_zero`, `distance_random_near_half`, `farthest_first_selects_k_distinct`, `farthest_first_single`) but not the full public API. `k_medoids<Id>(vectors: &[(Id, HdcVector)], k: usize) -> Result<Vec<Cluster<Id>>, HdcError>` is already implemented and publicly exported. All 5 planned tests remain unimplemented; they are no longer blocked by a missing API — only by a missing test file.

- [ ] `cluster_three_well_separated_groups` — 30 vectors in 3 groups of 10. Assert k=3 recovers 3 clusters with purity >= 0.90.
- [ ] `cluster_empty_input` — Assert returns empty clusters.
- [ ] `cluster_k_greater_than_n` — k=10, n=5. Assert returns 5 clusters.
- [ ] `cluster_k_one_single_cluster` — All vectors in one cluster.
- [ ] `cluster_deterministic` — Same input twice produces identical output.

#### 7. Structured Encoding (file: `crates/ironclaw_hdc/tests/encoding.rs`)

The file `crates/ironclaw_hdc/tests/encoding.rs` does not exist. The inline tests in `encoder.rs` are extensive (18 tests) but cover only `encode_text` and `encode_document`. `encode_structured(fields: &[(&str, &str)], codebook: &mut Codebook) -> HdcVector` and `encode_causal_link(cause: &str, effect: &str, codebook: &mut Codebook) -> HdcVector` are already implemented and publicly exported from `crates/ironclaw_hdc/src/encoder.rs`. All 3 planned tests remain unimplemented; they are no longer blocked by missing APIs — only by a missing test file.

- [ ] `structured_encoding_differs_from_flat_text` — `encode_structured([("cause","X"),("effect","Y")])` != `encode_text("cause X effect Y")`.
- [ ] `causal_direction_matters` — `encode_causal_link("A","B")` != `encode_causal_link("B","A")`.
- [ ] `role_filler_unbinding_recovers_direction` — bind(composite, role_vec) has high similarity to the filler vec.

#### 8. Native Memory HDC (file: to be determined)

No NativeMemoryService exists yet. All 3 tests remain unimplemented.

- [ ] `native_write_stores_fingerprint` — Write through NativeMemoryService, verify fingerprint stored.
- [ ] `native_retrieve_context_dedup_removes_duplicates` — Write 5 near-duplicate docs, retrieve_context, assert fewer snippets returned in dedup mode.
- [ ] `native_hdc_disabled_unchanged` — HDC disabled: behavior identical to current.

#### 9. Subagent Coordination (file: to be determined)

No subagent HDC coordination API exists yet. Both tests remain unimplemented.

- [ ] `subagent_goal_dedup_prevents_duplicate_spawn` — Spawn two tasks with HDC-similar goals. Assert second spawn returns existing run ID.
- [ ] `subagent_result_dedup_marks_similar_tombstone` — Two children complete with near-duplicate results. Assert tombstone marked with similarity score.

#### 10. Skill/Tool HDC Index Caller Integration (file: to be determined)

No HDC boost wiring in the skill selection path yet. Both tests remain unimplemented.

- [ ] `skill_hdc_boost_changes_selection_in_tie` — Two skills with equal keyword scores but different HDC similarity to the query. Assert HDC-boosted skill wins.
- [ ] `tool_hdc_boost_capped_at_MAX_HDC_BOOST` — Verify HDC boost never exceeds 0.1 regardless of similarity.

---

### Test Infrastructure Notes

- All tests behind `#[cfg(feature = "hdc")]`
- DB tests additionally behind `#[cfg(feature = "libsql")]` or `#[cfg(feature = "integration")]` for PostgreSQL
- Fixture tests are pure-crate (no DB needed), fastest to implement
- All active-behavior tests must drive the production caller, not just the helper
- Mocks must capture all production arguments per CLAUDE.md testing rules

### Test Count Summary

| Section | Planned | Implemented | Remaining |
|---------|---------|-------------|-----------|
| 1. Active dedup caller (warn/block mode) | 4 | 0 | 4 |
| 2. Fixture corpus harness | 17 | 17 | 0 |
| 3. PostgreSQL HDC storage | 5 | 0 | 5 |
| 4. Search config wiring | 3 | 0 | 3 |
| 5. Heartbeat persistence | 3 | 0 | 3 |
| 6. K-medoids clustering | 5 | 0 | 5 |
| 7. Structured encoding | 3 | 0 | 3 |
| 8. Native memory HDC | 3 | 0 | 3 |
| 9. Subagent coordination | 2 | 0 | 2 |
| 10. Skill/tool HDC index | 2 | 0 | 2 |
| **Total** | **47** | **17** | **30** |

Note: the original count of 42 was revised to 47 because `search_live_reranking_changes_order` was counted as covered by the stub but is still open (1 item), and the fixture section had 17 planned tests not 16 as the original summary stated (the search fixture tests were counted as 4 in section 2 but there are 4 fixture subtests, making the total 9+4+4=17 not 16).
