// Database models for ConHub

pub mod user;
pub mod connected_account;
pub mod document;
pub mod sync_job;
pub mod billing;
pub mod security;

pub use user::*;
pub use connected_account::*;
pub use document::*;
pub use sync_job::*;
pub use billing::*;
pub use security::*;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Common traits for all models
pub trait Model {
    type Id;
    
    fn id(&self) -> &Self::Id;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 20,
            offset: 0,
        }
    }
}

impl Pagination {
    pub fn new(limit: i64, offset: i64) -> Self {
        Self { limit, offset }
    }

    pub fn page(page: i64, per_page: i64) -> Self {
        Self {
            limit: per_page,
            offset: (page - 1) * per_page,
        }
    }
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total: i64, pagination: &Pagination) -> Self {
        Self {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        }
    }

    pub fn has_more(&self) -> bool {
        self.offset + self.limit < self.total
    }

    pub fn page(&self) -> i64 {
        (self.offset / self.limit) + 1
    }

    pub fn total_pages(&self) -> i64 {
        (self.total + self.limit - 1) / self.limit
    }
}
