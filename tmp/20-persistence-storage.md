# Roko Persistence and Storage Layer

## Document Purpose

This document provides a comprehensive technical analysis of roko's `roko-fs` crate -- the filesystem-backed persistence and storage layer that underpins all durable state in the roko AI agent system. It is written for someone seeing this codebase for the first time, with zero prior context. Every claim is backed by exact file paths and real code from the roko repository at `/Users/will/dev/nunchi/roko/roko/`.

---

## 1. Storage Philosophy: Why Append-Only JSONL for an AI Agent

### 1.1 The Case Against Mutable Storage

Traditional databases (PostgreSQL, SQLite, MongoDB) assume data is mutable -- rows are updated, deleted, replaced. For an AI agent system, this assumption is wrong. An AI agent's most valuable asset is its full history: every decision, every tool call, every observation, every intermediate reasoning step. Mutating or deleting that history destroys the audit trail that makes the agent debuggable, explainable, and trustworthy.

Roko makes a different choice. Its persistence layer is built on **append-only JSONL** (JSON Lines) files -- one JSON object per line, appended sequentially, never overwritten. This design draws on a long tradition in systems engineering:

- **Log-structured file systems** (Rosenblum and Ousterhout, 1992) demonstrated that sequential, append-only writes yield higher throughput than random in-place updates, because sequential I/O saturates disk bandwidth while random I/O is seek-bound [1].
- **Write-ahead logging** (WAL), formalized in the ARIES algorithm (Mohan et al., 1992), showed that durability and crash recovery can be achieved by writing intentions to a sequential log before applying them [2, 3].
- **Event sourcing** (Fowler, 2005) established the pattern of storing state as an immutable append-only sequence of domain events, with the current state reconstructed by replaying the log [4, 5].

Roko's JSONL approach is a simplification of all three: the log *is* the data, the log *is* the journal, and the log *is* the event store. There is no separate database, no WAL, and no event/state duality.

### 1.2 Properties of Append-Only JSONL

**Crash safety by construction.** An append-only write either completes (the line is there) or it does not (the last line is partial). There is no intermediate state where half of a record is overwritten. On restart, the system replays the log and skips any partial trailing line. No WAL, no transaction journal, no recovery procedure -- the log *is* the journal [6].

**Human readability.** A JSONL file can be inspected with `cat`, filtered with `grep`, parsed with `jq`, diffed with `diff`. When debugging an agent that made a bad decision at 3 AM, being able to `grep` through its memory log is invaluable. This is a key differentiator from binary formats like LevelDB's LST or SQLite's B-tree pages.

**Immutability as an audit guarantee.** Because records are never overwritten, the on-disk file is an authoritative, tamper-evident history. Lineage DAGs (which roko uses extensively) can always be traversed back to their roots. This property is directly analogous to content-addressable storage (CAS) systems, where the address of data is derived from its content hash, making deduplication automatic and integrity verification trivial [7].

**Trivial replication.** Appending to a file is the simplest possible write pattern. Streaming the file to a remote store, backing it up, or tailing it for live observability requires no special tooling.

The roko crate-level documentation states this rationale explicitly:

```rust
// Source: crates/roko-fs/src/lib.rs, lines 1-19

//! Filesystem-backed [`Store`](roko_core::Store).
//!
//! `FileSubstrate` persists signals to an append-only JSONL log under a
//! directory (typically `.roko/signals/`). It keeps an in-memory index of
//! all live signals for fast querying, and rebuilds that index from the log
//! on startup.
//!
//! # Why JSONL + in-memory?
//!
//! - **Append-only** writes are crash-safe: if the process dies mid-write,
//!   worst case is a partial last line that we skip on replay.
//! - **JSONL** is human-readable, grep-able, diff-able -- helpful for debugging
//!   and for users inspecting their `.roko/` directory.
//! - **In-memory index** gives us the same query latency as `MemorySubstrate`.
//!   Memory cost is low: tens of MB per million signals.
//!
//! When workload grows beyond in-memory capacity, we can swap in a different
//! backend (`SQLite`, `sled`) behind the same `Store` trait -- the callers
//! won't change.
```

---

## 2. The Store Trait -- The Kernel Storage Contract

Everything in roko flows through the `Store` trait. This trait defines the contract that all storage backends must honor. The full definition (from `crates/roko-core/src/traits.rs`, lines 37-80):

```rust
#[async_trait]
pub trait Store: Send + Sync {
    /// Store an engram. Returns its content hash. Idempotent on content.
    async fn put(&self, engram: Engram) -> Result<ContentHash>;

    /// Retrieve an engram by content hash. Does not apply decay.
    async fn get(&self, id: &ContentHash) -> Result<Option<Engram>>;

    /// Query for engrams matching the given filter. Impls may apply decay
    /// when evaluating `min_weight` and when ordering results.
    async fn query(&self, q: &Query, ctx: &Context) -> Result<Vec<Engram>>;

    /// Query by HDC similarity against a fingerprint, returning ranked matches.
    async fn query_similar(
        &self,
        _fp: &HdcVector,
        _radius: f32,
        _limit: usize,
        _ctx: &Context,
    ) -> Result<Vec<(ContentHash, f32)>> {
        Ok(Vec::new())
    }

    /// Remove engrams whose effective weight (score x decay) has fallen
    /// below `threshold` at `ctx.now_ms`. Returns count of pruned engrams.
    async fn prune(&self, threshold: f32, ctx: &Context) -> Result<usize>;

    /// Optional: total count of stored engrams (for metrics/health checks).
    async fn len(&self) -> Result<usize> { Ok(0) }

    /// Optional: is the substrate empty?
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.len().await? == 0)
    }

    /// Human-readable name for logging/debugging.
    fn name(&self) -> &'static str { "unnamed_store" }
}
```

Key design points:

- **Content-addressed identity**: `put()` returns a `ContentHash` (BLAKE3 digest). Two identical engrams produce the same hash. This makes `put()` idempotent -- re-storing the same data is a no-op. This is the same principle behind Git's object store and IPFS's content identifiers [7].
- **Decay-aware pruning**: `prune()` removes engrams whose effective weight has decayed below a threshold. This is not deletion in the traditional sense -- it is a lifecycle operation where time-decayed data is removed from the hot path.
- **HDC similarity search**: `query_similar()` enables hyperdimensional computing (HDC) based similarity queries, with a default no-op implementation for backends that do not index HDC vectors.
- **Send + Sync**: All stores must be safe for concurrent access from multiple async tasks.

---

## 3. The Engram -- What Gets Stored

The universal datum in roko is the **Engram** -- a content-addressed, scored, decaying, lineage-tracked record. Every event, tool call, agent output, gate verdict, episode, and knowledge entry is an Engram. The struct definition (from `crates/roko-core/src/engram.rs`, lines 62-98):

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Engram {
    /// Content-addressed identity (computed from kind + body + author + tags).
    pub id: ContentHash,
    /// HDC fingerprint plus encoder metadata used for similarity and clustering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<HdcFingerprint>,
    /// What kind of engram this is.
    pub kind: Kind,
    /// The engram's payload.
    pub body: Body,
    /// Unix milliseconds when this engram was first emitted.
    pub created_at_ms: i64,
    /// How this engram's weight decays over time.
    pub decay: Decay,
    /// Producer attribution and trust.
    pub provenance: Provenance,
    /// Quality score at emission time (may be recomputed by scorers).
    pub score: Score,
    /// ContentHashes of engrams this derived from (forms a DAG for auditing).
    pub lineage: Vec<ContentHash>,
    /// Arbitrary string metadata (ordered for stable hashing).
    pub tags: BTreeMap<String, String>,
    /// Optional cryptographic proof of origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attestation: Option<Attestation>,
    /// Optional emotional metadata associated with this engram.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emotional_tag: Option<EmotionalTag>,
    /// Demurrage balance in [0.0, 1.0]. Decays over time; refreshed on access.
    #[serde(default = "default_balance")]
    pub balance: f64,
}
```

The content hash is computed over identity fields only (kind, body, author, taint, lineage, tags), excluding mutable fields (score, decay, creation time, fingerprint, attestation, emotional_tag, balance). This means `Store.put()` is truly idempotent: re-putting the same content produces the same hash and is a no-op.

Each engram carries a `Decay` variant that determines how its effective weight changes over time:

| Decay Variant | Behavior |
|---|---|
| `None` | Never decays. For identity records, configuration, schemas. |
| `HalfLife { half_life_ms }` | Exponential decay with configurable half-life. For traces, short-lived signals. |
| `Ttl { ttl_ms }` | Hard time-to-live. Weight drops to zero after the TTL. |
| `Ebbinghaus` | Forgetting curve model from cognitive psychology. For recall-dependent knowledge. |

The effective weight at any point in time is:

```rust
pub fn weight_at(&self, now_ms: i64) -> f32 {
    let age = now_ms - self.created_at_ms;
    self.score.effective() * self.decay.apply(age)
}
```

The `Ebbinghaus` variant is inspired by Hermann Ebbinghaus's 1885 forgetting curve research, which showed that memory retention decays exponentially with time but can be strengthened through spaced repetition [8].

---

## 4. FileSubstrate -- The Crash-Safe JSONL Storage Engine

`FileSubstrate` is the concrete `Store` implementation that lives in `roko-fs`. It is the default persistence backend for all roko agents.

### 4.1 Structure and Initialization

From `crates/roko-fs/src/file_substrate.rs`, lines 23-33:

```rust
pub struct FileSubstrate {
    /// Directory containing `engrams.jsonl`.
    root: PathBuf,
    /// In-memory index: `ContentHash` -> `Engram`.
    index: RwLock<HashMap<ContentHash, Engram>>,
    /// Serializes writes to the log file.
    log_writer: Mutex<File>,
    /// Human-readable name (kept for Debug / logging).
    #[allow(dead_code)]
    name: String,
}
```

The design is a dual-layer architecture:

1. **On-disk**: An append-only `engrams.jsonl` file that is the source of truth.
2. **In-memory**: A `HashMap<ContentHash, Engram>` index that mirrors the log for fast reads.

The concurrency model uses two different lock types for reads and writes:

- **Reads** go through `parking_lot::RwLock` (synchronous, non-blocking for concurrent readers).
- **Writes** go through `tokio::sync::Mutex` (async, serializes all writes to the log file).

This separation is intentional: reads never block on writes, and writes never interleave at the byte level. The pattern mirrors the in-memory index + on-disk log design used by LSM-tree based systems like LevelDB and RocksDB, but with the radical simplification that the "on-disk" component is a flat JSONL file rather than a sorted string table (SSTable) hierarchy [9].

### 4.2 Opening and Replay

When `FileSubstrate::open()` is called, it replays the entire JSONL log into the in-memory index. From `crates/roko-fs/src/file_substrate.rs`, lines 45-67:

```rust
pub async fn open(root: impl Into<PathBuf>) -> Result<Self> {
    let root = root.into();
    fs::create_dir_all(&root).await?;
    let log_path = root.join("engrams.jsonl");

    // Replay: read any existing entries into the in-memory index.
    let index = replay_log(&log_path).await?;

    // Open for append -- all subsequent writes go to the end.
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .await?;

    let name = format!("file:{}", root.display());
    Ok(Self {
        root,
        index: RwLock::new(index),
        log_writer: Mutex::new(file),
        name,
    })
}
```

The replay function is the crash-recovery mechanism. From `crates/roko-fs/src/file_substrate.rs`, lines 181-208:

```rust
async fn replay_log(log_path: &Path) -> Result<HashMap<ContentHash, Engram>> {
    let mut index = HashMap::new();
    if !log_path.exists() {
        return Ok(index);
    }
    let file = File::open(log_path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut line_no = 0usize;
    while let Some(line) = lines.next_line().await? {
        line_no += 1;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Engram>(&line) {
            Ok(sig) => {
                index.insert(sig.id, sig);
            }
            Err(e) => {
                // Last line may be a partial write after a crash -- skip and
                // continue. For other malformed lines, log and keep going;
                // we prefer partial availability over total failure.
                tracing_line_error(log_path, line_no, &e);
            }
        }
    }
    Ok(index)
}
```

Critical crash-safety properties:

1. **Partial last line is silently skipped.** If the process died mid-write, the last line will fail JSON parsing and is ignored.
2. **Earlier valid lines are always preserved.** An append-only write to the middle of the file is impossible -- only the tail can be corrupted.
3. **No separate WAL or journal is needed.** The JSONL file *is* the journal. Replay is the recovery procedure.
4. **Malformed lines anywhere in the file are skipped.** The system prefers partial availability over total failure -- the same philosophy as Cassandra's "anti-entropy" approach, where partial data is better than no data [10].

### 4.3 The Write Protocol

Single-record writes follow a strict sequence. From `crates/roko-fs/src/file_substrate.rs`, lines 270-292:

```rust
#[async_trait]
impl Store for FileSubstrate {
    async fn put(&self, signal: Engram) -> Result<ContentHash> {
        // Dedupe: skip write if already present.
        if self.index.read().contains_key(&signal.id) {
            return Ok(signal.id);
        }
        let signal = attach_hdc_fingerprint(signal);
        let id = signal.id;
        // Serialize and append.
        let line = serde_json::to_string(&signal).map_err(RokoError::body_encode)?;
        let mut guard = self.log_writer.lock().await;
        guard.write_all(line.as_bytes()).await?;
        guard.write_all(b"\n").await?;
        guard.flush().await?;
        drop(guard);
        // Update index.
        self.index.write().insert(id, signal);
        Ok(id)
    }
```

The write sequence is:

1. **Deduplication check** (read lock on index) -- skip if content hash already exists.
2. **HDC fingerprint attachment** -- optionally compute a hyperdimensional fingerprint for similarity search.
3. **Serialize to JSON** -- produces a single line of text.
4. **Acquire write lock** -- `tokio::sync::Mutex` serializes all concurrent writes.
5. **Append line + newline** -- two separate `write_all` calls.
6. **Flush** -- ensures the data reaches the OS buffer (but does not `fsync` to disk).
7. **Drop write lock** -- releases for other writers.
8. **Update in-memory index** -- only after the disk write succeeds.

The ordering of steps 7 and 8 is important: the in-memory index is updated *after* the disk write succeeds. If the write fails, the index remains consistent with what is on disk. Note that `flush()` without `fsync()` pushes data to the kernel page cache but does not guarantee it reaches stable storage. This is a deliberate throughput/durability tradeoff -- a power failure could lose the last few unfsynced writes, but a process crash will not (since the kernel's page cache survives process death).

### 4.4 Batch Writes

For bulk ingestion, `put_batch()` is significantly more efficient. From `crates/roko-fs/src/file_substrate.rs`, lines 137-178:

```rust
pub async fn put_batch(&self, signals: Vec<Engram>) -> Result<Vec<ContentHash>> {
    let mut ids = Vec::with_capacity(signals.len());
    let mut lines = String::new();
    let mut to_index: Vec<Engram> = Vec::new();
    let mut seen_ids = HashSet::with_capacity(signals.len());

    // Phase 1: deduplicate and serialize without holding the write lock.
    {
        let index_read = self.index.read();
        for signal in signals {
            let signal = attach_hdc_fingerprint(signal);
            let id = signal.id;
            if index_read.contains_key(&id) || !seen_ids.insert(id) {
                ids.push(id);
                continue;
            }
            let line = serde_json::to_string(&signal).map_err(RokoError::body_encode)?;
            lines.push_str(&line);
            lines.push('\n');
            to_index.push(signal);
            ids.push(id);
        }
    }

    // Phase 2: single lock acquire -> write -> flush.
    if !lines.is_empty() {
        let mut guard = self.log_writer.lock().await;
        guard.write_all(lines.as_bytes()).await?;
        guard.flush().await?;
        drop(guard);
    }

    // Phase 3: update the in-memory index after the write succeeds.
    if !to_index.is_empty() {
        let mut index_write = self.index.write();
        for signal in to_index {
            index_write.insert(signal.id, signal);
        }
    }

    Ok(ids)
}
```

The batch write uses a three-phase pipeline:

1. **Phase 1 (no write lock)**: Deduplicate and serialize all records into a single `String`. This is the expensive CPU work (JSON serialization) and it happens without holding the write lock.
2. **Phase 2 (write lock held briefly)**: Single `write_all` + `flush`. The lock is held only for the I/O duration.
3. **Phase 3 (no write lock)**: Update the in-memory index.

This design minimizes lock contention: the write lock is held only during the actual I/O, not during serialization or index updates. The batch also deduplicates within itself (via `seen_ids`) in addition to against the existing index.

### 4.5 Query Execution

Queries run entirely against the in-memory index. From `crates/roko-fs/src/file_substrate.rs`, lines 298-315:

```rust
async fn query(&self, q: &Query, ctx: &Context) -> Result<Vec<Engram>> {
    let mut matching: Vec<Engram> = self
        .index
        .read()
        .values()
        .filter(|s| matches_query(s, q, ctx))
        .cloned()
        .collect();
    matching.sort_by(|a, b| {
        b.weight_at(ctx.now_ms)
            .partial_cmp(&a.weight_at(ctx.now_ms))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if let Some(limit) = q.limit {
        matching.truncate(limit);
    }
    Ok(matching)
}
```

The query filter function supports filtering by kind, author, session, time range, minimum weight, and tags. Results are sorted by effective weight (highest first) and optionally truncated to a limit. Because the index is in-memory, query latency is identical to an in-memory store -- zero disk I/O for reads.

### 4.6 Pruning

Pruning removes decayed engrams from the in-memory index but does *not* rewrite the log file. From `crates/roko-fs/src/file_substrate.rs`, lines 317-325:

```rust
async fn prune(&self, threshold: f32, ctx: &Context) -> Result<usize> {
    let mut index = self.index.write();
    let before = index.len();
    index.retain(|_, s| s.weight_at(ctx.now_ms) > threshold);
    Ok(before - index.len())
    // Note: the log file is not rewritten here; call `compact()` to
    // reclaim disk space. Pruning from memory is the hot path; compaction
    // is a maintenance task.
}
```

This is an intentional separation of concerns. Pruning is a frequent operation (every tick of the cognitive loop). Rewriting the log file is expensive and should be done infrequently. The `compact()` method handles the disk-side cleanup -- a pattern directly analogous to how LSM-trees separate memtable eviction from SSTable compaction [9].

---

## 5. Crash-Safe Write Protocols

Roko uses two distinct crash-safe write patterns, chosen based on the nature of the data being written.

### 5.1 Append-Only Writes (JSONL Logs)

Used for: engrams, tool audit events, metrics, traces, bandit arm snapshots.

The protocol is simple: open the file with `O_APPEND`, serialize the record, write it, flush. If the process crashes mid-write, at most one partial line is lost. On replay, the partial line fails JSON parsing and is skipped.

**Correctness argument:** On POSIX systems, `O_APPEND` atomically sets the file offset to the end of the file before each write. The kernel guarantees that concurrent `O_APPEND` writes do not interleave at the byte level for writes smaller than `PIPE_BUF` (at least 4096 bytes on Linux). For larger writes, roko serializes through a mutex. The result is that each JSONL line is either fully written or not written at all [6].

**Limitation:** `flush()` without `fsync()` means data may remain in the kernel page cache and be lost on power failure (though not on process crash). The `MetricsLog` and `JsonlMetricsSink` sinks offer optional `fsync` via `sync_data()` for workloads that require stronger durability guarantees.

### 5.2 Atomic Write-Tmp-Rename (JSON Snapshots)

Used for: executor snapshots, orchestrator snapshots, run state, cascade router state, gate thresholds, bandit router state.

For non-append-only state files that are overwritten wholesale, roko uses the classic write-to-temp, rename pattern. From `crates/roko-fs/src/atomic.rs`, lines 29-65:

```rust
pub fn atomic_write_json<T: serde::Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    atomic_write_bytes(path, json.as_bytes())
}

pub fn atomic_write_bytes(path: &Path, data: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Build a temp path by appending `.tmp` to the existing extension
    // (e.g. `data.json` -> `data.json.tmp`).
    let tmp = tmp_path_for(path);
    std::fs::write(&tmp, data)?;

    // Rename is atomic on the same filesystem (POSIX guarantee).
    // If rename fails, clean up the temp file best-effort.
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }

    Ok(())
}
```

The protocol:

1. **Create parent directories** if they do not exist.
2. **Write data to a sibling `.tmp` file** (e.g., `foo.json` becomes `foo.json.tmp`).
3. **Rename over the target** -- POSIX `rename()` is atomic on the same filesystem.
4. **On rename failure**, clean up the temp file.

The temp file naming convention keeps the temp file on the same filesystem as the target, which is required for POSIX `rename()` atomicity.

**Important nuance:** The `atomic_write_bytes()` function does *not* call `fsync()` on the temp file before renaming. This means the data may not have reached stable storage at the time of rename. On a power failure (not process crash), the renamed file could contain stale or partial data. This is a deliberate simplification -- for the files this function protects (learned state, bandit arms), losing the last few snapshots is acceptable.

The `FileSubstrate::compact()` method uses a stricter variant of this pattern that *does* call `sync_all()` before rename:

```rust
// From crates/roko-fs/src/file_substrate.rs, lines 88-119
pub async fn compact(&self) -> Result<()> {
    let snapshot: Vec<Engram> = self.index.read().values().cloned().collect();
    let log_path = self.log_path();
    let tmp_path = self.root.join("engrams.jsonl.tmp");

    {
        let mut tmp = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&tmp_path)
            .await?;
        for sig in &snapshot {
            let line = serde_json::to_string(sig).map_err(RokoError::body_encode)?;
            tmp.write_all(line.as_bytes()).await?;
            tmp.write_all(b"\n").await?;
        }
        tmp.flush().await?;
        tmp.sync_all().await?;  // <-- fsync: data on stable storage before rename
    }

    fs::rename(&tmp_path, &log_path).await?;

    // Re-open the log writer to point at the compacted file.
    let new_writer = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .await?;
    *self.log_writer.lock().await = new_writer;
    Ok(())
}
```

The compaction protocol is strictly crash-safe:

1. **Snapshot the in-memory index** (read lock).
2. **Write snapshot to a temp file** (`engrams.jsonl.tmp`).
3. **Flush and fsync the temp file** -- `sync_all()` ensures data is on stable storage.
4. **Atomic rename** -- `rename()` is atomic on POSIX, so the log is never in a half-written state.
5. **Re-open the log writer** -- point at the compacted file for future appends.

If the process dies during steps 1-3, the original log file is untouched. If it dies during step 4, the rename either completes or does not -- there is no partial state. This is the same crash-safe protocol used by PostgreSQL for checkpoint writes and by SQLite for its journal mode [3, 11].

---

## 6. The Directory Layout -- `.roko/` Structure

The `RokoLayout` struct (in `crates/roko-fs/src/layout.rs`) provides a typed path catalog for the entire `.roko/` directory tree:

```text
.roko/
  VERSION              # Layout version (currently 1)
  runtime/             # pid files, sockets, locks
  memory/              # episodes, playbook, skills
    episodes.jsonl
    playbook.toml
    skills/
    archive/           # monthly JSONL archives
  plans/               # enrichment artifacts per plan
    {plan_id}/
  runs/                # per-run metrics, traces, snapshots
    {run_id}/
      metrics.jsonl
      traces/
      pointers/
  state/               # orchestrator snapshots, event logs, session state
    executor.json
    orchestrator.json
    run-state.json
    run-ledger.jsonl
    events.json
    sessions/
  config/              # config.toml, presets
  cache/               # cargo-target, context-pack-cache
  learn/               # efficiency events, cascade router, experiments
    efficiency.jsonl
    cascade-router.json
    gate-thresholds.json
    experiments.json
    playbooks/
  cold/                # cold storage archives
    2026-04.jsonl
    index.json
```

Key JSONL files in the layout:

| Path | Purpose | Write Pattern |
|---|---|---|
| `.roko/engrams.jsonl` | Main engram log (hot storage) | Append-only JSONL |
| `.roko/episodes.jsonl` | Root episode log | Append-only JSONL |
| `.roko/events.jsonl` | Runner event log | Append-only JSONL |
| `.roko/custody.jsonl` | Custody audit chain | Append-only JSONL |
| `.roko/witness.jsonl` | Witness DAG log | Append-only JSONL |
| `.roko/memory/episodes.jsonl` | Per-memory episodes | Append-only JSONL |
| `.roko/runs/{run_id}/metrics.jsonl` | Per-run metrics | Append-only JSONL |
| `.roko/learn/efficiency.jsonl` | Per-turn efficiency events | Append-only JSONL |
| `.roko/tool_audit.jsonl` | Tool dispatch audit | Append-only JSONL |
| `.roko/state/run-ledger.jsonl` | Run ledger (task starts, completions) | Append-only JSONL |

Supporting JSON files (not append-only, but atomically written):

| Path | Purpose | Write Pattern |
|---|---|---|
| `.roko/state/executor.json` | Executor snapshot for crash recovery | Atomic write-tmp-rename |
| `.roko/state/orchestrator.json` | Orchestrator snapshot | Atomic write-tmp-rename |
| `.roko/state/run-state.json` | Runner resume state | Atomic write-tmp-rename |
| `.roko/learn/cascade-router.json` | Model routing bandit state | Atomic write-tmp-rename |
| `.roko/learn/gate-thresholds.json` | Adaptive gate thresholds | Atomic write-tmp-rename |

The layout includes a version file (`.roko/VERSION`) for forward compatibility:

```rust
// Source: crates/roko-fs/src/layout.rs, lines 31-57
pub enum LayoutVersion {
    /// Initial layout: `runtime/`, `memory/`, `plans/`, `runs/`,
    /// `state/`, `config/`, `cache/`.
    V1 = 1,
}

impl LayoutVersion {
    pub const CURRENT: Self = Self::V1;

    pub const fn from_u32(n: u32) -> Option<Self> {
        match n {
            1 => Some(Self::V1),
            _ => None,
        }
    }
}
```

---

## 7. Hot/Cold Tiering -- The ColdStore Trait

Roko implements a two-tier storage architecture inspired by the hot/cold data tiering pattern used in modern storage systems [12]. Active data stays in the fast in-memory index ("hot"). Aged-out data moves to compressed monthly archives ("cold").

### 7.1 The ColdStore Trait

From `crates/roko-core/src/traits.rs`, lines 82-151:

```rust
/// Archival store for aged-out engrams that no longer need hot-path access.
///
/// While a [`Store`] keeps engrams in-memory or on fast storage for
/// real-time queries, a `ColdStore` stores them in compressed, append-only
/// archives for durability and audit trails.
///
/// # Migration flow
///
/// Store (hot) --age_out()--> ColdStore (cold/archive)
///               <--thaw()--
#[async_trait]
pub trait ColdStore: Send + Sync {
    /// Archive an engram into cold storage. Returns its content hash.
    async fn archive(&self, engram: Engram) -> Result<ContentHash>;

    /// Archive a batch of engrams. Returns the count of successfully archived.
    async fn archive_batch(&self, engrams: Vec<Engram>) -> Result<usize> {
        // Default: archive one at a time
        let mut count = 0;
        for e in engrams {
            self.archive(e).await?;
            count += 1;
        }
        Ok(count)
    }

    /// Retrieve an engram from cold storage (potentially slow).
    async fn thaw(&self, id: &ContentHash) -> Result<Option<Engram>>;

    /// Check whether an engram exists in cold storage.
    async fn contains(&self, id: &ContentHash) -> Result<bool> {
        Ok(self.thaw(id).await?.is_some())
    }

    /// Total count of archived engrams.
    async fn archived_count(&self) -> Result<usize> { Ok(0) }

    /// Total size of cold storage in bytes (approximate).
    async fn storage_bytes(&self) -> Result<u64> { Ok(0) }

    /// Purge engrams older than the given epoch (millis since UNIX epoch).
    async fn purge_before(&self, epoch_ms: i64) -> Result<usize> {
        let _ = epoch_ms;
        Ok(0)
    }

    fn name(&self) -> &'static str { "unnamed_cold_store" }
}
```

The trait provides sensible defaults for batch archiving, containment checks, and metrics, so that implementors need only provide `archive()` and `thaw()`.

### 7.2 ArchiveColdSubstrate -- Monthly JSONL Archives

The concrete cold store implementation organizes archives by month. From `crates/roko-fs/src/cold_substrate.rs`, lines 1-49:

```rust
//! Archive-backed [`ColdStore`] implementation.
//!
//! Stores aged-out engrams in compressed JSONL archive files organized by month.
//!
//! # Storage layout
//!
//! ```text
//! .roko/cold/
//!   2026-04.jsonl     # one file per month
//!   2026-03.jsonl
//!   index.json        # hash -> (month, line_offset) lookup
//! ```

pub struct ArchiveColdSubstrate {
    /// Root directory for cold storage (e.g., `.roko/cold/`).
    root: PathBuf,
    /// In-memory index: content hash -> archive location.
    index: RwLock<HashMap<ContentHash, ColdIndexEntry>>,
    /// Serializes writes.
    write_lock: Mutex<()>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct ColdIndexEntry {
    /// Archive file name (e.g., "2026-04.jsonl").
    file: String,
    /// Byte offset within the archive file.
    offset: u64,
    /// Archived timestamp (epoch millis).
    archived_at: i64,
}
```

The cold store maintains an in-memory index that maps content hashes to `(file, byte_offset)` pairs. This allows O(1) lookup of any archived engram without scanning the archive files.

The `thaw()` operation reads from a specific byte offset in an archive file -- a standard technique in log-structured storage where the index stores physical offsets rather than logical keys [1].

### 7.3 The SubstrateMigrator -- Automated Hot-to-Cold Migration

From `crates/roko-fs/src/cold_substrate.rs`, lines 301-340:

```rust
pub struct SubstrateMigrator {
    /// Weight threshold below which engrams are candidates for migration.
    pub weight_threshold: f32,
    /// Maximum age in milliseconds. Engrams older than this are migrated.
    pub max_age_ms: i64,
    /// Maximum number of engrams to migrate per batch.
    pub batch_size: usize,
}

impl SubstrateMigrator {
    pub fn new() -> Self {
        Self {
            weight_threshold: 0.1,
            max_age_ms: 7 * 24 * 3600 * 1000, // 7 days
            batch_size: 100,
        }
    }
}
```

Default migration policy:

- **Weight threshold**: 0.1 -- engrams below this effective weight are candidates.
- **Max age**: 7 days -- engrams older than this are always candidates.
- **Batch size**: 100 -- up to 100 engrams per migration batch.

The migration flow is:

```
Hot Store -> query for aged-out engrams -> archive to ColdStore -> prune from hot
```

This three-tier lifecycle (hot in-memory -> cold monthly archives -> GC deletion) prevents unbounded growth while preserving audit trails for the configurable retention period.

---

## 8. The GC (Garbage Collection) Engine

The GC engine scans the entire `.roko/` directory tree and removes data that exceeds configured retention limits.

### 8.1 Retention Policy

From `crates/roko-fs/src/gc.rs`, lines 31-55:

```rust
pub struct RetentionPolicy {
    /// Maximum number of episode records to keep.
    pub max_episodes: usize,
    /// Maximum age of run directories in days.
    pub max_run_age_days: u32,
    /// Maximum age of archive entries in days.
    pub max_archive_age_days: u32,
    /// Size in MB above which GC is recommended.
    pub size_threshold_mb: u64,
    /// Maximum number of context-pack cache entries.
    pub max_cache_entries: usize,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_episodes: 200,
            max_run_age_days: 7,
            max_archive_age_days: 30,
            size_threshold_mb: 500,
            max_cache_entries: 2000,
        }
    }
}
```

Default retention limits:

| Store | Default Limit | Strategy |
|---|---|---|
| Episodes | 200 max | Keep most recent N |
| Runs | 7 days | Delete runs older than N days |
| Archives | 30 days | Delete archive entries older than N days |
| Total size | 500 MB | Auto-GC trigger when `.roko/` exceeds this |
| Cache entries | 2000 | Delete oldest entries beyond limit |

### 8.2 GC Safety Invariants

From the module documentation (`crates/roko-fs/src/gc.rs`, lines 20-24):

```
- **Never touches `config/`** -- user configuration is sacred.
- **Never touches `runtime/`** -- PID files and locks are the running
  process's responsibility.
- **Idempotent** -- re-running GC when nothing exceeds limits is a no-op.
```

### 8.3 GC Engine and Modes

The `GcEngine` (from `crates/roko-fs/src/gc.rs`, lines 96-100) operates on a `RokoLayout` under a `RetentionPolicy`:

```rust
pub struct GcEngine {
    layout: RokoLayout,
    policy: RetentionPolicy,
}
```

The engine supports three modes:

1. **`scan()` / `dry_run()`**: Report what would be removed without touching the filesystem.
2. **`collect()`**: Actually remove the candidates. Individual removal failures do not abort the run -- they are counted in `failed_count` for reporting.
3. **`should_auto_gc()`**: Check if the total `.roko/` size exceeds the threshold.

The GC report provides full transparency:

```rust
pub struct GcCandidate {
    pub path: PathBuf,
    pub reason: String,
    pub size_bytes: u64,
}

pub struct GcReport {
    pub candidates: Vec<GcCandidate>,
    pub total_bytes: u64,
    pub removed_count: usize,
    pub failed_count: usize,
}
```

---

## 9. Specialized JSONL Sinks

Beyond the main engram store, roko-fs provides several purpose-built JSONL sinks that follow the same append-only pattern but are optimized for specific workloads.

### 9.1 ToolAuditLog -- Append-Only Tool Dispatch Audit

From `crates/roko-fs/src/tool_audit.rs`, lines 52-55:

```rust
pub struct ToolAuditLog {
    path: PathBuf,
    writer: Mutex<BufWriter<tokio::fs::File>>,
}
```

Records every tool call admission and result as tagged JSONL lines:

```json
{"kind":"admit","ts_ms":1712345678000,"call":{"id":"c1","name":"read_file",...}}
{"kind":"result","ts_ms":1712345679000,"call_id":"c1","call_name":"read_file","result":{...}}
```

The `kind` discriminator allows consumers to filter while `tail -f`-ing a single file. The log uses `tokio::sync::Mutex` (async) for write serialization since tool dispatches are async operations.

### 9.2 JsonlTraceSink -- Per-Trace File JSONL

From `crates/roko-fs/src/trace_sink.rs`, lines 54-59:

```rust
pub struct JsonlTraceSink {
    root: PathBuf,
    inner: Arc<Mutex<Inner>>,
    clock: Clock,
}
```

Organizes traces into daily directories with one file per trace:

```
.roko/traces/
  2026-04-05/
    0102030405060708090a0b0c0d0e0f10.jsonl
  2026-04-06/
    ...
```

Design choices:

- **Synchronous I/O** -- traces are best-effort and never block agent execution. The `TraceSink` trait itself is `fn` (not async).
- **One file per trace** -- prevents interleaving between concurrent traces. Two traces running simultaneously write to separate files.
- **Clock injection** -- `with_clock()` allows tests to pin the date for deterministic rotation testing.
- **`parking_lot::Mutex`** (synchronous) -- because the trait methods are synchronous.

### 9.3 MetricsLog -- Per-Run Task Metrics

From `crates/roko-fs/src/metrics.rs`, lines 33-36:

```rust
pub struct MetricsLog {
    path: PathBuf,
    fsync: bool,
}
```

Supports configurable fsync: by default `sync_data()` is called after each append, ensuring at most one in-flight record is lost on process death. Callers that need higher throughput can construct with `without_fsync()`.

Unlike `FileSubstrate`, `MetricsLog` opens and closes the file for each `append()` call rather than holding a persistent file handle. This simplifies the implementation at the cost of some throughput -- acceptable for metric records produced at most a few per second.

### 9.4 JsonlMetricsSink -- Tool-Call Aggregate Metrics

From `crates/roko-fs/src/tool_metrics_sink.rs`, lines 54-58:

```rust
pub struct JsonlMetricsSink {
    path: PathBuf,
    fsync: bool,
    write_lock: Mutex<()>,
}
```

This sink uses `parking_lot::Mutex` (synchronous) and `std::fs` (synchronous I/O), not tokio. It implements the `roko_core::tool::MetricsSink` trait. Writes are best-effort: failures are reported to stderr but do not panic the agent runtime. Malformed lines from partial writes are silently skipped on read-back.

### 9.5 BanditStore -- Multi-Armed Bandit Arm Persistence

From `crates/roko-fs/src/bandit.rs`, lines 44-47:

```rust
pub struct BanditStore {
    /// Root directory (`.roko/bandit/`).
    root: PathBuf,
}
```

Each bandit key maps to its own `.jsonl` file under `.roko/bandit/`. Writes append full snapshots of the arm table; reads return the most recent valid snapshot. Keys are sanitized (non-alphanumeric characters replaced with `_`) to keep filenames portable.

### 9.6 PointerStore -- Large Payload Offloading

From `crates/roko-fs/src/pointer.rs`, lines 25-31:

```rust
pub struct PointerStore {
    /// Root directory (typically `.roko/`).
    root: PathBuf,
    /// Payloads at or below this size are considered inline and need
    /// not be stored. Default: 4096 bytes.
    max_inline_bytes: usize,
}
```

Stores large tool-result payloads (above 4 KiB by default) on disk and references them by pointer ID. Layout: `{root}/runs/{run_id}/pointers/{pointer_id}`. Operations are synchronous (`std::fs`) because pointer reads happen on the tool-loop hot path.

---

## 10. The Archiver -- Compressing Old Data Into Summaries

The `Archiver` provides a higher-level data lifecycle -- lossy compression of old data into statistical summaries. From `crates/roko-fs/src/archive.rs`, lines 22-61:

```rust
pub struct ArchiveEntry {
    pub kind: ArchiveKind,
    pub date: NaiveDate,
    pub source_id: String,
    pub item_count: usize,
    pub stats: ArchiveStats,
    pub archived_at: DateTime<Utc>,
}

pub struct ArchiveStats {
    pub total_bytes: u64,
    pub total_iterations: u64,
    pub gate_passes: u64,
    pub gate_failures: u64,
    pub total_cost_usd: f64,
}
```

Archives live under `.roko/memory/archive/` in monthly JSONL files (e.g., `2026-04.jsonl`). The archiver supports:

- **Run archival**: Summarize a run directory into a single `ArchiveEntry`, then delete the raw directory.
- **Episode archival**: Compress excess episodes into iteration summaries with aggregate statistics.
- **Daily sampling**: Keep only one entry per day from a set of entries.
- **Iteration summaries**: Reduce N entries into one aggregate record.

This is the system's lossy compression tier: raw data is replaced by statistical summaries that preserve the metrics (gate passes, cost, iteration counts) while discarding the full records. This is conceptually similar to how time-series databases like Prometheus downsample old data into larger time windows.

---

## 11. Crash Recovery Architecture

Roko uses a layered crash recovery strategy:

### 11.1 Dual Recovery Mechanism

1. **Executor snapshots**: Periodic point-in-time captures written atomically to `.roko/state/executor.json` using the write-tmp-rename pattern.
2. **Event log replay**: Reconstruction from the append-only, hash-chained event log.

### 11.2 Snapshot Write Protocol

```
1. Write to .roko/state/executor.json.tmp
2. fsync the temp file
3. Rename .roko/state/executor.json.tmp -> .roko/state/executor.json
```

Auto-save every 5 actions, meaning at most 5 actions of work can be lost in a crash.

### 11.3 Merged Recovery

When both snapshot and event log are available, the recovery engine merges them:

- **Event log wins on conflict** -- it may contain events recorded after the snapshot.
- **Disjoint plans are combined** -- plans from both sources are included.
- **Hash verification** -- BLAKE3 checksums detect corruption at multiple levels.

### 11.4 Torn Write Detection

The snapshot file format includes structural safeguards:

```
[4 bytes] magic: 0x524F4B4F ("ROKO")
[4 bytes] version: 1
[4 bytes] payload_length (little-endian)
[N bytes] JSON payload
[32 bytes] BLAKE3 hash of payload
[4 bytes] magic trailer: 0x454E4421 ("END!")
```

This detects truncation, bit flips, and partial writes. The magic-header-payload-hash-trailer pattern is used by many systems including PostgreSQL's control file format and Apache Kafka's record batches.

---

## 12. The Observability Layer

The `FsObservabilitySinks` struct (from `crates/roko-fs/src/observability.rs`, lines 18-23) wires together all filesystem-backed sinks:

```rust
pub struct FsObservabilitySinks {
    pub trace_sink: Arc<JsonlTraceSink>,
    pub metrics_sink: Arc<JsonlMetricsSink>,
}
```

This provides a single initialization point that creates backing directories for both sinks. The `initialize()` method is idempotent -- calling it multiple times only re-validates directory existence.

---

## 13. Design Patterns Summary

The roko-fs crate uses a consistent set of patterns across all its components:

### 13.1 The Append-Only JSONL Pattern

Used by: `FileSubstrate`, `ToolAuditLog`, `MetricsLog`, `JsonlMetricsSink`, `BanditStore`, `Archiver`, `ArchiveColdSubstrate`.

Properties:
- One JSON object per line, terminated by `\n`.
- File opened with `create(true).append(true)`.
- Concurrent writes serialized by a mutex.
- Malformed lines silently skipped on replay.
- File is never rewritten (except during compaction).

### 13.2 The Write-Tmp-Rename Pattern

Used by: `atomic_write_json()`, `atomic_write_bytes()`, `FileSubstrate::compact()`, executor snapshot saves.

Properties:
- Write to a sibling `.tmp` file.
- Optional `fsync` / `sync_all()` before rename (used by `compact()`, omitted by `atomic_write_bytes()`).
- `rename()` over the target (POSIX atomic).
- Clean up temp file on rename failure.

### 13.3 The In-Memory Index + On-Disk Log Pattern

Used by: `FileSubstrate`, `ArchiveColdSubstrate`.

Properties:
- Reads go through an in-memory `HashMap` (zero I/O latency).
- Writes go to both the on-disk log and the in-memory index.
- On startup, the log is replayed into the index.
- Index updates happen *after* successful disk writes.

### 13.4 The Configurable Fsync Pattern

Used by: `MetricsLog`, `JsonlMetricsSink`.

Properties:
- `sync_data()` called after each write by default.
- `without_fsync()` available for higher throughput when durability per-record is not critical.

---

## 14. Comparison with IronClaw's Dual-Backend Persistence

IronClaw uses a fundamentally different persistence architecture from roko. Understanding both approaches illuminates their respective strengths and the integration opportunities.

### 14.1 IronClaw's Architecture: PostgreSQL + libSQL/Turso

IronClaw's persistence is built on a **dual-backend database architecture** (see `src/db/CLAUDE.md`):

- **PostgreSQL** -- the default production backend, with full ACID transactions, connection pooling (`deadpool-postgres`), and a migration framework (`refinery`). Uses native `JSONB`, `VECTOR`, `TIMESTAMPTZ`, and `tsvector` types.
- **libSQL/Turso** -- an alternative backend for embedded or edge deployments. Uses SQLite-compatible SQL with FTS5 for text search and `libsql_vector_idx` for vector similarity. Connections are created per-operation (no pool).

The `Database` supertrait composes seven sub-traits (`ConversationStore`, `JobStore`, `SandboxStore`, `RoutineStore`, `ToolFailureStore`, `SettingsStore`, `WorkspaceStore`) providing ~78 async methods total. Both backends implement this full surface.

IronClaw's workspace memory system (see `src/workspace/README.md`) uses **hybrid search** combining FTS (keyword) and vector similarity (semantic) via Reciprocal Rank Fusion (RRF):

```
score(d) = SUM 1/(k + rank(d)) for each method where d appears
```

### 14.2 Architectural Comparison

| Concern | Roko (roko-fs) | IronClaw |
|---|---|---|
| **Storage engine** | Append-only JSONL + in-memory HashMap | PostgreSQL / libSQL relational tables |
| **Hot storage** | `FileSubstrate` (JSONL + in-memory index) | PostgreSQL / libSQL tables |
| **Cold storage** | `ArchiveColdSubstrate` (monthly JSONL archives) | No explicit cold tier |
| **Audit trail** | `ToolAuditLog`, `custody.jsonl`, `witness.jsonl` | `ActionRecord` in tool dispatch, `job_actions` table |
| **Crash recovery** | Snapshot + event log replay | Database transactions (ACID) |
| **GC policy** | `GcEngine` with `RetentionPolicy` | "LLM data is never deleted" invariant; no formalized GC |
| **Identity model** | Content-addressed (BLAKE3 `ContentHash`) | UUID-based row identity |
| **Decay model** | `Decay` enum (None/HalfLife/Ttl/Ebbinghaus) | No temporal decay |
| **Multi-user** | Single-user (`.roko/` per project) | Multi-user with scope isolation |
| **Replication** | No built-in replication | Database-level (PostgreSQL streaming, Turso edge sync) |
| **Transactions** | No transactions (append-only log) | Full ACID transactions |
| **Schema evolution** | JSON schema evolution (forward-compatible) | SQL migrations (refinery / INCREMENTAL_MIGRATIONS) |
| **Query complexity** | In-memory scan with filters | SQL queries with B-tree + HNSW/FTS indices |
| **Concurrent writers** | Single-process mutex | Database-level concurrency (PostgreSQL pool, libSQL WAL) |
| **Search** | In-memory filter + optional HDC similarity | Hybrid FTS + vector via RRF |
| **Tool metrics** | `JsonlMetricsSink` (file-per-sink) | Database-backed metrics (`llm_calls`, `job_actions`) |

### 14.3 What Roko Gets Right That IronClaw Could Adopt

**1. Formalized retention policies.** Roko's `RetentionPolicy` struct makes GC behavior explicit, configurable, and testable. IronClaw has the "LLM data is never deleted" invariant but no formalized policy for non-LLM data (cache entries, intermediate state, old tool results). A `RetentionPolicy` could be added:

```rust
// Sketch: IronClaw retention policy
pub struct RetentionPolicy {
    /// Maximum age of tool execution traces (days).
    pub max_trace_age_days: u32,
    /// Maximum number of cached context packs.
    pub max_cache_entries: usize,
    /// Size threshold for auto-GC (MB).
    pub size_threshold_mb: u64,
    /// Categories exempt from GC (enforces "LLM data is never deleted").
    pub protected_categories: Vec<String>,
}
```

**2. Hot/cold tiering for workspace memory.** IronClaw's workspace memory system accumulates documents over time. A cold tier could archive old, low-relevance memories while keeping them retrievable via a `thaw`-style operation. This is especially relevant for the libSQL backend, where a single SQLite file grows without bound.

**3. Append-only audit logs for tool dispatches.** IronClaw routes all actions through `ToolDispatcher::dispatch()` and records `ActionRecord`s in the `job_actions` database table. A complementary append-only JSONL audit log would provide:
   - A tamper-evident audit trail separate from the mutable database.
   - `tail -f` observability for live debugging without database queries.
   - A simple backup story (just copy the file).

**4. Content-addressed deduplication for workspace memory.** Roko's `ContentHash` makes `Store.put()` idempotent. IronClaw's `memory_write` tool currently uses path-based addressing. Adding a content hash to `memory_chunks` would prevent duplicate chunks from accumulating when the same content is written to different paths.

**5. Temporal decay for memory relevance.** Roko's `Decay` model (half-life, TTL, Ebbinghaus forgetting curves) provides a principled way to reduce the relevance of old data without deleting it. IronClaw's `memory_search` tool could weight results by temporal decay, improving result relevance for long-lived agents.

### 14.4 What IronClaw Gets Right That Roko Cannot Match

**1. Multi-user isolation.** IronClaw's database model natively supports multiple users with scope isolation. Roko's `.roko/` directory is fundamentally single-user.

**2. ACID transactions.** IronClaw's PostgreSQL backend provides full transaction support. Roko's append-only model has no rollback capability -- a partially completed multi-step operation leaves the log in a mixed state.

**3. Rich query capabilities.** IronClaw can run SQL joins, aggregations, and complex filters with database-level optimization. Roko's in-memory scan is O(N) over the entire dataset for every query.

**4. Native vector search.** IronClaw's hybrid FTS + vector search via RRF provides semantically-aware retrieval. Roko's optional HDC similarity search is less mature.

**5. Horizontal scalability.** PostgreSQL streaming replication and Turso's edge sync provide paths to horizontal scaling that flat-file JSONL cannot match.

### 14.5 Integration Plan

The right integration strategy is not to replace IronClaw's database with JSONL files, but to adopt specific patterns where they add value:

| Priority | Pattern to Adopt | IronClaw Location | Implementation |
|---|---|---|---|
| P0 | Formalized retention policy | `src/db/mod.rs` or new `src/retention/` | Add `RetentionPolicy` struct and `GcEngine` that runs against the database, respecting the "LLM data never deleted" invariant for `llm_calls` and `job_actions` tables |
| P1 | Append-only tool audit log | `src/tools/dispatch.rs` | Add a `ToolAuditLog` (JSONL file at `~/.ironclaw/audit/tool_audit.jsonl`) as a complement to the database `ActionRecord`, written on every `dispatch()` call |
| P1 | Content-hash deduplication | `src/workspace/` | Add BLAKE3 content hashing to `memory_chunks`; skip re-indexing if the chunk's hash already exists in the database |
| P2 | Temporal decay weighting | `src/workspace/` | Add a `decay_factor(age_days)` to `memory_search` scoring, so older memories rank lower unless their content is highly relevant |
| P2 | Hot/cold workspace tiering | `src/db/mod.rs`, `src/workspace/` | Add a `ColdMemoryStore` trait and archive endpoint that moves old `memory_chunks` rows to a compressed archive table, with a `thaw` operation for on-demand retrieval |
| P3 | Crash-safe config writes | `src/settings.rs` | Use atomic write-tmp-rename for `settings.json` updates (currently uses `std::fs::write` directly) |

---

## 15. Complete Module Map

All source files in the `roko-fs` crate:

| File | Purpose |
|---|---|
| `crates/roko-fs/src/lib.rs` | Crate root, module declarations, public re-exports |
| `crates/roko-fs/src/file_substrate.rs` | Core JSONL storage engine (FileSubstrate) |
| `crates/roko-fs/src/cold_substrate.rs` | Cold storage tier (ArchiveColdSubstrate, SubstrateMigrator) |
| `crates/roko-fs/src/gc.rs` | Garbage collection engine (GcEngine, RetentionPolicy, GcReport) |
| `crates/roko-fs/src/archive.rs` | Archival format (Archiver, ArchiveEntry, daily sampling) |
| `crates/roko-fs/src/atomic.rs` | Atomic write helpers (write-tmp-rename) |
| `crates/roko-fs/src/layout.rs` | `.roko/` directory layout (RokoLayout, LayoutVersion) |
| `crates/roko-fs/src/trace_sink.rs` | Per-trace JSONL sink (JsonlTraceSink) |
| `crates/roko-fs/src/tool_audit.rs` | Tool dispatch audit log (ToolAuditLog) |
| `crates/roko-fs/src/metrics.rs` | Per-run task metrics (MetricsLog) |
| `crates/roko-fs/src/tool_metrics_sink.rs` | Tool-call aggregate metrics (JsonlMetricsSink) |
| `crates/roko-fs/src/pointer.rs` | Large payload offloading (PointerStore) |
| `crates/roko-fs/src/bandit.rs` | Multi-armed bandit arm persistence (BanditStore) |
| `crates/roko-fs/src/observability.rs` | Observability sink wiring (FsObservabilitySinks) |

---

## 16. Key Takeaways

1. **Append-only JSONL is the fundamental storage primitive.** Every piece of durable state in roko is either an append-only JSONL log or an atomically-written JSON snapshot. There are no databases, no B-trees, no LSM trees. This radical simplicity makes the system easy to reason about, debug, and recover from crashes.

2. **The in-memory index provides read performance.** The JSONL log is write-optimized; the in-memory `HashMap` is read-optimized. Together they provide both durability and query speed. The tradeoff is memory usage (the entire hot dataset must fit in RAM), but for typical agent workloads (tens of thousands of engrams), this is well within bounds.

3. **Hot/cold tiering manages the growth trajectory.** Active data stays in the fast in-memory index. Aged-out data moves to compressed monthly archives. The GC engine enforces retention limits. This three-layer lifecycle (hot -> cold -> deleted) prevents unbounded growth while preserving audit trails.

4. **Crash safety is structural, not transactional.** There are no transactions, no WAL, no commit protocol. Safety comes from the physical properties of the storage format: append-only writes can only corrupt the last line, and atomic renames prevent partial snapshots. The system trades generality for simplicity and correctness.

5. **Every subsystem follows the same patterns.** Whether it is tool audit, trace logging, metrics, bandit arms, or the main engram store, the same patterns recur: append-only JSONL, malformed-line skipping, mutex-serialized writes, optional fsync. This consistency makes the codebase predictable and reduces the surface area for bugs.

6. **IronClaw and roko solve different problems with different tradeoffs.** Roko optimizes for single-user simplicity, human-readable storage, and zero-dependency deployment. IronClaw optimizes for multi-user isolation, rich queries, and horizontal scalability. The best integration adopts roko's *patterns* (retention policies, audit logs, content-addressed dedup, temporal decay) within IronClaw's existing database infrastructure, rather than replacing the database.

---

## References

[1] M. Rosenblum and J. K. Ousterhout, "The Design and Implementation of a Log-Structured File System," *ACM Transactions on Computer Systems*, vol. 10, no. 1, pp. 26-52, Feb. 1992. https://dl.acm.org/doi/10.1145/146941.146943

[2] C. Mohan, D. Haderle, B. Lindsay, H. Pirahesh, and P. Schwarz, "ARIES: A Transaction Recovery Method Supporting Fine-Granularity Locking and Partial Rollbacks Using Write-Ahead Logging," *ACM Transactions on Database Systems*, vol. 17, no. 1, pp. 94-162, Mar. 1992. https://dl.acm.org/doi/10.1145/128765.128770

[3] PostgreSQL Global Development Group, "Write-Ahead Logging (WAL)," *PostgreSQL 18 Documentation*, ch. 28.3. https://www.postgresql.org/docs/current/wal-intro.html

[4] M. Fowler, "Event Sourcing," *martinfowler.com*, Dec. 2005. https://martinfowler.com/eaaDev/EventSourcing.html

[5] Microsoft Azure Architecture Center, "Event Sourcing pattern," *Azure Architecture Center*. https://learn.microsoft.com/en-us/azure/architecture/patterns/event-sourcing

[6] D. Luu, "Files are hard," *danluu.com*, 2017. https://danluu.com/file-consistency/ -- A comprehensive survey of crash-safety assumptions in POSIX filesystems, including the subtleties of `rename()` atomicity, `fsync()` requirements, and the differences between process crash and power failure.

[7] StoneFly, "Content Addressable Storage: CAS, Deduplication Explained." https://stonefly.com/blog/content-addressable-storage-enterprise-guide/

[8] H. Ebbinghaus, *Uber das Gedachtnis: Untersuchungen zur experimentellen Psychologie* (Memory: A Contribution to Experimental Psychology), Leipzig: Duncker & Humblot, 1885.

[9] P. O'Neil, E. Cheng, D. Gawlick, and E. O'Neil, "The Log-Structured Merge-Tree (LSM-Tree)," *Acta Informatica*, vol. 33, no. 4, pp. 351-385, 1996. See also: Y. Zhang et al., "Rethinking LSM-tree based Key-Value Stores: A Survey," arXiv:2507.09642, Jul. 2025. https://arxiv.org/html/2507.09642v1

[10] A. Lakshman and P. Malik, "Cassandra: A Decentralized Structured Storage System," *ACM SIGOPS Operating Systems Review*, vol. 44, no. 2, pp. 35-40, Apr. 2010.

[11] J. Bornholt, A. Raber, S. Smith, C. Torlak, D. Woodruff, and L. Ceze, "Specifying and Checking File System Crash-Consistency Models," *Proceedings of ASPLOS 2016*. https://jamesbornholt.com/papers/ferrite-asplos16.pdf

[12] Z. Liu et al., "HotRAP: Hot Record Retention and Promotion for LSM-trees with Tiered Storage," *Proceedings of USENIX ATC 2025*, 2024. https://arxiv.org/pdf/2402.02070

[13] ESAA-Conversational, "An Event-Sourced Memory Layer for Continuity, Handoff, and Curation Across Heterogeneous LLM Coding Agents," arXiv:2606.23752, Jun. 2026. https://arxiv.org/pdf/2606.23752 -- Directly relevant: applies event sourcing as a persistence pattern for AI coding agent memory, independently validating roko's approach.

[14] CMU Database Group, "Lecture #21: Database Logging," 15-445/645 Database Systems (Fall 2025). https://15445.courses.cs.cmu.edu/fall2025/notes/21-logging.pdf
