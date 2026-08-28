use std::num::NonZeroUsize;

use super::{EntryMetadata, JournalMoment};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Query(QueryRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRequest {
    pub selection: EntrySelection,
    pub view: View,
    pub view_explicit: bool,
    pub limit: Option<NonZeroUsize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EntrySelection {
    pub file_prefix: Option<String>,
    pub window: Option<DateWindow>,
    pub content_terms: Vec<String>,
}

impl EntrySelection {
    pub fn matches_metadata(&self, entry: &EntryMetadata) -> bool {
        let prefix_matches = self
            .file_prefix
            .as_ref()
            .is_none_or(|prefix| entry.name.as_str().starts_with(prefix));
        let window_matches = self
            .window
            .as_ref()
            .is_none_or(|window| window.matches(&entry.name));
        prefix_matches && window_matches
    }

    pub fn matches_content(&self, content: &str) -> bool {
        if self.content_terms.is_empty() {
            return true;
        }

        let lowered = content.to_ascii_lowercase();
        self.content_terms.iter().any(|term| lowered.contains(term))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateWindow {
    Since(JournalMoment),
    Between {
        start: JournalMoment,
        end: JournalMoment,
    },
}

impl DateWindow {
    pub fn between(start: JournalMoment, end: JournalMoment) -> Option<Self> {
        (start.normalized_start() <= end.normalized_end()).then_some(Self::Between { start, end })
    }

    pub fn matches(&self, entry: &super::EntryName) -> bool {
        match self {
            Self::Since(since) => since.matches_since(entry),
            Self::Between { start, end } => {
                start.contains_as_start(entry) && end.contains_as_end(entry)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    List,
    Full,
    Section(SectionName),
    Search(SearchQuery),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionName(String);

impl SectionName {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| Self(trimmed.to_string()))
    }

    pub fn summary() -> Self {
        Self("Summary".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery(String);

impl SearchQuery {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (!value.trim().is_empty()).then_some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EntryName;

    fn metadata(value: &str) -> EntryMetadata {
        EntryMetadata::new(EntryName::parse(value).unwrap())
    }

    #[test]
    fn selection_combines_prefix_and_window() {
        let selection = EntrySelection {
            file_prefix: Some("2026-04".to_string()),
            window: Some(DateWindow::Since(
                JournalMoment::parse("2026-04-04-1500").unwrap(),
            )),
            content_terms: Vec::new(),
        };

        assert!(selection.matches_metadata(&metadata("2026-04-04-1525-entry.md")));
        assert!(!selection.matches_metadata(&metadata("2026-04-04-1400-entry.md")));
        assert!(!selection.matches_metadata(&metadata("2026-05-04-1525-entry.md")));
    }

    #[test]
    fn content_terms_are_case_insensitive_and_any_match() {
        let selection = EntrySelection {
            content_terms: vec!["auth".to_string(), "deploy".to_string()],
            ..EntrySelection::default()
        };
        assert!(selection.matches_content("We need DEPLOY notes"));
        assert!(selection.matches_content("Auth token issue"));
        assert!(!selection.matches_content("No related keywords"));
    }

    #[test]
    fn rejects_invalid_ranges_and_empty_names() {
        let start = JournalMoment::parse("2026-05-02").unwrap();
        let end = JournalMoment::parse("2026-05-01").unwrap();
        assert!(DateWindow::between(start, end).is_none());
        assert!(SectionName::new("  ").is_none());
        assert!(SearchQuery::new("").is_none());
    }
}
