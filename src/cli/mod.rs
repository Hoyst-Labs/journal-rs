mod exit;
pub(crate) mod help;
mod parse;

pub use exit::{CliExecution, ExitStatus};
pub(crate) use parse::parse_args;

use crate::app::Application;
use crate::error::AppResult;
use crate::output::render;
use crate::ports::{Clock, JournalStore};

pub(crate) fn execute<S, C>(args: &[String], application: &Application<S, C>) -> AppResult<String>
where
    S: JournalStore,
    C: Clock,
{
    match parse_args(args)? {
        crate::domain::Command::Help => Ok(help::HELP_TEXT.to_string()),
        crate::domain::Command::Query(request) => {
            application.execute(request).map(|outcome| render(&outcome))
        }
    }
}
