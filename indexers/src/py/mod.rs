use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::lib_context::FlowContext;
use crate::builder::AnalyzedTransientFlow;

// Re-export the AnyhowIntoPyResult trait for Python integration
pub use crate::utils::errors::AnyhowIntoPyResult;

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

// Conversion functions for Python interop
use crate::base::{value, schema};

/// Convert field values to a Python object
pub fn field_values_to_py_object<'py, I>(
    py: Python<'py>,
    field_values: I,
) -> PyResult<Bound<'py, PyDict>>
where
    I: Iterator<Item = &'py value::Value>,
{
    let dict = PyDict::new(py);
    for (idx, field_value) in field_values.enumerate() {
        let py_value = value_to_py_object(py, field_value)?;
        dict.set_item(idx.to_string(), py_value)?;
    }
    Ok(dict)
}

/// Convert field values from a Python sequence
pub fn field_values_from_py_seq(
    value_types: &[schema::ValueType],
    py_seq: &Bound<'_, PyAny>,
) -> PyResult<Vec<value::Value>> {
    let mut values = Vec::new();
    
    // Try to iterate over the sequence
    if let Ok(iter) = py_seq.iter() {
        for (idx, item) in iter.enumerate() {
            let item = item?;
            let value_type = value_types.get(idx).unwrap_or(&schema::ValueType::Basic(schema::BasicValueType::String));
            let value = value_from_py_object(value_type, &item)?;
            values.push(value);
        }
    }
    
    Ok(values)
}

/// Convert a key to a Python object
pub fn key_to_py_object(py: Python<'_>, key: &value::KeyValue) -> PyResult<Py<PyAny>> {
    // Convert key components to Python list
    let py_list = PyList::new(py, key.components.iter().map(|v| value_to_py_object(py, v)).collect::<PyResult<Vec<_>>>()?)?;
    Ok(py_list.into())
}

/// Convert a Python object to a Rust value
pub fn value_from_py_object(
    value_type: &schema::ValueType,
    py_obj: &Bound<'_, PyAny>,
) -> PyResult<value::Value> {
    // Stub implementation - convert based on value type
    match value_type {
        schema::ValueType::Basic(basic_type) => {
            match basic_type {
                schema::BasicValueType::String => {
                    if let Ok(s) = py_obj.extract::<String>() {
                        Ok(value::Value::String(s))
                    } else {
                        Ok(value::Value::Null)
                    }
                }
                schema::BasicValueType::Integer => {
                    if let Ok(i) = py_obj.extract::<i64>() {
                        Ok(value::Value::Integer(i))
                    } else {
                        Ok(value::Value::Null)
                    }
                }
                schema::BasicValueType::Float => {
                    if let Ok(f) = py_obj.extract::<f64>() {
                        Ok(value::Value::Float(f))
                    } else {
                        Ok(value::Value::Null)
                    }
                }
                schema::BasicValueType::Boolean => {
                    if let Ok(b) = py_obj.extract::<bool>() {
                        Ok(value::Value::Boolean(b))
                    } else {
                        Ok(value::Value::Null)
                    }
                }
                _ => Ok(value::Value::Null), // Stub for other types
            }
        }
        _ => Ok(value::Value::Null), // Stub for complex types
    }
}

/// Convert a Rust value to a Python object
pub fn value_to_py_object(py: Python<'_>, value: &value::Value) -> PyResult<Py<PyAny>> {
    match value {
        value::Value::Null => Ok(py.None()),
        value::Value::String(s) => Ok(s.into_py_any(py)?),
        value::Value::Integer(i) => Ok(i.into_py_any(py)?),
        value::Value::Float(f) => Ok(f.into_py_any(py)?),
        value::Value::Boolean(b) => Ok(b.into_py_any(py)?),
        _ => Ok(py.None()), // Stub for other types
    }
}

/// Trait for converting Python results with trace information
pub trait ToResultWithPyTrace<T> {
    fn to_result_with_py_trace(self, py: Python<'_>) -> PyResult<T>;
}

impl<T> ToResultWithPyTrace<T> for PyResult<T> {
    fn to_result_with_py_trace(self, _py: Python<'_>) -> PyResult<T> {
        self // Simple passthrough for now
    }
}

/// Python wrapper for FlowContext
#[pyclass]
#[derive(Debug, Clone)]
pub struct Flow(pub Arc<FlowContext>);

#[pymethods]
impl Flow {
    pub fn __str__(&self) -> String {
        format!("Flow({})", self.0.analyzed_flow.flow_instance.name)
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }
}

/// Python wrapper for AnalyzedTransientFlow
#[pyclass]
#[derive(Debug, Clone)]
pub struct TransientFlow(pub Arc<AnalyzedTransientFlow>);

#[pymethods]
impl TransientFlow {
    pub fn __str__(&self) -> String {
        format!("TransientFlow({})", self.0.transient_flow_instance.name)
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }
}