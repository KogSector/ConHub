// Common imports used throughout the indexers crate
pub use anyhow::{anyhow, bail, Context, Result};

// Re-export our custom macros
pub use crate::{api_bail, api_error};
pub use serde::{Deserialize, Serialize};
pub use std::collections::{BTreeMap, HashMap, HashSet};
pub use std::sync::{Arc, Mutex, OnceLock};
pub use indexmap::IndexMap;
pub use std::sync::LazyLock;
pub use std::borrow::Cow;
pub use futures::future::Ready;
pub use tokio::sync::{Semaphore, OwnedSemaphorePermit};
pub use futures::future::BoxFuture;
pub use futures::stream::BoxStream;
pub use indexmap::IndexSet;
pub use std::any::Any;

// Re-export commonly used crate modules
pub use crate::ops::{interface, execution};
pub use crate::execution::exec_ctx;
// concurrency control helpers live under services; expose them here for
// convenience.
pub use crate::services::concur_control;

// Bring common attribute macro and stream macro into prelude so modules that
// do `use crate::prelude::*;` can use `#[async_trait]` and `try_stream!`.
pub use async_trait::async_trait;
pub use async_stream::try_stream;

// Tracing macros
pub use tracing::{trace, debug, info, warn, error};

// Re-export minimal lib-context helpers (created as a stub).
pub use crate::lib_context::{get_lib_context, get_runtime, LibContext, LibContextRef};

// Re-export top-level crate modules so files using unqualified `spec::` /
// `schema::` / `value::` / `setup::` resolve correctly when they import
// the crate prelude.
pub use crate::spec;
pub use crate::schema;
pub use crate::value;
pub use crate::setup;

// Helper used across the crate for returning a standard "invariance violation"
// error where code expects a closure `FnOnce() -> Error` (used in `.ok_or_else`).
pub fn invariance_violation() -> anyhow::Error {
	anyhow!("invariance violation")
}