use std::collections::BTreeMap;

use crate::journal::NO_ENTRIES_MESSAGE;

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
