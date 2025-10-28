use anyhow::Result;

/// Trait for converting anyhow results into Python-compatible results
pub trait AnyhowIntoPyResult<T> {
    fn into_py_result(self) -> Result<T>;
}

impl<T> AnyhowIntoPyResult<T> for Result<T> {
    fn into_py_result(self) -> Result<T> {
        self
    }
}

impl<T, E> AnyhowIntoPyResult<T> for std::result::Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn into_py_result(self) -> Result<T> {
        self.map_err(|e| e.into())
    }
}