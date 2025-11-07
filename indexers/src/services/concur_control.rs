use std::sync::Arc;
use tokio::sync::{Semaphore, OwnedSemaphorePermit};
use anyhow::Result;

pub const BYTES_UNKNOWN_YET: Option<fn() -> usize> = None;

#[derive(Debug, Clone)]
pub struct Options {
    pub max_concurrent_operations: usize,
    pub max_memory_usage: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_concurrent_operations: 10,
            max_memory_usage: None,
        }
    }
}

pub struct ConcurrencyController {
    semaphore: Arc<Semaphore>,
}

impl ConcurrencyController {
    pub fn new(options: &Options) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(options.max_concurrent_operations)),
        }
    }

    pub async fn acquire(&self, _size_estimator: Option<fn() -> usize>) -> Result<OwnedSemaphorePermit> {
        let permit = self.semaphore.clone().acquire_owned().await
            .map_err(|_| anyhow::anyhow!("Failed to acquire semaphore permit"))?;
        Ok(permit)
    }
}

pub struct CombinedConcurrencyController {
    inner: ConcurrencyController,
}

impl CombinedConcurrencyController {
    pub fn new(options: &Options) -> Self {
        Self {
            inner: ConcurrencyController::new(options),
        }
    }

    pub async fn acquire(&self, _size_estimator: Option<fn() -> usize>) -> Result<CombinedConcurrencyControllerPermit> {
        let permit = self.inner.acquire(_size_estimator).await?;
        Ok(CombinedConcurrencyControllerPermit { _permit: permit })
    }

    /// Acquire a permit while reserving memory based on a size estimator.
    /// This minimal implementation ignores the estimator and just acquires a permit.
    pub async fn acquire_bytes_with_reservation<F>(&self, _size_estimator: F) -> Result<CombinedConcurrencyControllerPermit>
    where
        F: Fn() -> usize,
    {
        let permit = self.inner.acquire(BYTES_UNKNOWN_YET).await?;
        Ok(CombinedConcurrencyControllerPermit { _permit: permit })
    }
}

pub struct CombinedConcurrencyControllerPermit {
    _permit: OwnedSemaphorePermit,
}