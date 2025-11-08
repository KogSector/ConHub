use serde::Serialize;
use std::io::Write;

/// A YAML serializer for writing data in YAML format
#[derive(Debug)]
pub struct YamlSerializer<W: Write> {
    writer: W,
}

impl<W: Write> YamlSerializer<W> {
    /// Create a new YAML serializer with the given writer
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Serialize a value to YAML and write it to the underlying writer
    pub fn serialize<T: Serialize>(&mut self, value: &T) -> anyhow::Result<()> {
        let yaml_string = serde_yaml::to_string(value)?;
        self.writer.write_all(yaml_string.as_bytes())?;
        Ok(())
    }

    /// Serialize a value to YAML with a document separator
    pub fn serialize_document<T: Serialize>(&mut self, value: &T) -> anyhow::Result<()> {
        self.writer.write_all(b"---\n")?;
        self.serialize(value)?;
        Ok(())
    }

    /// Write raw YAML content
    pub fn write_raw(&mut self, content: &str) -> Result<(), std::io::Error> {
        self.writer.write_all(content.as_bytes())
    }

    /// Flush the underlying writer
    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.writer.flush()
    }

    /// Get a reference to the underlying writer
    pub fn get_ref(&self) -> &W {
        &self.writer
    }

    /// Get a mutable reference to the underlying writer
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Consume the serializer and return the underlying writer
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl YamlSerializer<Vec<u8>> {
    /// Create a new YAML serializer that writes to a Vec<u8>
    pub fn new_vec() -> Self {
        Self::new(Vec::new())
    }

    /// Serialize a value to a YAML string
    pub fn to_string<T: Serialize>(value: &T) -> anyhow::Result<String> {
        let mut serializer = Self::new_vec();
        serializer.serialize(value)?;
        Ok(String::from_utf8(serializer.into_inner())?)
    }
}