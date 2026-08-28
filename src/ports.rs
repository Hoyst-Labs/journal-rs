use crate::domain::{EntryMetadata, EntryName};
use crate::error::StoreError;

pub trait JournalStore {
    fn list_entries(&self) -> Result<Vec<EntryMetadata>, StoreError>;
    fn read_entry(&self, entry: &EntryName) -> Result<String, StoreError>;
}

pub trait Clock {
    fn now_unix(&self) -> i64;
}
