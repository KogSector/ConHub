use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A fingerprint representing a unique identifier for data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fingerprint(String);

impl Fingerprint {
    /// Create a new fingerprint from a string
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Get the fingerprint value as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create a fingerprint from bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let result = hasher.finalize();
        Self(hex::encode(result))
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Fingerprint {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Fingerprint {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// A utility for generating fingerprints from various data types
#[derive(Debug, Default)]
pub struct Fingerprinter {
    hasher: Sha256,
}

impl Fingerprinter {
    /// Create a new fingerprinter
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    /// Add bytes to the fingerprint calculation
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Add a string to the fingerprint calculation
    pub fn update_str(&mut self, data: &str) {
        self.hasher.update(data.as_bytes());
    }

    /// Add serializable data to the fingerprint calculation
    pub fn update_json<T: Serialize>(&mut self, data: &T) -> Result<(), serde_json::Error> {
        let json = serde_json::to_string(data)?;
        self.update_str(&json);
        Ok(())
    }

    /// Finalize the fingerprint calculation and return the result
    pub fn finalize(self) -> Fingerprint {
        let result = self.hasher.finalize();
        Fingerprint(hex::encode(result))
    }

    /// Create a fingerprint from a single piece of data
    pub fn fingerprint_bytes(data: &[u8]) -> Fingerprint {
        let mut fingerprinter = Self::new();
        fingerprinter.update(data);
        fingerprinter.finalize()
    }

    /// Create a fingerprint from a string
    pub fn fingerprint_str(data: &str) -> Fingerprint {
        let mut fingerprinter = Self::new();
        fingerprinter.update_str(data);
        fingerprinter.finalize()
    }

    /// Create a fingerprint from serializable data
    pub fn fingerprint_json<T: Serialize>(data: &T) -> Result<Fingerprint, serde_json::Error> {
        let mut fingerprinter = Self::new();
        fingerprinter.update_json(data)?;
        Ok(fingerprinter.finalize())
    }

    /// Add serializable data to the fingerprint and return self for chaining
    pub fn with<T: Serialize>(mut self, data: &T) -> Result<Self, serde_json::Error> {
        self.update_json(data)?;
        Ok(self)
    }
}