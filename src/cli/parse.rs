use std::num::NonZeroUsize;

use crate::domain::{
    Command, DateWindow, EntrySelection, JournalMoment, QueryRequest, SearchQuery, SectionName,
    View,
};
use crate::error::UsageError;

pub fn parse_args(args: &[String]) -> Result<Command, UsageError> {
    let positional_command = args.first().map(String::as_str);
    let help_requested =
        args.iter().any(|arg| arg == "--help" || arg == "-h") || positional_command == Some("help");
    if help_requested {
        return Ok(Command::Help);
    }

    let mut summary_requested = positional_command == Some("summary");
    let mut full_requested = positional_command == Some("full");
    let mut type_heading: Option<SectionName> = None;
    let mut search_query: Option<SearchQuery> = None;
    let mut file_prefix: Option<String> = None;
    let mut since: Option<JournalMoment> = None;
    let mut between: Option<(JournalMoment, JournalMoment)> = None;
    let mut content_terms = Vec::new();
    let mut limit: Option<NonZeroUsize> = None;

    if positional_command == Some("search") {
        search_query = Some(parse_search_query(args.get(1))?);
    }
    if matches!(positional_command, Some("files" | "full")) {
        file_prefix = args
            .get(1)
            .filter(|value| !value.starts_with("--"))
            .cloned();
    }

    let has_known_command = matches!(
        positional_command,
        Some("help" | "summary" | "full" | "files" | "search")
    );
    let mut index = usize::from(has_known_command);
    while index < args.len() {
        match args[index].as_str() {
            "--summary" => {
                summary_requested = true;
                index += 1;
            }
            "--full" => {
                full_requested = true;
                index += 1;
            }
            "--type" => {
                let value = next_value(args, index, UsageError::MissingTypeHeading)?;
                type_heading = Some(SectionName::new(value).ok_or(UsageError::MissingTypeHeading)?);
                index += 2;
            }
            "--files" => {
                file_prefix = Some(next_value(args, index, UsageError::MissingFilesQuery)?);
                index += 2;
            }
            "--since" => {
                let value = next_value(args, index, UsageError::MissingSince)?;
                since = Some(JournalMoment::parse(value).ok_or(UsageError::InvalidSince)?);
                index += 2;
            }
            "--between" => {
                let start = args
                    .get(index + 1)
                    .filter(|value| !value.starts_with("--"))
                    .cloned()
                    .ok_or(UsageError::MissingBetweenStart)?;
                let end = args
                    .get(index + 2)
                    .filter(|value| !value.starts_with("--"))
                    .cloned()
                    .ok_or(UsageError::MissingBetweenEnd)?;
                let start = JournalMoment::parse(start).ok_or(UsageError::InvalidBetweenValues)?;
                let end = JournalMoment::parse(end).ok_or(UsageError::InvalidBetweenValues)?;
                between = Some((start, end));
                index += 3;
            }
            "--filter" => {
                let pattern = next_value(args, index, UsageError::MissingFilter)?;
                content_terms = parse_filter_terms(&pattern)?;
                index += 2;
            }
            "--latest" => {
                let value = next_value(args, index, UsageError::MissingLatest)?;
                let value = value
                    .parse::<usize>()
                    .ok()
                    .and_then(NonZeroUsize::new)
                    .ok_or(UsageError::InvalidLatest)?;
                limit = Some(value);
                index += 2;
            }
            "--search" => {
                search_query = Some(parse_search_query(args.get(index + 1))?);
                index += 2;
            }
            token if token.starts_with("--") => {
                return Err(UsageError::UnknownOption(token.to_string()));
            }
            _ => index += 1,
        }
    }

    if !has_known_command && !args.is_empty() && !args[0].starts_with("--") && file_prefix.is_none()
    {
        file_prefix = Some(args[0].clone());
    }

    if summary_requested && type_heading.is_some() {
        return Err(UsageError::SummaryTypeConflict);
    }
    if search_query.is_some() && !content_terms.is_empty() {
        return Err(UsageError::SearchFilterConflict);
    }
    if since.is_some() && between.is_some() {
        return Err(UsageError::SinceBetweenConflict);
    }

    let window = if let Some((start, end)) = between {
        Some(DateWindow::between(start, end).ok_or(UsageError::InvalidBetweenRange)?)
    } else {
        since.map(DateWindow::Since)
    };
    let view_explicit =
        search_query.is_some() || type_heading.is_some() || summary_requested || full_requested;
    let view = if let Some(query) = search_query {
        View::Search(query)
    } else if let Some(heading) = type_heading {
        View::Section(heading)
    } else if summary_requested {
        View::Section(SectionName::summary())
    } else if full_requested {
        View::Full
    } else {
        View::List
    };

    Ok(Command::Query(QueryRequest {
        selection: EntrySelection {
            file_prefix,
            window,
            content_terms,
        },
        view,
        view_explicit,
        limit,
    }))
}

fn next_value(args: &[String], index: usize, missing: UsageError) -> Result<String, UsageError> {
    args.get(index + 1)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .ok_or(missing)
}

fn parse_search_query(value: Option<&String>) -> Result<SearchQuery, UsageError> {
    value
        .filter(|query| !query.starts_with("--"))
        .cloned()
        .and_then(SearchQuery::new)
        .ok_or(UsageError::MissingSearchQuery)
}

fn parse_filter_terms(pattern: &str) -> Result<Vec<String>, UsageError> {
    let terms = pattern
        .split('|')
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    (!terms.is_empty())
        .then_some(terms)
        .ok_or(UsageError::EmptyFilter)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn rejects_conflicts_and_invalid_ranges() {
        assert_eq!(
            parse_args(&strings(&["--summary", "--type", "Summary"])),
            Err(UsageError::SummaryTypeConflict)
        );
        assert_eq!(
            parse_args(&strings(&[
                "--since",
                "2026-05-01",
                "--between",
                "2026-04-01",
                "2026-04-30"
            ])),
            Err(UsageError::SinceBetweenConflict)
        );
        assert_eq!(
            parse_args(&strings(&["--between", "2026-05-02", "2026-05-01"])),
            Err(UsageError::InvalidBetweenRange)
        );
    }

    #[test]
    fn parses_combined_query_into_typed_state() {
        let command = parse_args(&strings(&[
            "--type",
            "Next Actions",
            "--since",
            "2026-04-26",
            "--filter",
            "deploy|release",
        ]))
        .unwrap();
        let Command::Query(request) = command else {
            panic!("expected query");
        };
        assert_eq!(request.selection.content_terms, vec!["deploy", "release"]);
        assert!(matches!(request.view, View::Section(_)));
        assert!(request.view_explicit);
    }

    #[test]
    fn preserves_positional_alias_and_bare_prefix_behavior() {
        let Command::Query(files) = parse_args(&strings(&["files", "2026-04-04-1054"])).unwrap()
        else {
            panic!("expected query");
        };
        assert_eq!(
            files.selection.file_prefix.as_deref(),
            Some("2026-04-04-1054")
        );

        let Command::Query(bare) = parse_args(&strings(&["2026-04-04"])).unwrap() else {
            panic!("expected query");
        };
        assert_eq!(bare.selection.file_prefix.as_deref(), Some("2026-04-04"));
    }

    #[test]
    fn rejects_missing_or_empty_values_and_zero_latest() {
        assert_eq!(
            parse_args(&strings(&["--search"])),
            Err(UsageError::MissingSearchQuery)
        );
        assert_eq!(
            parse_args(&strings(&["--search", ""])),
            Err(UsageError::MissingSearchQuery)
        );
        assert_eq!(
            parse_args(&strings(&["--latest", "0"])),
            Err(UsageError::InvalidLatest)
        );
        assert_eq!(
            parse_args(&strings(&["--filter", "|"])),
            Err(UsageError::EmptyFilter)
        );
    }
}
