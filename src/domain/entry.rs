use std::fmt;

use super::filename::{extract_date_prefix, extract_timestamp_prefix};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryName(String);

impl EntryName {
    /// Parses and validates a journal entry filename.
    ///
    /// ```
    /// use journal::EntryName;
    ///
    /// let entry = EntryName::parse("2026-05-01-0930-release.md")?;
    /// assert_eq!(entry.date_prefix(), "2026-05-01");
    /// assert_eq!(entry.timestamp_prefix(), Some("2026-05-01-0930"));
    ///
    /// # Ok::<(), journal::EntryNameError>(())
    /// ```
    pub fn parse(value: impl Into<String>) -> Result<Self, EntryNameError> {
        let value = value.into();
        if value.is_empty()
            || value == "."
            || value == ".."
            || value.contains('/')
            || value.contains('\\')
            || value.contains(':')
        {
            return Err(EntryNameError::UnsafePath(value));
        }

        if !value.ends_with(".md") || extract_date_prefix(&value).is_none() {
            return Err(EntryNameError::InvalidFormat(value));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn date_prefix(&self) -> &str {
        extract_date_prefix(&self.0).unwrap_or_default()
    }

    pub fn timestamp_prefix(&self) -> Option<&str> {
        extract_timestamp_prefix(&self.0)
    }
}

impl fmt::Display for EntryName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryNameError {
    UnsafePath(String),
    InvalidFormat(String),
}

impl fmt::Display for EntryNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsafePath(value) => write!(formatter, "Unsafe journal entry name: {value}"),
            Self::InvalidFormat(value) => write!(formatter, "Invalid journal entry name: {value}"),
        }
    }
}

impl std::error::Error for EntryNameError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryMetadata {
    pub name: EntryName,
}

impl EntryMetadata {
    pub fn new(name: EntryName) -> Self {
        Self { name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_journal_entry_names() {
        let name = EntryName::parse("2026-05-01-0930-entry.md").unwrap();
        assert_eq!(name.date_prefix(), "2026-05-01");
        assert_eq!(name.timestamp_prefix(), Some("2026-05-01-0930"));
    }

    #[test]
    fn rejects_paths_and_invalid_names() {
        assert!(EntryName::parse("../2026-05-01-entry.md").is_err());
        assert!(EntryName::parse("folder/2026-05-01-entry.md").is_err());
        assert!(EntryName::parse("folder\\2026-05-01-entry.md").is_err());
        assert!(EntryName::parse("C:2026-05-01-entry.md").is_err());
        assert!(EntryName::parse("notes.md").is_err());
        assert!(EntryName::parse("2026-05-01-entry.txt").is_err());
    }
}
