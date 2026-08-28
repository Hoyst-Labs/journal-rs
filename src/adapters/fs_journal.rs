use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{EntryMetadata, EntryName};
use crate::error::StoreError;
use crate::ports::JournalStore;

#[derive(Debug, Clone)]
pub struct FsJournalStore {
    current_dir: PathBuf,
}

impl FsJournalStore {
    pub fn new(current_dir: impl Into<PathBuf>) -> Self {
        Self {
            current_dir: current_dir.into(),
        }
    }

    fn journal_root(&self) -> Option<PathBuf> {
        [".journal", "journal"]
            .into_iter()
            .map(|name| self.current_dir.join(name))
            .find(|path| path.is_dir())
    }

    fn read_contained(&self, root: &Path, entry: &EntryName) -> Result<String, StoreError> {
        let candidate = root.join(entry.as_str());
        let canonical_root = fs::canonicalize(root).map_err(|source| StoreError::Read {
            entry: entry.clone(),
            path: root.to_path_buf(),
            source,
        })?;
        let canonical_candidate =
            fs::canonicalize(&candidate).map_err(|source| StoreError::Read {
                entry: entry.clone(),
                path: candidate.clone(),
                source,
            })?;

        if !canonical_candidate.starts_with(&canonical_root) {
            return Err(StoreError::EscapesRoot(entry.clone()));
        }

        fs::read_to_string(&canonical_candidate).map_err(|source| StoreError::Read {
            entry: entry.clone(),
            path: candidate,
            source,
        })
    }
}

impl JournalStore for FsJournalStore {
    fn list_entries(&self) -> Result<Vec<EntryMetadata>, StoreError> {
        let Some(root) = self.journal_root() else {
            return Ok(Vec::new());
        };
        let directory = fs::read_dir(&root).map_err(|source| StoreError::List {
            path: root.clone(),
            source,
        })?;

        let mut entries = directory
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                if !path.is_file() {
                    return None;
                }
                let name = path.file_name()?.to_str()?.to_string();
                EntryName::parse(name).ok().map(EntryMetadata::new)
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| right.name.cmp(&left.name));
        Ok(entries)
    }

    fn read_entry(&self, entry: &EntryName) -> Result<String, StoreError> {
        let root = self
            .journal_root()
            .ok_or(StoreError::MissingJournalDirectory)?;
        self.read_contained(&root, entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("journal-adapter-{name}-{unique}"));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn write(&self, relative: &str, content: impl AsRef<[u8]>) {
            let path = self.0.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, content).unwrap();
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn hidden_store_precedes_plain_store() {
        let temp = TempDir::new("precedence");
        temp.write(".journal/2026-05-01-hidden.md", "hidden");
        temp.write("journal/2026-05-02-plain.md", "plain");
        let entries = FsJournalStore::new(&temp.0).list_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name.as_str(), "2026-05-01-hidden.md");
    }

    #[test]
    fn falls_back_to_plain_store_and_never_searches_parents() {
        let temp = TempDir::new("fallback");
        temp.write("journal/2026-05-01-entry.md", "entry");
        let entries = FsJournalStore::new(&temp.0).list_entries().unwrap();
        assert_eq!(entries.len(), 1);

        fs::create_dir(temp.0.join("child")).unwrap();
        let child_entries = FsJournalStore::new(temp.0.join("child"))
            .list_entries()
            .unwrap();
        assert!(child_entries.is_empty());
    }

    #[test]
    fn filters_invalid_names_and_sorts_newest_first() {
        let temp = TempDir::new("listing");
        temp.write(".journal/notes.md", "invalid");
        temp.write(".journal/2026-05-01-entry.txt", "invalid");
        temp.write(".journal/2026-05-01-0900-first.md", "first");
        temp.write(".journal/2026-05-02-0900-second.md", "second");

        let entries = FsJournalStore::new(&temp.0).list_entries().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name.as_str(), "2026-05-02-0900-second.md");
        assert_eq!(entries[1].name.as_str(), "2026-05-01-0900-first.md");
    }

    #[test]
    fn reads_contained_entries_and_reports_utf8_errors() {
        let temp = TempDir::new("read");
        temp.write(".journal/2026-05-01-good.md", "content");
        temp.write(".journal/2026-05-02-bad.md", [0xff, 0xfe, 0xfd]);
        let store = FsJournalStore::new(&temp.0);

        assert_eq!(
            store
                .read_entry(&EntryName::parse("2026-05-01-good.md").unwrap())
                .unwrap(),
            "content"
        );
        assert!(matches!(
            store.read_entry(&EntryName::parse("2026-05-02-bad.md").unwrap()),
            Err(StoreError::Read { .. })
        ));
    }

    #[test]
    fn rejects_links_that_escape_the_journal_root_when_supported() {
        let temp = TempDir::new("escape");
        temp.write("outside.md", "outside");
        fs::create_dir(temp.0.join(".journal")).unwrap();
        let link = temp.0.join(".journal/2026-05-01-link.md");
        let target = temp.0.join("outside.md");

        #[cfg(unix)]
        let linked = std::os::unix::fs::symlink(&target, &link).is_ok();
        #[cfg(windows)]
        let linked = std::os::windows::fs::symlink_file(&target, &link).is_ok();

        if !linked {
            return;
        }

        let store = FsJournalStore::new(&temp.0);
        let result = store.read_entry(&EntryName::parse("2026-05-01-link.md").unwrap());
        assert!(matches!(result, Err(StoreError::EscapesRoot(_))));
    }
}
