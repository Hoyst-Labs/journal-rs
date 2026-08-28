mod adapters;
mod app;
mod cli;
pub mod domain;
mod error;
mod output;
mod ports;

use std::path::Path;

use adapters::{FsJournalStore, SystemClock};

pub use app::{Application, EntryBlock, EntryBody, Outcome};
pub use cli::{CliExecution, ExitStatus};
pub use domain::{
    Command, DateWindow, EntryMetadata, EntryName, EntryNameError, EntrySelection, JournalMoment,
    QueryRequest, SearchMatch, SearchQuery, SectionName, View,
};
pub use error::{AppError, AppResult, StoreError, UsageError};
pub use ports::{Clock, JournalStore};

pub fn run(args: &[String]) -> AppResult<String> {
    let current_dir = std::env::current_dir().map_err(AppError::CurrentDirectory)?;
    run_from(args, &current_dir)
}

pub fn run_cli(args: &[String]) -> CliExecution {
    CliExecution::from_result(run(args))
}

fn run_from(args: &[String], current_dir: &Path) -> AppResult<String> {
    let application = Application::new(FsJournalStore::new(current_dir), SystemClock);
    cli::execute(args, &application)
}
