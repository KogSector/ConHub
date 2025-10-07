use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryContextProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContextProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlContextProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceContextProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchToolProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisToolProvider;