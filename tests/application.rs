use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use journal::{
    Application, Clock, DateWindow, EntryBody, EntryMetadata, EntryName, EntrySelection,
    JournalMoment, JournalStore, Outcome, QueryRequest, SearchQuery, SectionName, StoreError, View,
};

struct FakeJournalStore {
    entries: Vec<EntryMetadata>,
    content: BTreeMap<EntryName, Result<String, String>>,
    reads: RefCell<BTreeMap<EntryName, usize>>,
}

impl FakeJournalStore {
    fn new(values: &[(&str, Result<&str, &str>)]) -> Self {
        let mut entries = Vec::new();
        let mut content = BTreeMap::new();
        for (name, body) in values {
            let name = EntryName::parse(*name).unwrap();
            entries.push(EntryMetadata::new(name.clone()));
            content.insert(name, body.map(str::to_string).map_err(str::to_string));
        }
        entries.sort_by(|left, right| right.name.cmp(&left.name));
        Self {
            entries,
            content,
            reads: RefCell::new(BTreeMap::new()),
        }
    }

    fn read_count(&self, name: &str) -> usize {
        let name = EntryName::parse(name).unwrap();
        self.reads.borrow().get(&name).copied().unwrap_or(0)
    }
}

impl JournalStore for &FakeJournalStore {
    fn list_entries(&self) -> Result<Vec<EntryMetadata>, StoreError> {
        Ok(self.entries.clone())
    }

    fn read_entry(&self, entry: &EntryName) -> Result<String, StoreError> {
        *self.reads.borrow_mut().entry(entry.clone()).or_default() += 1;
        match self.content.get(entry) {
            Some(Ok(content)) => Ok(content.clone()),
            Some(Err(message)) => Err(StoreError::Read {
                entry: entry.clone(),
                path: PathBuf::from(entry.as_str()),
                source: io::Error::other(message.clone()),
            }),
            None => Err(StoreError::Read {
                entry: entry.clone(),
                path: PathBuf::from(entry.as_str()),
                source: io::Error::new(io::ErrorKind::NotFound, "missing fake entry"),
            }),
        }
    }
}

#[derive(Clone, Copy)]
struct FixedClock(i64);

impl Clock for FixedClock {
    fn now_unix(&self) -> i64 {
        self.0
    }
}

fn request(view: View) -> QueryRequest {
    QueryRequest {
        selection: EntrySelection::default(),
        view,
        view_explicit: true,
        limit: None,
    }
}

#[test]
fn combined_filters_run_before_content_and_cached_content_is_reused() {
    let store = FakeJournalStore::new(&[
        (
            "2026-04-25-0900-old.md",
            Ok("## Next Actions\n\n- deploy old\n"),
        ),
        (
            "2026-04-27-1100-release.md",
            Ok("## Summary\n\nRelease\n\n## Next Actions\n\n- deploy release\n"),
        ),
        (
            "2026-04-28-1200-other.md",
            Ok("## Next Actions\n\n- investigate\n"),
        ),
    ]);
    let app = Application::new(&store, FixedClock(1_780_000_000));
    let outcome = app
        .execute(QueryRequest {
            selection: EntrySelection {
                file_prefix: Some("2026-04".to_string()),
                window: Some(DateWindow::Since(
                    JournalMoment::parse("2026-04-26").unwrap(),
                )),
                content_terms: vec!["deploy".to_string()],
            },
            view: View::Section(SectionName::new("Next Actions").unwrap()),
            view_explicit: true,
            limit: None,
        })
        .unwrap();

    let Outcome::Blocks(blocks) = outcome else {
        panic!("expected blocks");
    };
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].entry.name.as_str(), "2026-04-27-1100-release.md");
    assert_eq!(
        blocks[0].body,
        EntryBody::Content("- deploy release".to_string())
    );
    assert_eq!(store.read_count("2026-04-25-0900-old.md"), 0);
    assert_eq!(store.read_count("2026-04-27-1100-release.md"), 1);
    assert_eq!(store.read_count("2026-04-28-1200-other.md"), 1);
}

#[test]
fn latest_without_explicit_view_returns_summary_blocks() {
    let store = FakeJournalStore::new(&[
        ("2026-05-01-0900-first.md", Ok("## Summary\n\nFirst\n")),
        ("2026-05-02-0900-second.md", Ok("## Summary\n\nSecond\n")),
        ("2026-05-03-0900-third.md", Ok("## Summary\n\nThird\n")),
    ]);
    let app = Application::new(&store, FixedClock(1_780_000_000));
    let outcome = app
        .execute(QueryRequest {
            selection: EntrySelection::default(),
            view: View::List,
            view_explicit: false,
            limit: NonZeroUsize::new(2),
        })
        .unwrap();

    let Outcome::Blocks(blocks) = outcome else {
        panic!("expected blocks");
    };
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].body, EntryBody::Content("Third".to_string()));
    assert_eq!(blocks[1].body, EntryBody::Content("Second".to_string()));
}

#[test]
fn search_uses_fixed_clock_default_limit_and_skips_unusable_entries() {
    let mut values = (1..=12)
        .map(|index| {
            let name = format!("2026-05-{index:02}-1000-entry{index}.md");
            let body = format!("## Summary\n\nDeploy production iteration {index}.\n");
            (name, body)
        })
        .collect::<Vec<_>>();
    values.push((
        "2026-05-13-1000-no-summary.md".to_string(),
        "## Context\n\nDeploy context only.\n".to_string(),
    ));
    let borrowed = values
        .iter()
        .map(|(name, body)| (name.as_str(), Ok(body.as_str())))
        .collect::<Vec<_>>();
    let store = FakeJournalStore::new(&borrowed);
    let app = Application::new(&store, FixedClock(1_780_000_000));
    let outcome = app
        .execute(request(View::Search(
            SearchQuery::new("old deploy production").unwrap(),
        )))
        .unwrap();

    let Outcome::Search(results) = outcome else {
        panic!("expected search results");
    };
    assert_eq!(results.len(), 10);
    assert_eq!(results[0].entry.name.as_str(), "2026-05-12-1000-entry12.md");
    assert!(
        results
            .iter()
            .all(|result| result.entry.name.as_str() != "2026-05-13-1000-no-summary.md")
    );
}

#[test]
fn read_errors_continue_for_non_search_and_are_skipped_for_search() {
    let store = FakeJournalStore::new(&[
        ("2026-05-02-1000-good.md", Ok("## Summary\n\nDeploy good\n")),
        ("2026-05-01-1000-bad.md", Err("bad utf-8")),
    ]);
    let app = Application::new(&store, FixedClock(1_780_000_000));

    let full = app.execute(request(View::Full)).unwrap();
    let Outcome::Blocks(blocks) = full else {
        panic!("expected full blocks");
    };
    assert_eq!(blocks.len(), 2);
    assert!(matches!(blocks[1].body, EntryBody::ReadError(_)));

    let search = app
        .execute(request(View::Search(
            SearchQuery::new("old deploy").unwrap(),
        )))
        .unwrap();
    let Outcome::Search(results) = search else {
        panic!("expected search results");
    };
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.name.as_str(), "2026-05-02-1000-good.md");
    assert_eq!(store.read_count("2026-05-01-1000-bad.md"), 2);
}

#[test]
fn empty_store_returns_structured_empty_outcome() {
    let store = FakeJournalStore::new(&[]);
    let app = Application::new(&store, FixedClock(1_780_000_000));
    assert_eq!(
        app.execute(request(View::List)).unwrap(),
        Outcome::EntriesNotFound
    );
}
