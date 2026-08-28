use std::collections::BTreeMap;

use crate::app::{EntryBlock, EntryBody, Outcome};
use crate::domain::{EntryMetadata, SearchMatch};

pub const NO_ENTRIES_MESSAGE: &str = "No journal entries found.";
pub const NO_MATCHES_MESSAGE: &str = "No matching entries found.";

pub fn render(outcome: &Outcome) -> String {
    match outcome {
        Outcome::EntriesNotFound => NO_ENTRIES_MESSAGE.to_string(),
        Outcome::Listing {
            entries,
            read_errors,
        } => render_listing(entries, read_errors),
        Outcome::Blocks(blocks) => render_blocks(blocks),
        Outcome::Search(results) => render_search(results),
    }
}

fn render_listing(entries: &[EntryMetadata], read_errors: &[EntryBlock]) -> String {
    if entries.is_empty() {
        return NO_ENTRIES_MESSAGE.to_string();
    }

    let mut grouped: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for entry in entries {
        grouped
            .entry(entry.name.date_prefix())
            .or_default()
            .push(entry.name.as_str());
    }
    let listing = grouped
        .iter()
        .rev()
        .map(|(date, files)| {
            let files = files
                .iter()
                .map(|file| format!(" - {file}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!("{date}\n{files}")
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    if read_errors.is_empty() {
        listing
    } else {
        format!("{listing}\n\n{}", render_blocks(read_errors))
    }
}

fn render_blocks(blocks: &[EntryBlock]) -> String {
    if blocks.is_empty() {
        return NO_ENTRIES_MESSAGE.to_string();
    }

    blocks
        .iter()
        .map(|block| {
            let body = match &block.body {
                EntryBody::Content(content) => content.trim_end().to_string(),
                EntryBody::MissingSection(heading) => {
                    format!("[No ## {} section found]", heading.as_str())
                }
                EntryBody::ReadError(error) => error.clone(),
            };
            format!("{}\n{body}", block.entry.name)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_search(results: &[SearchMatch]) -> String {
    if results.is_empty() {
        return NO_MATCHES_MESSAGE.to_string();
    }

    results
        .iter()
        .map(|result| {
            let preview = truncate_summary(&result.summary, 120);
            format!(
                "[{:.2}] {}\n  {preview}",
                result.final_score, result.entry.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn truncate_summary(summary: &str, max_length: usize) -> String {
    let one_line = summary.lines().collect::<Vec<_>>().join(" ");
    if one_line.chars().count() <= max_length {
        one_line
    } else {
        format!(
            "{}...",
            one_line.chars().take(max_length).collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::EntryBody;
    use crate::domain::EntryName;

    fn entry(name: &str) -> EntryMetadata {
        EntryMetadata::new(EntryName::parse(name).unwrap())
    }

    #[test]
    fn renders_grouped_entries_and_blocks() {
        let outcome = Outcome::Listing {
            entries: vec![
                entry("2026-05-02-1000-second.md"),
                entry("2026-05-01-1000-first.md"),
            ],
            read_errors: Vec::new(),
        };
        assert_eq!(
            render(&outcome),
            concat!(
                "2026-05-02\n - 2026-05-02-1000-second.md\n\n",
                "2026-05-01\n - 2026-05-01-1000-first.md"
            )
        );

        let blocks = Outcome::Blocks(vec![EntryBlock {
            entry: entry("2026-05-01-1000-first.md"),
            body: EntryBody::Content("value\n".to_string()),
        }]);
        assert_eq!(render(&blocks), "2026-05-01-1000-first.md\nvalue");
    }

    #[test]
    fn truncates_unicode_without_splitting_code_points() {
        let summary = "é".repeat(121);
        let truncated = truncate_summary(&summary, 120);
        assert_eq!(truncated.chars().count(), 123);
        assert!(truncated.ends_with("..."));
    }
}
