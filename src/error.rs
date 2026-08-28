use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::domain::EntryName;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Usage(UsageError),
    CurrentDirectory(io::Error),
    Store(StoreError),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(error) => error.fmt(formatter),
            Self::CurrentDirectory(error) => {
                write!(formatter, "Failed to determine current directory: {error}")
            }
            Self::Store(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Usage(error) => Some(error),
            Self::CurrentDirectory(error) => Some(error),
            Self::Store(error) => Some(error),
        }
    }
}

impl From<UsageError> for AppError {
    fn from(value: UsageError) -> Self {
        Self::Usage(value)
    }
}

impl From<StoreError> for AppError {
    fn from(value: StoreError) -> Self {
        Self::Store(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsageError {
    UnknownOption(String),
    MissingTypeHeading,
    MissingFilesQuery,
    MissingSince,
    InvalidSince,
    MissingBetweenStart,
    MissingBetweenEnd,
    InvalidBetweenValues,
    InvalidBetweenRange,
    MissingFilter,
    EmptyFilter,
    MissingLatest,
    InvalidLatest,
    MissingSearchQuery,
    SummaryTypeConflict,
    SearchFilterConflict,
    SinceBetweenConflict,
}

impl fmt::Display for UsageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnknownOption(option) => return write!(formatter, "Unknown option: {option}"),
            Self::MissingTypeHeading => "--type requires a heading value.",
            Self::MissingFilesQuery => "--files requires a date or timestamp query.",
            Self::MissingSince => {
                "--since requires a value in YYYY-MM-DD or YYYY-MM-DD-HHmm format."
            }
            Self::InvalidSince => "--since value must be YYYY-MM-DD or YYYY-MM-DD-HHmm.",
            Self::MissingBetweenStart => "--between requires a start value.",
            Self::MissingBetweenEnd => "--between requires an end value.",
            Self::InvalidBetweenValues => "--between values must be YYYY-MM-DD or YYYY-MM-DD-HHmm.",
            Self::InvalidBetweenRange => "Invalid --between range: start must be <= end.",
            Self::MissingFilter => "--filter requires a pipe-delimited value.",
            Self::EmptyFilter => "--filter must contain at least one non-empty term.",
            Self::MissingLatest | Self::InvalidLatest => "--latest requires a positive integer.",
            Self::MissingSearchQuery => "--search requires a query string.",
            Self::SummaryTypeConflict => {
                "Cannot use --summary with --type. Use one display selector."
            }
            Self::SearchFilterConflict => "--search and --filter cannot be used together.",
            Self::SinceBetweenConflict => "Cannot use --since with --between.",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for UsageError {}

#[derive(Debug)]
pub enum StoreError {
    List {
        path: PathBuf,
        source: io::Error,
    },
    Read {
        entry: EntryName,
        path: PathBuf,
        source: io::Error,
    },
    MissingJournalDirectory,
    EscapesRoot(EntryName),
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::List { path, source } => {
                write!(formatter, "Failed to list {}: {source}", path.display())
            }
            Self::Read { path, source, .. } => {
                write!(formatter, "Failed to read {}: {source}", path.display())
            }
            Self::MissingJournalDirectory => formatter.write_str("No journal entries found."),
            Self::EscapesRoot(entry) => {
                write!(
                    formatter,
                    "Refusing to read entry outside journal root: {entry}"
                )
            }
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::List { source, .. } | Self::Read { source, .. } => Some(source),
            Self::MissingJournalDirectory | Self::EscapesRoot(_) => None,
        }
    }
}
