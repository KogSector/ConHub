use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::{OwnedSemaphorePermit, RwLock, Semaphore};
use uuid::Uuid;

use crate::models::{ChunkRecord, ContentFingerprint, IndexingStatus, MutationSet, RowSnapshot, SourceVersionKind};

#[derive(Debug)]
pub struct RowProcessingGuard {
    row_key: String,
    state: Arc<IndexerState>,
    processing_permit: Option<OwnedSemaphorePermit>,
    snapshot_before: Option<RowSnapshot>,
}

impl RowProcessingGuard {
    pub fn row_key(&self) -> &str {
        &self.row_key
    }

    pub fn previous_snapshot(&self) -> Option<&RowSnapshot> {
        self.snapshot_before.as_ref()
    }
}

impl Drop for RowProcessingGuard {
    fn drop(&mut self) {
        self.state.as_ref().release_row(&self.row_key, self.snapshot_before.take());
    }
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryIndexedStore {
    rows: Arc<DashMap<String, RowSnapshot>>,
}

impl InMemoryIndexedStore {
    pub fn new() -> Self {
        Self {
            rows: Arc::new(DashMap::new()),
        }
    }

    pub fn upsert(&self, snapshot: RowSnapshot) {
        self.rows.insert(snapshot.row_id.clone(), snapshot);
    }

    pub fn remove(&self, row_key: &str) {
        self.rows.remove(row_key);
    }

    pub fn iter(&self) -> Vec<RowSnapshot> {
        self.rows.iter().map(|entry| entry.value().clone()).collect()
    }

    pub fn get(&self, row_key: &str) -> Option<RowSnapshot> {
        self.rows.get(row_key).map(|entry| entry.value().clone())
    }
}

#[derive(Debug, Clone)]
pub struct MemoizationCache {
    map: Arc<DashMap<String, Vec<ChunkRecord>>>,
}

impl MemoizationCache {
    pub fn new() -> Self {
        Self {
            map: Arc::new(DashMap::new()),
        }
    }

    pub fn get(&self, fingerprint: &ContentFingerprint) -> Option<Vec<ChunkRecord>> {
        self.map
            .get(&fingerprint.value)
            .map(|entry| entry.value().clone())
    }

    pub fn put(&self, fingerprint: ContentFingerprint, chunks: Vec<ChunkRecord>) {
        self.map.insert(fingerprint.value, chunks);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowState {
    pub fingerprint: Option<ContentFingerprint>,
    pub version_kind: SourceVersionKind,
    pub last_mutation: Option<MutationSet>,
}

#[derive(Debug, Default)]
pub struct IndexerState {
    rows: DashMap<String, RowStateEntry>,
    semaphore_registry: DashMap<String, Arc<Semaphore>>,
}

#[derive(Debug)]
pub struct RowStateEntry {
    pub state: RowState,
    pub generation: u64,
}

#[derive(Debug, Clone)]
pub struct IndexStateManager {
    inner: Arc<IndexerState>,
    memoization: MemoizationCache,
    indexed_store: InMemoryIndexedStore,
}

impl IndexStateManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(IndexerState::default()),
            memoization: MemoizationCache::new(),
            indexed_store: InMemoryIndexedStore::new(),
        }
    }

    pub fn memoization(&self) -> &MemoizationCache {
        &self.memoization
    }

    pub fn indexed_store(&self) -> &InMemoryIndexedStore {
        &self.indexed_store
    }

    pub async fn begin_row_processing(
        &self,
        row_key: String,
    ) -> RowProcessingGuard {
        let sem = self
            .inner
            .semaphore_registry
            .entry(row_key.clone())
            .or_insert_with(|| Arc::new(Semaphore::new(1)))
            .clone();

        let permit = sem.acquire_owned().await.expect("semaphore poisoned");

        let snapshot_before = self
            .inner
            .rows
            .get(&row_key)
            .map(|entry| Self::build_snapshot(&row_key, &entry.state));

        RowProcessingGuard {
            row_key,
            state: self.inner.clone(),
            processing_permit: Some(permit),
            snapshot_before,
        }
    }

    pub fn update_row_state(
        &self,
        row_key: &str,
        fingerprint: Option<ContentFingerprint>,
        kind: SourceVersionKind,
        mutation: Option<MutationSet>,
    ) -> RowSnapshot {
        let snapshot = RowSnapshot {
            row_id: row_key.to_string(),
            fingerprint: fingerprint.clone(),
            version_kind: kind,
            last_mutation_at: chrono::Utc::now(),
            mutation: mutation.clone(),
        };

        self.inner.rows.insert(
            row_key.to_string(),
            RowStateEntry {
                state: RowState {
                    fingerprint,
                    version_kind: kind,
                    last_mutation: mutation,
                },
                generation: self.next_generation(),
            },
        );

        snapshot
    }

    pub fn remove_row_state(&self, row_key: &str) {
        self.inner.rows.remove(row_key);
    }

    pub fn release_row(&self, row_key: &str, snapshot_before: Option<RowSnapshot>) {
        if let Some(snapshot) = snapshot_before {
            self.indexed_store.upsert(snapshot);
        }
    }

    fn build_snapshot(row_key: &str, row_state: &RowState) -> RowSnapshot {
        RowSnapshot {
            row_id: row_key.to_string(),
            fingerprint: row_state.fingerprint.clone(),
            version_kind: row_state.version_kind,
            last_mutation_at: chrono::Utc::now(),
            mutation: row_state.last_mutation.clone(),
        }
    }

    fn next_generation(&self) -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static GENERATION: AtomicU64 = AtomicU64::new(1);
        GENERATION.fetch_add(1, Ordering::Relaxed)
    }
}

#[derive(Debug, Clone)]
pub struct JobRegistry {
    jobs: Arc<DashMap<String, IndexingStatus>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(DashMap::new()),
        }
    }

    pub fn register(&self, status: IndexingStatus) -> String {
        let id = Uuid::new_v4().to_string();
        self.jobs.insert(id.clone(), status);
        id
    }

    pub fn update_status(&self, job_id: &str, status: IndexingStatus) {
        self.jobs.insert(job_id.to_string(), status);
    }

    pub fn status(&self, job_id: &str) -> Option<IndexingStatus> {
        self.jobs.get(job_id).map(|entry| entry.value().clone())
    }

    pub fn all(&self) -> HashMap<String, IndexingStatus> {
        self.jobs
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
}