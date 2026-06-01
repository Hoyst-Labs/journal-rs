use std::collections::BTreeMap;

use crate::journal::NO_ENTRIES_MESSAGE;
use crate::search::SearchResult;

pub fn format_grouped_files(grouped: &BTreeMap<String, Vec<String>>) -> String {
    if grouped.is_empty() {
        return NO_ENTRIES_MESSAGE.to_string();
    }

    grouped
        .iter()
        .rev()
        .map(|(date, files)| {
            let file_list = files
                .iter()
                .map(|file| format!(" - {file}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!("{date}\n{file_list}")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn format_file_blocks(blocks: &[(String, String)]) -> String {
    if blocks.is_empty() {
        return NO_ENTRIES_MESSAGE.to_string();
    }

    blocks
        .iter()
        .map(|(file, body)| format!("{file}\n{body}"))
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn format_search_results(results: &[SearchResult]) -> String {
    if results.is_empty() {
        return "No matching entries found.".to_string();
    }

    results
        .iter()
        .map(|result| {
            let preview = truncate_summary(&result.summary, 120);
            format!("[{:.2}] {}\n  {}", result.final_score, result.filename, preview)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn truncate_summary(summary: &str, max_len: usize) -> String {
    let one_line = summary.lines().collect::<Vec<_>>().join(" ");
    if one_line.len() <= max_len {
        one_line
    } else {
        format!("{}...", &one_line[..max_len])
    }
}
