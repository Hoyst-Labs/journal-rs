use std::collections::BTreeMap;

use crate::domain::search::{SearchCandidate, rank};
use crate::domain::{
    EntryMetadata, EntryName, QueryRequest, SearchMatch, SectionName, View, extract_section,
};
use crate::error::{AppError, AppResult};
use crate::ports::{Clock, JournalStore};

pub struct Application<S, C> {
    store: S,
    clock: C,
}

impl<S, C> Application<S, C>
where
    S: JournalStore,
    C: Clock,
{
    pub fn new(store: S, clock: C) -> Self {
        Self { store, clock }
    }

    pub fn execute(&self, request: QueryRequest) -> AppResult<Outcome> {
        let mut entries = self
            .store
            .list_entries()
            .map_err(AppError::Store)?
            .into_iter()
            .filter(|entry| request.selection.matches_metadata(entry))
            .collect::<Vec<_>>();
        let mut cache: ContentCache = BTreeMap::new();

        if !request.selection.content_terms.is_empty() {
            entries.retain(|entry| match self.load_entry(entry, &mut cache) {
                Ok(content) => request.selection.matches_content(content),
                Err(_) => true,
            });
        }

        if let View::Search(query) = &request.view {
            let mut candidates = Vec::new();
            for entry in entries {
                let Ok(content) = self.load_entry(&entry, &mut cache) else {
                    continue;
                };
                let Some(summary) =
                    extract_section(content, "Summary").filter(|value| !value.is_empty())
                else {
                    continue;
                };
                candidates.push(SearchCandidate { entry, summary });
            }

            let mut results = rank(&candidates, query, self.clock.now_unix());
            let limit = request.limit.map_or(10, |value| value.get());
            results.truncate(limit);
            return Ok(Outcome::Search(results));
        }

        if let Some(limit) = request.limit {
            entries.truncate(limit.get());
        }
        if entries.is_empty() {
            return Ok(Outcome::EntriesNotFound);
        }

        let view = if request.limit.is_some()
            && !request.view_explicit
            && matches!(request.view, View::List)
        {
            View::Section(SectionName::summary())
        } else {
            request.view
        };

        match view {
            View::List => {
                let read_errors = entries
                    .iter()
                    .filter_map(|entry| match cache.get(&entry.name) {
                        Some(Err(error)) => Some(EntryBlock {
                            entry: entry.clone(),
                            body: EntryBody::ReadError(error.clone()),
                        }),
                        _ => None,
                    })
                    .collect();
                Ok(Outcome::Listing {
                    entries,
                    read_errors,
                })
            }
            View::Full => Ok(Outcome::Blocks(
                entries
                    .into_iter()
                    .map(|entry| {
                        let body = match self.load_entry(&entry, &mut cache) {
                            Ok(content) => EntryBody::Content(content.clone()),
                            Err(error) => EntryBody::ReadError(error.clone()),
                        };
                        EntryBlock { entry, body }
                    })
                    .collect(),
            )),
            View::Section(heading) => Ok(Outcome::Blocks(
                entries
                    .into_iter()
                    .map(|entry| {
                        let body = match self.load_entry(&entry, &mut cache) {
                            Ok(content) => extract_section(content, heading.as_str())
                                .map(EntryBody::Content)
                                .unwrap_or_else(|| EntryBody::MissingSection(heading.clone())),
                            Err(error) => EntryBody::ReadError(error.clone()),
                        };
                        EntryBlock { entry, body }
                    })
                    .collect(),
            )),
            View::Search(_) => unreachable!("search returns before non-search view handling"),
        }
    }

    fn load_entry<'a>(
        &self,
        entry: &EntryMetadata,
        cache: &'a mut ContentCache,
    ) -> &'a Result<String, String> {
        cache.entry(entry.name.clone()).or_insert_with(|| {
            self.store
                .read_entry(&entry.name)
                .map_err(|error| error.to_string())
        })
    }
}

type ContentCache = BTreeMap<EntryName, Result<String, String>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    EntriesNotFound,
    Listing {
        entries: Vec<EntryMetadata>,
        read_errors: Vec<EntryBlock>,
    },
    Blocks(Vec<EntryBlock>),
    Search(Vec<SearchMatch>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryBlock {
    pub entry: EntryMetadata,
    pub body: EntryBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryBody {
    Content(String),
    MissingSection(SectionName),
    ReadError(String),
}
