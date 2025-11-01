pub(crate) mod db_tracking_setup;
pub(crate) mod dumper;
pub(crate) mod evaluator;
pub(crate) mod indexing_status;
pub(crate) mod memoization;
pub(crate) mod row_indexer;
pub(crate) mod source_indexer;
pub(crate) mod stats;

mod live_updater;
pub(crate) use live_updater::*;

mod db_tracking;

// Stub implementation for FlowExecutionContext
#[derive(Debug, Default)]
pub struct FlowExecutionContext {
    // Placeholder fields - to be implemented as needed
    pub setup_change: FlowSetupChange,
}

impl FlowExecutionContext {
    pub fn new() -> Self {
        Self::default()
    }
}

// Stub implementation for FlowSetupChange
#[derive(Debug, Default)]
pub struct FlowSetupChange {
    // Placeholder fields
}

impl FlowSetupChange {
    pub fn has_internal_changes(&self) -> bool {
        false // Stub implementation
    }
    
    pub fn has_external_changes(&self) -> bool {
        false // Stub implementation
    }
}
