use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

/// A wrapper type for Python-compatible values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pythonized<T>(pub T);

impl<T> Pythonized<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
    
    pub fn inner(&self) -> &T {
        &self.0
    }
    
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Pythonized<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Pythonized<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Python execution context for running Python code
pub struct PythonExecutionContext {
    pub python: Python<'static>,
    pub event_loop: Option<Py<PyAny>>,
    pub globals: Py<PyDict>,
}

impl PythonExecutionContext {
    /// Create a new Python execution context
    pub fn new(python: Python<'static>, event_loop: Option<Py<PyAny>>) -> PyResult<Self> {
        let globals = PyDict::new(python).into();
        
        Ok(Self {
            python,
            event_loop,
            globals,
        })
    }
    
    /// Execute Python code in this context
    pub fn execute(&self, code: &str) -> PyResult<Py<PyAny>> {
        self.python.run(code, Some(&self.globals), None)?;
        Ok(self.python.None())
    }
    
    /// Evaluate a Python expression and return the result
    pub fn eval(&self, expression: &str) -> PyResult<Py<PyAny>> {
        self.python.eval(expression, Some(&self.globals), None)
            .map(|obj| obj.into())
    }
    
    /// Get a reference to the Python interpreter
    pub fn python(&self) -> Python<'static> {
        self.python
    }
    
    /// Get the event loop if available
    pub fn event_loop(&self) -> Option<&Py<PyAny>> {
        self.event_loop.as_ref()
    }
    
    /// Get the global namespace
    pub fn globals(&self) -> &Py<PyDict> {
        &self.globals
    }
}

unsafe impl Send for PythonExecutionContext {}
unsafe impl Sync for PythonExecutionContext {}