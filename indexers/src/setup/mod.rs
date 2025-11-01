use crate::prelude::*;
use crate::execution::db_tracking_setup;

// Minimal stub of the `setup` module to satisfy type references across the
// indexers crate. This is intentionally light-weight: it exposes the types,
// fields and small helper methods used by other modules so the project can
// compile and we can iterate further. Behaviour is intentionally minimal.

#[derive(Clone, Debug)]
pub struct ResourceIdentifier {
    pub key: serde_json::Value,
    pub target_kind: String,
}

impl PartialEq for ResourceIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.target_kind == other.target_kind
            && serde_json::to_string(&self.key).ok() == serde_json::to_string(&other.key).ok()
    }
}
impl Eq for ResourceIdentifier {}
impl std::hash::Hash for ResourceIdentifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Ok(s) = serde_json::to_string(&self.key) {
            s.hash(state);
        }
        self.target_kind.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct SourceSetupState {
    pub source_id: i32,
    #[allow(dead_code)]
    pub keys_schema: Option<Box<[crate::base::schema::ValueType]>>,
    #[cfg(feature = "legacy-states-v0")]
    pub key_schema: Option<crate::base::schema::ValueType>,
    pub source_kind: String,
}

#[derive(Clone, Debug)]
pub struct TargetSetupStateCommon {
    pub target_id: i32,
    pub schema_version_id: usize,
    pub max_schema_version_id: usize,
    pub setup_by_user: bool,
    pub key_type: Option<Box<[crate::base::schema::ValueType]>>,
}

#[derive(Clone, Debug)]
pub struct TargetSetupState {
    pub common: TargetSetupStateCommon,
    pub state: serde_json::Value,
    pub attachments: IndexMap<String, serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct FlowSetupMetadata {
    pub last_source_id: i32,
    pub last_target_id: i32,
    pub sources: std::collections::BTreeMap<String, SourceSetupState>,
    pub features: std::collections::BTreeSet<String>,
}

#[derive(Clone, Debug)]
pub struct FlowSetupState<Mode> {
    pub seen_flow_metadata_version: Option<usize>,
    pub tracking_table: db_tracking_setup::TrackingTableSetupState,
    pub targets: IndexMap<ResourceIdentifier, TargetSetupState>,
    pub metadata: CombinedState<FlowSetupMetadata>,
    // Mode parameter is only for typing compatibility with existing code
    #[allow(dead_code)]
    _mode: std::marker::PhantomData<Mode>,
}

// Mode marker types used as generic parameters in various places.
#[derive(Clone, Debug)]
pub struct DesiredMode;

#[derive(Clone, Debug)]
pub struct ExistingMode;

#[derive(Clone, Debug)]
pub enum ChangeDescription {
    Action(String),
    Note(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetupChangeType {
    NoChange,
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug)]
pub enum FlowSetupChangeAction {
    Setup,
    Teardown,
    Validate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StateChange<T> {
    Upsert(T),
    Delete,
}

pub trait ResourceSetupChange: Send + Sync {
    fn describe_changes(&self) -> Vec<ChangeDescription>;
    fn change_type(&self) -> SetupChangeType;
}

#[derive(Clone, Debug)]
pub struct CombinedState<T> {
    pub versions: Vec<T>,
}

impl<T> CombinedState<T> {
    pub fn possible_versions(&self) -> impl Iterator<Item = &T> {
        self.versions.iter()
    }
}

pub mod flow_features {
    use std::collections::BTreeSet;

    pub const SOURCE_STATE_TABLE: &str = "source_state_table";
    pub const FAST_FINGERPRINT: &str = "fast_fingerprint";

    pub fn default_features() -> BTreeSet<String> {
        BTreeSet::new()
    }
}

// Small helper used by some callers elsewhere; minimal implementation.
pub async fn apply_changes_for_flow_ctx(
    _action: FlowSetupChangeAction,
    _flow_ctx: &crate::lib_context::FlowContext,
    _flow_exec_ctx: &mut crate::execution::FlowExecutionContext,
    _lib_setup_ctx: &mut crate::lib_context::LibSetupContext,
    _db_pool: &tokio_postgres::Pool<tokio_postgres::NoTls>,
    _output_buffer: &mut Vec<u8>,
) -> Result<()> {
    // Stub implementation - to be implemented later
    Ok(())
}
