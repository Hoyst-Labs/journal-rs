use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    Success,
    Failure,
}

impl ExitStatus {
    pub fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliExecution {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exit: ExitStatus,
}

impl CliExecution {
    pub(crate) fn from_result(result: Result<String, AppError>) -> Self {
        match result {
            Ok(output) => Self {
                stdout: Some(output),
                stderr: None,
                exit: ExitStatus::Success,
            },
            Err(error) => Self {
                stdout: None,
                stderr: Some(error.to_string()),
                exit: ExitStatus::Failure,
            },
        }
    }
}
