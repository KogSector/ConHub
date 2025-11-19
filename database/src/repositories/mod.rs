// Repository pattern for database operations

pub mod user;
pub mod connected_account;
pub mod document;
pub mod sync_job;
pub mod billing;
pub mod security;

pub use user::UserRepository;
pub use connected_account::ConnectedAccountRepository;
pub use document::DocumentRepository;
pub use sync_job::SyncJobRepository;
pub use billing::BillingRepository;
pub use security::SecurityRepository;

use async_trait::async_trait;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Model, Pagination, PaginatedResult};

/// Base repository trait with common CRUD operations
#[async_trait]
pub trait Repository<T: Model> {
    /// Create a new entity
    async fn create(&self, entity: &T) -> Result<T>;

    /// Find entity by ID
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>>;

    /// Update an existing entity
    async fn update(&self, id: &T::Id, entity: &T) -> Result<T>;

    /// Delete an entity by ID
    async fn delete(&self, id: &T::Id) -> Result<bool>;

    /// List all entities with pagination
    async fn list(&self, pagination: &Pagination) -> Result<PaginatedResult<T>>;
}

/// Repository manager that provides access to all repositories
pub struct RepositoryManager {
    pool: PgPool,
}

impl RepositoryManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn users(&self) -> UserRepository {
        UserRepository::new(self.pool.clone())
    }

    pub fn connected_accounts(&self) -> ConnectedAccountRepository {
        ConnectedAccountRepository::new(self.pool.clone())
    }

    pub fn documents(&self) -> DocumentRepository {
        DocumentRepository::new(self.pool.clone())
    }

    pub fn sync_jobs(&self) -> SyncJobRepository {
        SyncJobRepository::new(self.pool.clone())
    }

    pub fn billing(&self) -> BillingRepository {
        BillingRepository::new(self.pool.clone())
    }

    pub fn security(&self) -> SecurityRepository {
        SecurityRepository::new(self.pool.clone())
    }
}
