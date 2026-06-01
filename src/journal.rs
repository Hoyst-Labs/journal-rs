use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const NO_ENTRIES_MESSAGE: &str = "No journal entries found.";

pub fn discover_journal_dir_from(current_dir: &Path) -> Option<PathBuf> {
    [".journal", "journal"]
        .into_iter()
        .map(|name| current_dir.join(name))
        .find(|path| path.is_dir())
}

pub fn list_journal_files(journal_dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(journal_dir) else {
        return Vec::new();
    };

    let mut files: Vec<String> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }

            let name = path.file_name()?.to_str()?.to_string();
            if !name.ends_with(".md") || extract_date_prefix(&name).is_none() {
                return None;
            }

            Some(name)
        })
        .collect();

    files.sort_by(|left, right| right.cmp(left));
    files
}

pub fn group_files_by_date(files: &[String]) -> BTreeMap<String, Vec<String>> {
    let mut grouped = BTreeMap::new();
    for file in files {
        if let Some(date) = extract_date_prefix(file) {
            grouped
                .entry(date.to_string())
                .or_insert_with(Vec::new)
                .push(file.clone());
        }
    }
    grouped
}

pub fn read_file(journal_dir: &Path, file_name: &str) -> Result<String, String> {
    let path = journal_dir.join(file_name);
    fs::read_to_string(&path).map_err(|error| format!("Failed to read {}: {error}", path.display()))
}

pub fn extract_date_prefix(value: &str) -> Option<&str> {
    let prefix = value.get(0..10)?;
    if is_valid_date(prefix) {
        Some(prefix)
    } else {
        None
    }
}

pub fn extract_timestamp_prefix(value: &str) -> Option<&str> {
    let prefix = value.get(0..15)?;
    if is_valid_date_time(prefix) {
        Some(prefix)
    } else {
        None
    }
}

pub fn is_valid_date_or_time(value: &str) -> bool {
    is_valid_date(value) || is_valid_date_time(value)
}

pub fn is_valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 {
        return false;
    }

    if bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }

    bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

pub fn is_valid_date_time(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 15 {
        return false;
    }

    if bytes[4] != b'-' || bytes[7] != b'-' || bytes[10] != b'-' {
        return false;
    }

    bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| matches!(index, 4 | 7 | 10) || byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn validates_date_and_datetime_inputs() {
        assert!(is_valid_date("2026-05-01"));
        assert!(is_valid_date_time("2026-05-01-0930"));
        assert!(is_valid_date_or_time("2026-05-01"));
        assert!(is_valid_date_or_time("2026-05-01-0930"));
        assert!(!is_valid_date_or_time("2026/05/01"));
    }

    #[test]
    fn extracts_prefixes_from_file_names() {
        let file = "2026-05-01-0930-entry.md";
        assert_eq!(extract_date_prefix(file), Some("2026-05-01"));
        assert_eq!(extract_timestamp_prefix(file), Some("2026-05-01-0930"));
    }

    #[test]
    fn discovers_hidden_journal_before_plain_journal() {
        let temp_dir = create_temp_dir("discover-hidden");
        fs::create_dir(temp_dir.join(".journal")).unwrap();
        fs::create_dir(temp_dir.join("journal")).unwrap();

        let discovered = discover_journal_dir_from(&temp_dir);

        assert_eq!(discovered, Some(temp_dir.join(".journal")));
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn returns_none_when_no_journal_directory_exists() {
        let temp_dir = create_temp_dir("discover-missing");

        let discovered = discover_journal_dir_from(&temp_dir);

        assert_eq!(discovered, None);
        fs::remove_dir_all(temp_dir).unwrap();
    }

    fn create_temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("journal-tests-{prefix}-{unique}"));
        fs::create_dir(&path).unwrap();
        path
    }
}
