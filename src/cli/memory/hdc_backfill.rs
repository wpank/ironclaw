//! HDC fingerprint backfill command.
//!
//! Computes HDC fingerprints for existing workspace documents that don't
//! already have one, writing them back to the database.

use std::sync::Arc;

use clap::Args;

use crate::db::Database;

/// Arguments for `ironclaw memory hdc backfill`.
#[derive(Args, Debug, Clone)]
pub struct HdcBackfillArgs {
    /// Only backfill documents for this user ID
    #[arg(long)]
    user: Option<String>,

    /// Maximum number of documents to fingerprint
    #[arg(long)]
    limit: Option<usize>,

    /// Compute fingerprints but don't write to database
    #[arg(long)]
    dry_run: bool,

    /// Batch size for processing
    #[arg(long, default_value = "100")]
    batch_size: usize,
}

/// Run the HDC backfill command.
pub async fn run_hdc_backfill(args: HdcBackfillArgs, db: Arc<dyn Database>) -> anyhow::Result<()> {
    use ironclaw_hdc::codebook::Codebook;
    use ironclaw_hdc::{DocumentEncodingInput, encode_document};

    let mut codebook = Codebook::new();
    let mut scanned = 0u64;
    let mut skipped = 0u64;
    let mut fingerprinted = 0u64;
    let mut failed = 0u64;

    // Determine user scope. If not specified, use the configured owner_id.
    let user_id = match &args.user {
        Some(u) => u.clone(),
        None => {
            let config = crate::config::Config::from_env()
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            config.owner_id.clone()
        }
    };

    // List all documents for this user
    let documents = db.list_documents(&user_id, None).await?;
    let total = documents.len();

    println!("HDC Backfill: scanning {total} document(s) for user \"{user_id}\"");
    if args.dry_run {
        println!("  (dry-run mode -- no changes will be written)");
    }
    println!();

    for doc in documents {
        if let Some(limit) = args.limit {
            if fingerprinted >= limit as u64 {
                break;
            }
        }
        scanned += 1;

        // Skip if already has fingerprint
        if doc.hdc_fingerprint.is_some() {
            skipped += 1;
            continue;
        }

        // Extract tags from metadata if present
        let tags_vec: Vec<String> = doc
            .metadata
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let tags_refs: Vec<&str> = tags_vec.iter().map(|s| s.as_str()).collect();

        // Compute fingerprint
        let fingerprint = encode_document(
            DocumentEncodingInput {
                content: &doc.content,
                tags: &tags_refs,
                path: &doc.path,
            },
            &mut codebook,
        );

        if !args.dry_run {
            match db
                .update_document_hdc_fingerprint(doc.id, &fingerprint.to_bytes())
                .await
            {
                Ok(()) => fingerprinted += 1,
                Err(e) => {
                    eprintln!("  Failed to fingerprint \"{}\": {e}", doc.path);
                    failed += 1;
                }
            }
        } else {
            fingerprinted += 1;
        }

        // Progress indicator for large runs
        if scanned % args.batch_size as u64 == 0 {
            eprint!("\r  Progress: {scanned}/{total}");
        }
    }

    // Clear progress line
    if scanned >= args.batch_size as u64 {
        eprint!("\r");
    }

    println!("HDC Backfill Results:");
    println!("  Scanned:       {scanned}");
    println!("  Skipped:       {skipped} (already has fingerprint)");
    println!("  Fingerprinted: {fingerprinted}");
    println!("  Failed:        {failed}");
    if args.dry_run {
        println!("  (dry-run mode -- no changes written)");
    }

    Ok(())
}
