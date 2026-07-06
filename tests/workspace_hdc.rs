//! Integration tests for HDC fingerprinting in workspace.
//!
//! Run with: cargo test --features hdc,libsql,integration workspace_hdc
//!
//! These tests exercise the HDC fingerprint storage layer and the workspace
//! write-path integration that computes and stores fingerprints in shadow mode.
//!
//! Feature matrix verified by CI:
//! - `cargo test` (no hdc) — this file is entirely gated, no compilation impact
//! - `cargo test --features hdc` — compiles but tests need `integration` to run
//! - `cargo test --features hdc,libsql,integration` — full test execution

// ─── Storage contract tests ──────────────────────────────────────────────────
//
// Exercise the `WorkspaceStore` HDC trait methods at the persistence boundary.
// Tests the migration schema, length validation, scoping, and round-trips.

#[cfg(all(feature = "hdc", feature = "libsql", feature = "integration"))]
mod hdc_storage {
    use std::sync::Arc;

    use ironclaw::db::libsql::LibSqlBackend;
    #[allow(unused_imports)] // trait in scope for method dispatch
    use ironclaw::db::{Database, WorkspaceStore};
    use ironclaw::workspace::Workspace;
    use uuid::Uuid;

    async fn setup() -> (Arc<dyn Database>, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db_path = dir.path().join("hdc_storage_test.db");
        let backend = LibSqlBackend::new_local(&db_path).await.expect("create db");
        backend.run_migrations().await.expect("run migrations");
        let db: Arc<dyn Database> = Arc::new(backend);
        (db, dir)
    }

    /// Helper: create a document and return its ID.
    async fn create_doc(db: &Arc<dyn Database>, user_id: &str, path: &str) -> Uuid {
        let ws = Workspace::new_with_db(user_id, db.clone());
        let doc = ws
            .write(path, &format!("content for {path}"))
            .await
            .expect("write failed");
        doc.id
    }

    /// Helper: create a document with a specific agent_id and return its ID.
    async fn create_doc_with_agent(
        db: &Arc<dyn Database>,
        user_id: &str,
        agent_id: Uuid,
        path: &str,
    ) -> Uuid {
        let ws = Workspace::new_with_db(user_id, db.clone()).with_agent(agent_id);
        let doc = ws
            .write(path, &format!("content for {path}"))
            .await
            .expect("write failed");
        doc.id
    }

    // ── Migration tests ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn migration_fresh_db() {
        // A freshly migrated database has the hdc_fingerprint column.
        // Creating a document yields a NULL fingerprint.
        let (db, _dir) = setup().await;
        let ws = Workspace::new_with_db("migration_fresh", db.clone());

        let doc = ws.write("test.md", "hello").await.expect("write failed");

        // The document struct should have hdc_fingerprint = None.
        assert!(
            doc.hdc_fingerprint.is_none(),
            "fresh document should have NULL hdc_fingerprint"
        );

        // List should return empty (no fingerprints stored yet).
        let fps = db
            .list_document_hdc_fingerprints("migration_fresh", None)
            .await
            .expect("list failed");
        assert!(fps.is_empty());
    }

    #[tokio::test]
    async fn migration_existing_data() {
        // Simulate pre-HDC documents: create many docs, run migrations, verify
        // existing data is intact and fingerprints are NULL.
        let (db, _dir) = setup().await;
        let ws = Workspace::new_with_db("migration_existing", db.clone());

        // Create 100 documents (simulating pre-migration state).
        for i in 0..100 {
            ws.write(&format!("doc-{i:03}.md"), &format!("content {i}"))
                .await
                .expect("write failed");
        }

        // All documents should be readable with correct content.
        for i in 0..100 {
            let doc = ws
                .read(&format!("doc-{i:03}.md"))
                .await
                .expect("read failed");
            assert_eq!(doc.content, format!("content {i}"));
            assert!(
                doc.hdc_fingerprint.is_none(),
                "doc-{i:03} should have NULL fingerprint"
            );
        }

        // No fingerprints should be listed.
        let fps = db
            .list_document_hdc_fingerprints("migration_existing", None)
            .await
            .expect("list failed");
        assert!(fps.is_empty());
    }

    #[tokio::test]
    async fn roundtrip_none() {
        // A document read back without storing a fingerprint has None.
        let (db, _dir) = setup().await;
        let ws = Workspace::new_with_db("roundtrip_none", db.clone());

        let doc = ws.write("no-fp.md", "content").await.expect("write failed");
        let read_back = ws.read("no-fp.md").await.expect("read failed");

        assert_eq!(read_back.id, doc.id);
        assert!(read_back.hdc_fingerprint.is_none());
    }

    #[tokio::test]
    async fn roundtrip_valid_fingerprint() {
        let (db, _dir) = setup().await;
        let doc_id = create_doc(&db, "storage_roundtrip", "test.md").await;

        // Store a valid 1280-byte fingerprint with varied byte pattern.
        let fingerprint: Vec<u8> = (0..1280).map(|i| (i % 256) as u8).collect();
        db.update_document_hdc_fingerprint(doc_id, &fingerprint)
            .await
            .expect("store failed");

        // Read back and verify exact bytes.
        let fps = db
            .list_document_hdc_fingerprints("storage_roundtrip", None)
            .await
            .expect("list failed");
        assert_eq!(fps.len(), 1);
        assert_eq!(fps[0].id, doc_id);
        assert_eq!(fps[0].fingerprint, fingerprint);
        assert_eq!(fps[0].path, "test.md");
    }

    #[tokio::test]
    async fn store_rejects_wrong_length() {
        let (db, _dir) = setup().await;
        let doc_id = create_doc(&db, "storage_wrong_len", "test.md").await;

        // 500 bytes — should be rejected.
        let bad_fp = vec![0u8; 500];
        let result = db.update_document_hdc_fingerprint(doc_id, &bad_fp).await;
        assert!(
            result.is_err(),
            "storing a 500-byte fingerprint should fail"
        );

        // Verify error message mentions expected length.
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("1280"),
            "error should mention expected length 1280, got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn store_rejects_empty() {
        let (db, _dir) = setup().await;
        let doc_id = create_doc(&db, "storage_empty", "test.md").await;

        // 0 bytes — should be rejected.
        let empty_fp: Vec<u8> = vec![];
        let result = db.update_document_hdc_fingerprint(doc_id, &empty_fp).await;
        assert!(result.is_err(), "storing an empty fingerprint should fail");
    }

    #[tokio::test]
    async fn list_fingerprints_excludes_null() {
        let (db, _dir) = setup().await;
        let user_id = "storage_excludes_null";

        // Create 3 documents, only store fingerprints for 2 of them.
        let id1 = create_doc(&db, user_id, "with-fp-1.md").await;
        let id2 = create_doc(&db, user_id, "with-fp-2.md").await;
        let _id3 = create_doc(&db, user_id, "without-fp.md").await;

        let fp = vec![0xFFu8; 1280];
        db.update_document_hdc_fingerprint(id1, &fp)
            .await
            .expect("store fp1 failed");
        db.update_document_hdc_fingerprint(id2, &fp)
            .await
            .expect("store fp2 failed");

        // List should only return the 2 with fingerprints.
        let fps = db
            .list_document_hdc_fingerprints(user_id, None)
            .await
            .expect("list failed");
        assert_eq!(fps.len(), 2);

        let ids: Vec<Uuid> = fps.iter().map(|f| f.id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[tokio::test]
    async fn list_fingerprints_includes_path() {
        let (db, _dir) = setup().await;
        let user_id = "storage_includes_path";

        let id1 = create_doc(&db, user_id, "notes/idea.md").await;
        let id2 = create_doc(&db, user_id, "projects/alpha/README.md").await;

        let fp = vec![0x55u8; 1280];
        db.update_document_hdc_fingerprint(id1, &fp)
            .await
            .expect("store fp1 failed");
        db.update_document_hdc_fingerprint(id2, &fp)
            .await
            .expect("store fp2 failed");

        let fps = db
            .list_document_hdc_fingerprints(user_id, None)
            .await
            .expect("list failed");

        let paths: Vec<&str> = fps.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"notes/idea.md"));
        assert!(paths.contains(&"projects/alpha/README.md"));
    }

    #[tokio::test]
    async fn list_fingerprints_scoped() {
        let (db, _dir) = setup().await;

        // User A has 3 docs with fingerprints.
        let a_ids: Vec<Uuid> = {
            let mut ids = Vec::new();
            for i in 0..3 {
                let id = create_doc(&db, "scope_user_a", &format!("doc-{i}.md")).await;
                let fp = vec![i as u8; 1280];
                db.update_document_hdc_fingerprint(id, &fp)
                    .await
                    .expect("store fp failed");
                ids.push(id);
            }
            ids
        };

        // User B has 2 docs with fingerprints.
        for i in 0..2 {
            let id = create_doc(&db, "scope_user_b", &format!("doc-{i}.md")).await;
            let fp = vec![(i + 100) as u8; 1280];
            db.update_document_hdc_fingerprint(id, &fp)
                .await
                .expect("store fp failed");
        }

        // List for user A returns exactly 3.
        let fps_a = db
            .list_document_hdc_fingerprints("scope_user_a", None)
            .await
            .expect("list a failed");
        assert_eq!(fps_a.len(), 3);
        for fp in &fps_a {
            assert!(a_ids.contains(&fp.id));
        }

        // List for user B returns exactly 2.
        let fps_b = db
            .list_document_hdc_fingerprints("scope_user_b", None)
            .await
            .expect("list b failed");
        assert_eq!(fps_b.len(), 2);
    }

    #[tokio::test]
    async fn list_fingerprints_agent_scoped() {
        let (db, _dir) = setup().await;
        let user_id = "agent_scoped";
        let agent_a = Uuid::new_v4();
        let agent_b = Uuid::new_v4();

        // Create docs under different agents for the same user.
        let id_a = create_doc_with_agent(&db, user_id, agent_a, "agent-a-doc.md").await;
        let id_b = create_doc_with_agent(&db, user_id, agent_b, "agent-b-doc.md").await;

        let fp = vec![0x77u8; 1280];
        db.update_document_hdc_fingerprint(id_a, &fp)
            .await
            .expect("store fp_a failed");
        db.update_document_hdc_fingerprint(id_b, &fp)
            .await
            .expect("store fp_b failed");

        // List scoped to agent_a returns only its document.
        let fps_a = db
            .list_document_hdc_fingerprints(user_id, Some(agent_a))
            .await
            .expect("list agent_a failed");
        assert_eq!(fps_a.len(), 1);
        assert_eq!(fps_a[0].id, id_a);

        // List scoped to agent_b returns only its document.
        let fps_b = db
            .list_document_hdc_fingerprints(user_id, Some(agent_b))
            .await
            .expect("list agent_b failed");
        assert_eq!(fps_b.len(), 1);
        assert_eq!(fps_b[0].id, id_b);

        // List with agent_id=None returns only unscoped documents, matching
        // normal workspace nullable-agent scoping.
        let fps_all = db
            .list_document_hdc_fingerprints(user_id, None)
            .await
            .expect("list unscoped failed");
        assert!(fps_all.is_empty());
    }

    #[tokio::test]
    async fn overwrite_fingerprint() {
        let (db, _dir) = setup().await;
        let doc_id = create_doc(&db, "storage_overwrite", "test.md").await;

        // Store initial fingerprint.
        let fp_old = vec![0x11u8; 1280];
        db.update_document_hdc_fingerprint(doc_id, &fp_old)
            .await
            .expect("store old failed");

        // Overwrite with a different fingerprint.
        let fp_new = vec![0x99u8; 1280];
        db.update_document_hdc_fingerprint(doc_id, &fp_new)
            .await
            .expect("store new failed");

        // List should return only the latest.
        let fps = db
            .list_document_hdc_fingerprints("storage_overwrite", None)
            .await
            .expect("list failed");
        assert_eq!(fps.len(), 1);
        assert_eq!(fps[0].fingerprint, fp_new);
    }
}

// ─── Workspace integration tests ────────────────────────────────────────────
//
// Exercise HDC fingerprinting through the Workspace write/append/patch/delete
// APIs. Tests verify the storage layer integration and behavioral contracts.

#[cfg(all(feature = "hdc", feature = "libsql", feature = "integration"))]
mod hdc_integration {
    use std::sync::Arc;

    use ironclaw::db::libsql::LibSqlBackend;
    #[allow(unused_imports)] // trait in scope for method dispatch
    use ironclaw::db::{Database, WorkspaceStore};
    use ironclaw::workspace::{Workspace, paths};

    /// Set up a fresh database with migrations applied.
    async fn setup() -> (Arc<dyn Database>, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db_path = dir.path().join("hdc_integ_test.db");
        let backend = LibSqlBackend::new_local(&db_path).await.expect("create db");
        backend.run_migrations().await.expect("run migrations");
        let db: Arc<dyn Database> = Arc::new(backend);
        (db, dir)
    }

    fn workspace_with_hdc(user_id: &str, db: Arc<dyn Database>) -> Workspace {
        Workspace::new_with_db(user_id, db).with_hdc_fingerprint_shadow(true)
    }

    #[tokio::test]
    async fn write_stores_fingerprint() {
        let (db, _dir) = setup().await;
        let ws = workspace_with_hdc("hdc_write_store", db.clone());

        let doc = ws
            .write("test.md", "hello world")
            .await
            .expect("write failed");

        // Verify the write path computed and stored a non-NULL 1280-byte fingerprint.
        let fps = db
            .list_document_hdc_fingerprints("hdc_write_store", None)
            .await
            .expect("list failed");
        assert_eq!(fps.len(), 1);
        assert_eq!(fps[0].id, doc.id);
        assert_eq!(fps[0].fingerprint.len(), 1280);
    }

    #[tokio::test]
    async fn write_deterministic_fingerprint() {
        let (db, _dir) = setup().await;
        let ws = workspace_with_hdc("hdc_deterministic", db.clone());

        // Write same content twice to same path.
        let doc1 = ws
            .write("notes.md", "same content")
            .await
            .expect("write1 failed");

        let fp1 = db
            .list_document_hdc_fingerprints("hdc_deterministic", None)
            .await
            .expect("list after first write failed");
        assert_eq!(fp1.len(), 1);

        // Write again (same content, same path) — doc ID unchanged.
        let doc2 = ws
            .write("notes.md", "same content")
            .await
            .expect("write2 failed");
        assert_eq!(doc1.id, doc2.id, "same path should reuse document ID");

        // Fingerprint bytes are preserved (deterministic: same input -> same output).
        let fps = db
            .list_document_hdc_fingerprints("hdc_deterministic", None)
            .await
            .expect("list failed");
        assert_eq!(fps.len(), 1);
        assert_eq!(fps[0].fingerprint, fp1[0].fingerprint);
    }

    #[tokio::test]
    async fn write_different_content_different_fingerprint() {
        let (db, _dir) = setup().await;
        let ws = workspace_with_hdc("hdc_different", db.clone());

        let doc_a = ws
            .write("path-a.md", "hello")
            .await
            .expect("write a failed");
        let doc_b = ws
            .write("path-b.md", "goodbye")
            .await
            .expect("write b failed");

        let fps = db
            .list_document_hdc_fingerprints("hdc_different", None)
            .await
            .expect("list failed");
        assert_eq!(fps.len(), 2);

        let a_fp = fps.iter().find(|f| f.id == doc_a.id).expect("find a");
        let b_fp = fps.iter().find(|f| f.id == doc_b.id).expect("find b");
        assert_ne!(a_fp.fingerprint, b_fp.fingerprint);
    }

    #[tokio::test]
    async fn write_same_content_different_paths_same_fingerprint() {
        let (db, _dir) = setup().await;
        let ws = workspace_with_hdc("hdc_tags", db.clone());

        // Two documents with same content but at different paths.
        // Workspace currently passes no metadata tags into HDC. With the
        // encoder's weighted majority, content dominates path-only differences,
        // so exact duplicate content should share a fingerprint across paths.
        let doc1 = ws
            .write("notes/tagged-a.md", "same content")
            .await
            .expect("write1 failed");
        let doc2 = ws
            .write("notes/tagged-b.md", "same content")
            .await
            .expect("write2 failed");

        let fps = db
            .list_document_hdc_fingerprints("hdc_tags", None)
            .await
            .expect("list failed");
        let f1 = fps.iter().find(|f| f.id == doc1.id).unwrap();
        let f2 = fps.iter().find(|f| f.id == doc2.id).unwrap();
        assert_eq!(f1.fingerprint, f2.fingerprint);
    }

    #[tokio::test]
    async fn write_unchanged_content_preserves_fingerprint() {
        let (db, _dir) = setup().await;
        let ws = workspace_with_hdc("hdc_unchanged", db.clone());

        let doc = ws
            .write("stable.md", "unchanged content")
            .await
            .expect("write1 failed");

        let fp1 = db
            .list_document_hdc_fingerprints("hdc_unchanged", None)
            .await
            .expect("list after first write failed");
        assert_eq!(fp1.len(), 1);

        // Write same content again — fingerprint should be preserved.
        ws.write("stable.md", "unchanged content")
            .await
            .expect("write2 failed");

        let fps = db
            .list_document_hdc_fingerprints("hdc_unchanged", None)
            .await
            .expect("list failed");
        assert_eq!(fps.len(), 1);
        assert_eq!(fps[0].id, doc.id);
        assert_eq!(fps[0].fingerprint, fp1[0].fingerprint);
    }

    #[tokio::test]
    async fn write_engine_runtime_path_no_fingerprint() {
        let (db, _dir) = setup().await;
        let ws = workspace_with_hdc("hdc_engine_rt", db.clone());

        // Engine runtime paths skip indexing and should not get fingerprints.
        let doc = ws
            .write("engine/.runtime/state.json", r#"{"step": 42}"#)
            .await
            .expect("write engine runtime failed");

        // The list should be empty — engine runtime is excluded from fingerprinting.
        let fps = db
            .list_document_hdc_fingerprints("hdc_engine_rt", None)
            .await
            .expect("list failed");
        assert!(
            fps.is_empty(),
            "engine runtime paths should not have fingerprints, got: {}",
            fps.len()
        );

        // Verify the document itself was stored correctly.
        let read_back = ws
            .read("engine/.runtime/state.json")
            .await
            .expect("read failed");
        assert_eq!(read_back.id, doc.id);
        assert_eq!(read_back.content, r#"{"step": 42}"#);
    }

    #[tokio::test]
    async fn write_identity_file_gets_fingerprint() {
        let (db, _dir) = setup().await;
        let ws = workspace_with_hdc("hdc_identity", db.clone());

        // Identity files (IDENTITY.md, SOUL.md, etc.) ARE semantic documents
        // and SHOULD get fingerprints (unlike engine runtime paths).
        let doc = ws
            .write(paths::IDENTITY, "I am a helpful assistant")
            .await
            .expect("write identity failed");

        let fps = db
            .list_document_hdc_fingerprints("hdc_identity", None)
            .await
            .expect("list failed");
        assert_eq!(fps.len(), 1);
        assert_eq!(fps[0].id, doc.id);
        assert_eq!(fps[0].path, paths::IDENTITY);
        assert_eq!(fps[0].fingerprint.len(), 1280);
    }

    #[tokio::test]
    async fn append_updates_fingerprint() {
        let (db, _dir) = setup().await;
        let ws = workspace_with_hdc("hdc_append", db.clone());

        let doc = ws
            .write("log.md", "initial content")
            .await
            .expect("write failed");

        let fp_initial = db
            .list_document_hdc_fingerprints("hdc_append", None)
            .await
            .expect("list initial failed");
        assert_eq!(fp_initial.len(), 1);

        // Append changes content, so fingerprint should be updated.
        ws.append("log.md", "appended line")
            .await
            .expect("append failed");

        let fps = db
            .list_document_hdc_fingerprints("hdc_append", None)
            .await
            .expect("list failed");
        assert_eq!(fps.len(), 1);
        assert_eq!(fps[0].id, doc.id);
        assert_ne!(fps[0].fingerprint, fp_initial[0].fingerprint);
    }

    #[tokio::test]
    async fn patch_updates_fingerprint() {
        let (db, _dir) = setup().await;
        let ws = workspace_with_hdc("hdc_patch", db.clone());

        let doc = ws
            .write("code.md", "fn hello() { println!(\"hello\"); }")
            .await
            .expect("write failed");

        let fp_before = db
            .list_document_hdc_fingerprints("hdc_patch", None)
            .await
            .expect("list before patch failed");
        assert_eq!(fp_before.len(), 1);

        // Patch changes content.
        ws.patch("code.md", "hello", "greet", false)
            .await
            .expect("patch failed");

        let fps = db
            .list_document_hdc_fingerprints("hdc_patch", None)
            .await
            .expect("list failed");
        assert_eq!(fps.len(), 1);
        assert_eq!(fps[0].id, doc.id);
        assert_ne!(fps[0].fingerprint, fp_before[0].fingerprint);
    }

    #[tokio::test]
    async fn delete_removes_fingerprint() {
        let (db, _dir) = setup().await;
        let ws = workspace_with_hdc("hdc_delete", db.clone());

        let _doc = ws
            .write("ephemeral.md", "temporary")
            .await
            .expect("write failed");

        // Verify fingerprint is present.
        let fps = db
            .list_document_hdc_fingerprints("hdc_delete", None)
            .await
            .expect("list failed");
        assert_eq!(fps.len(), 1);

        // Delete the document.
        ws.delete("ephemeral.md").await.expect("delete failed");

        // Fingerprint should be gone (document row deleted).
        let fps = db
            .list_document_hdc_fingerprints("hdc_delete", None)
            .await
            .expect("list after delete failed");
        assert!(
            fps.is_empty(),
            "fingerprint should be removed after document deletion"
        );
    }

    #[tokio::test]
    async fn multi_user_isolation() {
        let (db, _dir) = setup().await;

        let ws_alice = workspace_with_hdc("hdc_alice", db.clone());
        let ws_bob = workspace_with_hdc("hdc_bob", db.clone());

        let doc_a = ws_alice
            .write("secret.md", "alice's secret")
            .await
            .expect("alice write failed");
        let doc_b = ws_bob
            .write("secret.md", "bob's secret")
            .await
            .expect("bob write failed");

        // Alice's list should only include her documents.
        let alice_fps = db
            .list_document_hdc_fingerprints("hdc_alice", None)
            .await
            .expect("list alice failed");
        assert_eq!(alice_fps.len(), 1);
        assert_eq!(alice_fps[0].id, doc_a.id);
        assert_eq!(alice_fps[0].fingerprint.len(), 1280);

        // Bob's list should only include his documents.
        let bob_fps = db
            .list_document_hdc_fingerprints("hdc_bob", None)
            .await
            .expect("list bob failed");
        assert_eq!(bob_fps.len(), 1);
        assert_eq!(bob_fps[0].id, doc_b.id);
        assert_eq!(bob_fps[0].fingerprint.len(), 1280);
        assert_ne!(alice_fps[0].fingerprint, bob_fps[0].fingerprint);
    }

    #[tokio::test]
    async fn fingerprint_shadow_disabled_stores_no_fingerprint() {
        let (db, _dir) = setup().await;
        let ws = Workspace::new_with_db("hdc_shadow_disabled", db.clone());

        // Shadow mode is opt-in; a normal write with the feature compiled in
        // still stores no fingerprint when the workspace flag is disabled.
        let doc = ws
            .write("normal.md", "content that should persist")
            .await
            .expect("write must succeed regardless of HDC state");

        // Verify the document is readable.
        let read_back = ws.read("normal.md").await.expect("read failed");
        assert_eq!(read_back.content, "content that should persist");
        assert_eq!(read_back.id, doc.id);

        // No fingerprint was stored — list is empty.
        let fps = db
            .list_document_hdc_fingerprints("hdc_shadow_disabled", None)
            .await
            .expect("list failed");
        assert!(fps.is_empty());
    }
}

// ─── Caller-level tests ──────────────────────────────────────────────────────
//
// Verify that the memory_write tool produces correct output with HDC enabled.
// Phase 2 is shadow-mode: no new fields exposed in tool output.

#[cfg(all(feature = "hdc", feature = "libsql", feature = "integration"))]
mod hdc_caller {
    use std::sync::Arc;

    use ironclaw::context::JobContext;
    use ironclaw::db::Database;
    #[allow(unused_imports)] // trait in scope for method dispatch on Database
    use ironclaw::db::WorkspaceStore;
    use ironclaw::db::libsql::LibSqlBackend;
    use ironclaw::tools::Tool;
    use ironclaw::tools::builtin::memory::MemoryWriteTool;
    use ironclaw::workspace::Workspace;

    async fn setup_tool(
        user_id: &str,
    ) -> (
        MemoryWriteTool,
        JobContext,
        Arc<dyn Database>,
        tempfile::TempDir,
    ) {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db_path = dir.path().join("hdc_caller_test.db");
        let backend = LibSqlBackend::new_local(&db_path).await.expect("create db");
        backend.run_migrations().await.expect("run migrations");
        let db: Arc<dyn Database> = Arc::new(backend);
        let ws =
            Arc::new(Workspace::new_with_db(user_id, db.clone()).with_hdc_fingerprint_shadow(true));
        let tool = MemoryWriteTool::from_workspace(ws);
        let ctx = JobContext::with_user(user_id, "hdc caller test", "workspace hdc caller test");
        (tool, ctx, db, dir)
    }

    #[tokio::test]
    async fn memory_write_tool_succeeds_with_hdc() {
        // Exercise the real caller: MemoryWriteTool.execute() should succeed
        // while shadow-mode HDC persists derived fingerprint metadata.
        let (tool, ctx, db, _dir) = setup_tool("caller_write").await;

        let output = tool
            .execute(
                serde_json::json!({
                    "target": "notes/idea.md",
                    "content": "A great idea for the product",
                    "append": false
                }),
                &ctx,
            )
            .await
            .expect("memory_write must succeed with hdc feature enabled");

        assert_eq!(output.result["status"], "written");
        assert_eq!(output.result["path"], "notes/idea.md");
        assert_eq!(output.result["append"], false);

        let fingerprints = db
            .list_document_hdc_fingerprints("caller_write", None)
            .await
            .expect("list fingerprints");
        assert_eq!(fingerprints.len(), 1);
        assert_eq!(fingerprints[0].path, "notes/idea.md");
        assert_eq!(fingerprints[0].fingerprint.len(), 1280);
    }

    #[tokio::test]
    async fn memory_write_tool_output_unchanged() {
        // Phase 2 is shadow-mode: the tool output shape stays baseline even
        // when the write stored a fingerprint internally.
        let (tool, ctx, db, _dir) = setup_tool("caller_output").await;

        let output = tool
            .execute(
                serde_json::json!({
                    "target": "test.md",
                    "content": "content",
                    "append": false
                }),
                &ctx,
            )
            .await
            .expect("memory_write failed");

        let fingerprints = db
            .list_document_hdc_fingerprints("caller_output", None)
            .await
            .expect("list fingerprints");
        assert_eq!(fingerprints.len(), 1);

        let json = serde_json::to_string(&output.result).expect("serialize output");
        assert!(
            !json.contains("hdc_fingerprint"),
            "Phase 2 shadow mode: hdc_fingerprint must not appear in serialized output.\n\
             Got: {json}"
        );
        assert!(
            !json.contains("\"dedup\""),
            "Default-off dedup should not add a dedup field in Phase 2 output: {json}"
        );
    }
}

// ─── PostgreSQL parity tests ──────────────────────────────────────────────────
//
// Verify that the PostgreSQL backend implements HDC fingerprint storage with
// the same contract as the libSQL backend.
//
// Run with:
//   DATABASE_URL=postgres://localhost/ironclaw_test \
//   cargo test --features hdc,integration -- pg_hdc
//
// Requirements:
//   - PostgreSQL 15+ with pgvector extension installed
//   - V33__memory_hdc_fingerprint.sql migration applied
//     (ironclaw onboard, or: psql -f migrations/V33__memory_hdc_fingerprint.sql)
//   - DATABASE_URL env var pointing at a test database (defaults to
//     postgres://localhost/ironclaw_test)
//
// CI: these tests are marked #[ignore] because no PostgreSQL instance is
// available in the default Rust matrix. They run in the dedicated integration
// CI job that provisions a postgres service container.

#[cfg(all(feature = "hdc", feature = "integration"))]
mod pg_hdc {
    use std::sync::Arc;

    use ironclaw::db::Database;
    #[allow(unused_imports)] // trait in scope for WorkspaceStore method dispatch
    use ironclaw::db::WorkspaceStore;
    use ironclaw::workspace::Workspace;
    use uuid::Uuid;

    // ── Database setup helpers ────────────────────────────────────────────

    /// Build a deadpool-postgres pool from DATABASE_URL (or the default).
    ///
    /// Returns `None` when the env var is absent or the server is unreachable,
    /// which causes the individual tests to skip rather than fail.
    fn pg_pool() -> Option<deadpool_postgres::Pool> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/ironclaw_test".to_string());
        let config: tokio_postgres::Config = match database_url.parse() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("pg_hdc: invalid DATABASE_URL ({e}), skipping");
                return None;
            }
        };
        let mgr = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
        deadpool_postgres::Pool::builder(mgr)
            .max_size(4)
            .build()
            .ok()
    }

    /// Attempt a connection; return `None` so the test can skip gracefully
    /// when Postgres is not available in the local environment.
    async fn try_connect(pool: &deadpool_postgres::Pool) -> Option<()> {
        match pool.get().await {
            Ok(_) => Some(()),
            Err(e) => {
                eprintln!("pg_hdc: database unavailable ({e}), skipping");
                None
            }
        }
    }

    /// Remove all test documents for a user so tests are idempotent.
    async fn cleanup_user(pool: &deadpool_postgres::Pool, user_id: &str) {
        let conn = pool.get().await.expect("cleanup: get conn");
        conn.execute(
            "DELETE FROM memory_documents WHERE user_id = $1",
            &[&user_id],
        )
        .await
        .ok();
    }

    /// Build a `PgBackend` wrapped as `Arc<dyn Database>` from the pool.
    ///
    /// The `Repository` inside `PgBackend` already implements all `WorkspaceStore`
    /// HDC methods — this exercises exactly the same code path as production.
    async fn pg_backend(pool: deadpool_postgres::Pool) -> Arc<dyn Database> {
        use ironclaw::db::postgres::PgBackend;
        // PgBackend::from_pool is the unit-test-friendly constructor that accepts
        // a pre-built pool (skips env-var parsing and re-validation).
        Arc::new(PgBackend::from_pool(pool))
    }

    // ── Storage contract tests ────────────────────────────────────────────

    /// Verify that a document written via the Workspace has NULL hdc_fingerprint
    /// by default (no shadow mode), matching the migration contract.
    #[tokio::test]
    #[ignore = "requires PostgreSQL with V33 migration applied (see module doc)"]
    async fn pg_hdc_null_by_default() {
        let Some(pool) = pg_pool() else { return };
        if try_connect(&pool).await.is_none() {
            return;
        }
        let user_id = "pg_hdc_null_default";
        cleanup_user(&pool, user_id).await;

        let db = pg_backend(pool.clone()).await;
        let ws = Workspace::new_with_db(user_id, db.clone());
        let doc = ws.write("test.md", "hello").await.expect("write failed");

        // Fingerprint should be NULL until explicitly stored.
        assert!(
            doc.hdc_fingerprint.is_none(),
            "pg: fresh document should have NULL hdc_fingerprint"
        );

        let fps = db
            .list_document_hdc_fingerprints(user_id, None)
            .await
            .expect("list failed");
        assert!(fps.is_empty(), "pg: no fingerprints before any store call");

        cleanup_user(&pool, user_id).await;
    }

    /// Store a 1280-byte fingerprint and read it back — full round-trip.
    #[tokio::test]
    #[ignore = "requires PostgreSQL with V33 migration applied (see module doc)"]
    async fn pg_hdc_fingerprint_roundtrip() {
        let Some(pool) = pg_pool() else { return };
        if try_connect(&pool).await.is_none() {
            return;
        }
        let user_id = "pg_hdc_roundtrip";
        cleanup_user(&pool, user_id).await;

        let db = pg_backend(pool.clone()).await;
        let ws = Workspace::new_with_db(user_id, db.clone());
        let doc = ws
            .write("roundtrip.md", "content")
            .await
            .expect("write failed");

        // Store a varied 1280-byte fingerprint.
        let fingerprint: Vec<u8> = (0..1280).map(|i| (i % 256) as u8).collect();
        db.update_document_hdc_fingerprint(doc.id, &fingerprint)
            .await
            .expect("pg: store fingerprint failed");

        // Read back and verify bytes.
        let fps = db
            .list_document_hdc_fingerprints(user_id, None)
            .await
            .expect("pg: list failed");
        assert_eq!(fps.len(), 1, "pg: exactly one fingerprint");
        assert_eq!(fps[0].id, doc.id);
        assert_eq!(fps[0].path, "roundtrip.md");
        assert_eq!(fps[0].fingerprint, fingerprint);

        cleanup_user(&pool, user_id).await;
    }

    /// Wrong-length fingerprints must be rejected at the trait level.
    #[tokio::test]
    #[ignore = "requires PostgreSQL with V33 migration applied (see module doc)"]
    async fn pg_hdc_rejects_wrong_length() {
        let Some(pool) = pg_pool() else { return };
        if try_connect(&pool).await.is_none() {
            return;
        }
        let user_id = "pg_hdc_wrong_len";
        cleanup_user(&pool, user_id).await;

        let db = pg_backend(pool.clone()).await;
        let ws = Workspace::new_with_db(user_id, db.clone());
        let doc = ws.write("test.md", "content").await.expect("write failed");

        // 500-byte fingerprint — must be rejected.
        let bad = vec![0u8; 500];
        let result = db.update_document_hdc_fingerprint(doc.id, &bad).await;
        assert!(
            result.is_err(),
            "pg: 500-byte fingerprint should be rejected"
        );
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("1280"),
            "pg: error must mention expected length 1280, got: {msg}"
        );

        cleanup_user(&pool, user_id).await;
    }

    /// list_document_hdc_fingerprints excludes documents with NULL fingerprint.
    #[tokio::test]
    #[ignore = "requires PostgreSQL with V33 migration applied (see module doc)"]
    async fn pg_hdc_list_excludes_null() {
        let Some(pool) = pg_pool() else { return };
        if try_connect(&pool).await.is_none() {
            return;
        }
        let user_id = "pg_hdc_excludes_null";
        cleanup_user(&pool, user_id).await;

        let db = pg_backend(pool.clone()).await;
        let ws = Workspace::new_with_db(user_id, db.clone());

        let doc_with = ws.write("with-fp.md", "a").await.expect("write 1");
        let _doc_without = ws.write("without-fp.md", "b").await.expect("write 2");

        let fp = vec![0xAAu8; 1280];
        db.update_document_hdc_fingerprint(doc_with.id, &fp)
            .await
            .expect("store fp");

        let fps = db
            .list_document_hdc_fingerprints(user_id, None)
            .await
            .expect("list");
        assert_eq!(fps.len(), 1, "pg: only the document with a fingerprint");
        assert_eq!(fps[0].id, doc_with.id);

        cleanup_user(&pool, user_id).await;
    }

    /// list_document_hdc_fingerprints is user-scoped; other users' data must
    /// not bleed through.
    #[tokio::test]
    #[ignore = "requires PostgreSQL with V33 migration applied (see module doc)"]
    async fn pg_hdc_user_scope_isolation() {
        let Some(pool) = pg_pool() else { return };
        if try_connect(&pool).await.is_none() {
            return;
        }
        let user_a = "pg_hdc_scope_a";
        let user_b = "pg_hdc_scope_b";
        cleanup_user(&pool, user_a).await;
        cleanup_user(&pool, user_b).await;

        let db = pg_backend(pool.clone()).await;

        let ws_a = Workspace::new_with_db(user_a, db.clone());
        let ws_b = Workspace::new_with_db(user_b, db.clone());

        let doc_a = ws_a.write("secret.md", "alice").await.expect("write a");
        let doc_b = ws_b.write("secret.md", "bob").await.expect("write b");

        let fp_a = vec![0x11u8; 1280];
        let fp_b = vec![0x22u8; 1280];
        db.update_document_hdc_fingerprint(doc_a.id, &fp_a)
            .await
            .expect("store fp_a");
        db.update_document_hdc_fingerprint(doc_b.id, &fp_b)
            .await
            .expect("store fp_b");

        // Each user sees only their own fingerprint.
        let fps_a = db
            .list_document_hdc_fingerprints(user_a, None)
            .await
            .expect("list a");
        assert_eq!(fps_a.len(), 1);
        assert_eq!(fps_a[0].id, doc_a.id);
        assert_eq!(fps_a[0].fingerprint, fp_a);

        let fps_b = db
            .list_document_hdc_fingerprints(user_b, None)
            .await
            .expect("list b");
        assert_eq!(fps_b.len(), 1);
        assert_eq!(fps_b[0].id, doc_b.id);
        assert_eq!(fps_b[0].fingerprint, fp_b);

        cleanup_user(&pool, user_a).await;
        cleanup_user(&pool, user_b).await;
    }

    /// Agent-scoped documents follow nullable-agent semantics: `agent_id = None`
    /// in list_document_hdc_fingerprints returns only unscoped documents.
    #[tokio::test]
    #[ignore = "requires PostgreSQL with V33 migration applied (see module doc)"]
    async fn pg_hdc_agent_scope() {
        let Some(pool) = pg_pool() else { return };
        if try_connect(&pool).await.is_none() {
            return;
        }
        let user_id = "pg_hdc_agent_scope";
        cleanup_user(&pool, user_id).await;

        let db = pg_backend(pool.clone()).await;
        let agent_id = Uuid::new_v4();

        let ws_unscoped = Workspace::new_with_db(user_id, db.clone());
        let ws_scoped = Workspace::new_with_db(user_id, db.clone()).with_agent(agent_id);

        let doc_u = ws_unscoped
            .write("unscoped.md", "no agent")
            .await
            .expect("write unscoped");
        let doc_s = ws_scoped
            .write("scoped.md", "with agent")
            .await
            .expect("write scoped");

        let fp = vec![0x55u8; 1280];
        db.update_document_hdc_fingerprint(doc_u.id, &fp)
            .await
            .expect("store fp unscoped");
        db.update_document_hdc_fingerprint(doc_s.id, &fp)
            .await
            .expect("store fp scoped");

        // Unscoped list: only the unscoped document.
        let fps_none = db
            .list_document_hdc_fingerprints(user_id, None)
            .await
            .expect("list None");
        assert_eq!(fps_none.len(), 1);
        assert_eq!(fps_none[0].id, doc_u.id);

        // Agent-scoped list: only the agent document.
        let fps_agent = db
            .list_document_hdc_fingerprints(user_id, Some(agent_id))
            .await
            .expect("list agent");
        assert_eq!(fps_agent.len(), 1);
        assert_eq!(fps_agent[0].id, doc_s.id);

        cleanup_user(&pool, user_id).await;
    }

    /// Overwriting a fingerprint stores the new bytes (no duplicate rows).
    #[tokio::test]
    #[ignore = "requires PostgreSQL with V33 migration applied (see module doc)"]
    async fn pg_hdc_overwrite_fingerprint() {
        let Some(pool) = pg_pool() else { return };
        if try_connect(&pool).await.is_none() {
            return;
        }
        let user_id = "pg_hdc_overwrite";
        cleanup_user(&pool, user_id).await;

        let db = pg_backend(pool.clone()).await;
        let ws = Workspace::new_with_db(user_id, db.clone());
        let doc = ws.write("doc.md", "text").await.expect("write failed");

        let fp_old = vec![0x01u8; 1280];
        db.update_document_hdc_fingerprint(doc.id, &fp_old)
            .await
            .expect("store old");

        let fp_new = vec![0xFFu8; 1280];
        db.update_document_hdc_fingerprint(doc.id, &fp_new)
            .await
            .expect("store new");

        let fps = db
            .list_document_hdc_fingerprints(user_id, None)
            .await
            .expect("list");
        assert_eq!(
            fps.len(),
            1,
            "pg: still exactly one fingerprint after overwrite"
        );
        assert_eq!(fps[0].fingerprint, fp_new, "pg: new bytes stored");

        cleanup_user(&pool, user_id).await;
    }

    /// Shadow-mode writes via Workspace::with_hdc_fingerprint_shadow compute
    /// and store a fingerprint automatically via the PG backend.
    #[tokio::test]
    #[ignore = "requires PostgreSQL with V33 migration applied (see module doc)"]
    async fn pg_hdc_shadow_mode_stores_fingerprint() {
        let Some(pool) = pg_pool() else { return };
        if try_connect(&pool).await.is_none() {
            return;
        }
        let user_id = "pg_hdc_shadow";
        cleanup_user(&pool, user_id).await;

        let db = pg_backend(pool.clone()).await;
        let ws = Workspace::new_with_db(user_id, db.clone()).with_hdc_fingerprint_shadow(true);

        let doc = ws
            .write("shadow.md", "shadow mode content")
            .await
            .expect("write failed");

        let fps = db
            .list_document_hdc_fingerprints(user_id, None)
            .await
            .expect("list");
        assert_eq!(fps.len(), 1, "pg shadow: one fingerprint after write");
        assert_eq!(fps[0].id, doc.id);
        assert_eq!(
            fps[0].fingerprint.len(),
            1280,
            "pg shadow: full 1280-byte vector"
        );

        cleanup_user(&pool, user_id).await;
    }

    /// check_dedup via PG: two docs with identical content share a fingerprint
    /// close enough for near-duplicate detection (Hamming distance ~0).
    #[tokio::test]
    #[ignore = "requires PostgreSQL with V33 migration applied (see module doc)"]
    async fn pg_hdc_check_dedup_identical_content() {
        let Some(pool) = pg_pool() else { return };
        if try_connect(&pool).await.is_none() {
            return;
        }
        let user_id = "pg_hdc_dedup";
        cleanup_user(&pool, user_id).await;

        let db = pg_backend(pool.clone()).await;
        let ws = Workspace::new_with_db(user_id, db.clone()).with_hdc_fingerprint_shadow(true);

        // Write the same content to two different paths.
        ws.write("dup-a.md", "identical content here")
            .await
            .expect("write a");
        ws.write("dup-b.md", "identical content here")
            .await
            .expect("write b");

        let fps = db
            .list_document_hdc_fingerprints(user_id, None)
            .await
            .expect("list");
        assert_eq!(fps.len(), 2, "pg dedup: both documents have fingerprints");

        // Fingerprints for identical content should be equal (deterministic encoder).
        assert_eq!(
            fps[0].fingerprint, fps[1].fingerprint,
            "pg dedup: identical content yields identical fingerprints"
        );

        cleanup_user(&pool, user_id).await;
    }
}
